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

use crate::{config::Config, error::AppError, task::TaskStore};

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

pub fn get_data() -> (Config, TaskStore) {
    let config_local_dir = dirs::config_local_dir();
    let data_local_dir = dirs::data_local_dir();

    let theme = load_from_file(
        config_local_dir,
        CONFIG_FILE,
        serde_yaml::from_str::<Config>,
        "config",
    );

    let task_store = load_from_file(
        data_local_dir,
        DATA_FILE,
        // NOTE: This doesn't work:
        // serde_json::from_str::<TaskStore>,
        |x| serde_json::from_str::<TaskStore>(x),
        "task data",
    );

    (theme, task_store)
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

pub fn save_data(config: &Config, task_store: &TaskStore) {
    save_to_file(
        dirs::data_local_dir(),
        DATA_FILE,
        || serde_json::to_string_pretty(task_store),
        "data",
    );

    save_to_file(
        dirs::config_local_dir(),
        CONFIG_FILE,
        || serde_yaml::to_string(config),
        "config",
    );
}
