use std::{
    fs,
    io::Write,
    io::{stdin, stdout},
    path::PathBuf,
    process::exit,
};

// TODOs:
// - Create a custom error type and return it from functions to handle it
// outside of them

use crate::{
    config::{Config, DataSource},
    data::{
        data_store::DataTaskStore, json_data_store::JsonDataStore, todoist::todoist_main::sync,
    },
    error::AppError,
    storage::json::{legacy::legacy_task::LegacyTaskStore, version::JSONVersion},
    utils,
};

const DIR: &str = "dotodo";

const CONFIG_FILE: &str = "config.yml";
const DATA_FILE: &str = "data.json";

fn should_overwrite(message: String) -> std::io::Result<bool> {
    println!("{}", message);
    print!(r"Continue (y/n)? ");
    stdout().flush()?;

    let mut answer = String::new();
    stdin().read_line(&mut answer)?;

    let answer = answer.trim();
    let answer_check_len = answer.len().clamp(0, 2);
    let should_load = *answer == "yes"[..answer_check_len];

    Ok(should_load)
}

fn fail_load_string(common_name: &str, file_name: &str, err: AppError) -> String {
    format!("Failed to load {common_name} '{file_name}', {err}. If you continue, this will be overwritten.")
}

fn fail_write_string(common_name: &str, file_name: &str, err: AppError) -> String {
    format!("Failed to write {common_name} '{file_name}', {err}. If you continue, this will not be saved.")
}

// NOTE: Hacky, but could've saved me 6-8 duplicate changes already
fn load_from_file<T, F, E>(
    local_dir: Option<PathBuf>,
    file_name: &'static str,
    de_f: F,
    kind: &'static str,
) -> T
where
    T: Default,
    F: Fn(&str) -> Result<T, E>,
    E: Into<AppError>,
{
    if let Some(dir) = local_dir {
        let path = dir.join(DIR).join(file_name);

        if path.exists() {
            let deserialised = fs::read_to_string(&path)
                .map(|contents| de_f(&contents).map_err(|err| err.into()))
                .map_err(|err| err.into());

            match deserialised {
                Ok(Ok(de)) => {
                    return de;
                }
                Err(err) | Ok(Err(err)) => {
                    match should_overwrite(fail_load_string(kind, file_name, err)) {
                        Ok(true) => return Default::default(),
                        Ok(false) | Err(_) => exit(0),
                    }
                }
            }
        } else {
            eprintln!("{kind} file doesn't seem to exist - creating");
        }
    } else {
        eprintln!("Failed to determine {kind} directory on your system. Please report this issue at https://github.com/SleepySwords/do_todo/issues/new");
    }

    Default::default()
}

pub async fn get_data(is_debug: bool) -> (Config, Box<dyn DataTaskStore>) {
    let (data_local_dir, config_local_dir) = if is_debug {
        (
            Some(std::env::current_dir().unwrap()),
            Some(std::env::current_dir().unwrap()),
        )
    } else {
        (dirs::data_local_dir(), dirs::config_local_dir())
    };

    let config = load_from_file(
        config_local_dir,
        CONFIG_FILE,
        serde_yaml::from_str::<Config>,
        "config",
    );

    // let tasks = sync();
    let task_store: Box<dyn DataTaskStore> = match &config.data_source {
        DataSource::Json => {
            Box::new(load_from_file(
                data_local_dir,
                DATA_FILE,
                // NOTE: This doesn't work:
                // serde_json::from_str::<TaskStore>,
                |x| {
                    serde_json::from_str::<JSONVersion>(x)
                        .map(Into::<JsonDataStore>::into) // NOTE: we might do something similar as below
                        // if/when we introduce integers as ids again
                        // This is because for some reason, serde tags
                        // don't like int strings as keys
                        // See: https://github.com/serde-rs/serde/issues/2672
                        .or_else(|_| serde_json::from_str::<LegacyTaskStore>(x).map(|x| x.into()))
                },
                "task data",
            ))
        }
        DataSource::Todoist(todoist_auth) => Box::new(sync(todoist_auth).await),
    };

    (config, task_store)
}

fn save_to_file<T, F, E>(local_dir: Option<PathBuf>, file_name: &str, ser_f: F, kind: &str)
where
    T: AsRef<[u8]>,
    F: Fn() -> Result<T, E>,
    E: Into<AppError>,
{
    match local_dir {
        None => eprintln!("Failed to determine {kind} directory on your system. Please report this issue at https://github.com/SleepySwords/do_todo/issues/new"),
        Some(dir) => {
            let path = dir.join(DIR);

            loop {
                // NOTE: It's _technically_ possible for OS-specific utils that this fn calls
                // to fail if the path already exists.
                fs::create_dir_all(path.clone()).is_err().then(|| {
                    eprintln!("Failed to create config directory")
                });

                let serialized = ser_f();

                let result = serialized.map(|ser| {
                    fs::write(path.join(file_name), ser).map_err(|err| err.into())
                }).map_err(|err| err.into());

                if let Err(e) | Ok(Err(e)) = result {
                    match should_overwrite(fail_write_string(kind, file_name, e)) {
                        Ok(false) | Err(_) => continue,
                        _ => {},
                    }
                }
                break;
            }
        }
    }
}

pub fn save_config(config: &Config, task_store: Box<dyn DataTaskStore>) {
    task_store.save();

    save_to_file(
        if utils::IS_DEBUG {
            Some(std::env::current_dir().unwrap())
        } else {
            dirs::config_local_dir()
        },
        CONFIG_FILE,
        || serde_yaml::to_string(config),
        "config",
    );
}

pub fn save_task_json(task_store: &JsonDataStore, is_debug: bool) {
    let json = JSONVersion::V1(task_store.clone());

    save_to_file(
        if is_debug {
            Some(std::env::current_dir().unwrap())
        } else {
            dirs::data_local_dir()
        },
        DATA_FILE,
        || serde_json::to_string_pretty(&json),
        "data",
    );
}
