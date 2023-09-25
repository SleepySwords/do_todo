use std::{
    error::Error,
    fs,
    io::Write,
    io::{stdin, stdout},
    path::PathBuf,
    process::exit,
};

// TODOs:
// - Create a custom error type and return it from functions to handle it
// outside of them

use crate::{app::TaskStore, error::AppError, theme::Theme};

const DIR: &str = "dotodo";

const CONFIG_FILE: &str = "config.yml";
const DATA_FILE: &str = "data.json";

fn should_load_if_de_failed(
    common_name: &str,
    file_name: &str,
    err: AppError,
) -> std::io::Result<bool> {
    print!(
        r"Failed to load {common_name} '{file_name}', {err}. If you continue, it will be overwritten.
Continue (y/n)? "
    );
    stdout().flush()?;

    let mut answer = String::new();
    stdin().read_line(&mut answer)?; // TODO: Handle Result

    let answer = answer.trim();
    let answer_check_len = answer.len().clamp(0, 2);
    let should_load = *answer == "yes"[..answer_check_len];

    Ok(should_load)
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
    E: Into<AppError>,
    F: Fn(&str) -> Result<T, E>,
{
    if let Some(dir) = local_dir {
        let path = dir.join(DIR).join(file_name);

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    let deserialized = de_f(&contents);

                    match deserialized {
                        Ok(de) => {
                            return de;
                        }
                        Err(err) => match should_load_if_de_failed(kind, file_name, err.into()) {
                            Ok(true) => return Default::default(),
                            Ok(false) | Err(_) => exit(0),
                        },
                    }
                }
                Err(err) => match should_load_if_de_failed(kind, file_name, err.into()) {
                    Ok(true) => return Default::default(),
                    Ok(false) | Err(_) => exit(0),
                },
            }
        } else {
            eprintln!("{kind} file doesn't seem to exist - creating");
        }
    } else {
        eprintln!("Failed to determine {kind} directory on your system. Please report this issue at https://github.com/SleepySwords/do_todo/issues/new");
    }

    Default::default()
}

pub fn get_data() -> (Theme, TaskStore) {
    let config_local_dir = dirs::config_local_dir();
    let data_local_dir = dirs::data_local_dir();

    let theme = load_from_file(
        config_local_dir,
        CONFIG_FILE,
        serde_yaml::from_str::<Theme>,
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
    F: FnOnce() -> Result<T, E>,
    E: Error,
{
    match local_dir {
        None => eprintln!("Failed to determine {kind} directory on your system. Please report this issue at https://github.com/SleepySwords/do_todo/issues/new"),
        Some(dir) => {
            let path = dir.join(DIR);

            // NOTE: It's _technically_ possible for OS-specific utils that this fn calls
            // to fail if the path already exists.
            fs::create_dir_all(path.clone()).is_err().then(|| {
                eprintln!("Failed to create config directory")
            });

            let serialized = ser_f();

            match serialized {
                Ok(ser) => {
                    if let Err(e) = fs::write(path.join(file_name), ser) {
                        eprintln!("Failed to write to {kind} file: {e}");
                    }
                },
                Err(e) => eprintln!("Failed to serialize {kind}: {e}")
            }
        }
    }
}

pub fn save_data(theme: &Theme, task_store: &TaskStore) {
    save_to_file(
        dirs::config_local_dir(),
        CONFIG_FILE,
        || serde_json::to_string(theme),
        "config",
    );

    save_to_file(
        dirs::data_local_dir(),
        DATA_FILE,
        || serde_json::to_string(task_store),
        "data",
    );
}
