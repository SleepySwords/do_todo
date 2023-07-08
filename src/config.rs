use dirs;

use std::{error::Error, fs};

use crate::{app::TaskStore, theme::Theme};

const DIR: &str = "dotodo";

const CONFIG_FILE: &str = "config.yml";
const DATA_FILE: &str = "data.json";

pub fn get_data() -> (Theme, TaskStore) {
    let config_local_dir = dirs::config_local_dir();
    let data_local_dir = dirs::data_local_dir();

    let mut theme = Theme::default();

    if let Some(dir) = config_local_dir {
        let config_path = dir.join(DIR).join(CONFIG_FILE);

        if config_path.exists() {
            if let Ok(config) = fs::read_to_string(&config_path) {
                if let Ok(theme_config) = serde_yaml::from_str::<Theme>(&config) {
                    theme = theme_config;
                }
            }
        }
    }

    let mut task_store = TaskStore::default();

    if let Some(dir) = data_local_dir {
        let data_path = dir.join(DIR).join(DATA_FILE);

        if data_path.exists() {
            if let Ok(data) = fs::read_to_string(&data_path) {
                if let Ok(task_store_data) = serde_json::from_str::<TaskStore>(&data) {
                    task_store = task_store_data;
                }
            }
        }
    }

    (theme, task_store)
}

// FIX: proper error handling, pring data out if cannout save.
pub fn save_data(theme: &Theme, task_store: &TaskStore) -> Result<(), Box<dyn Error>> {
    let config_local_dir = dirs::config_local_dir();
    let data_local_dir = dirs::data_local_dir();

    if let Some(dir) = config_local_dir {
        let config_dir = dir.join(DIR);

        // NOTE: It's technically possible for OS-specific utils that this fn calls
        // to fail if the path already exists.
        fs::create_dir_all(config_dir.clone())?;

        fs::write(config_dir.join(CONFIG_FILE), serde_yaml::to_string(theme)?)?;
    }

    if let Some(dir) = data_local_dir {
        let data_dir = dir.join(DIR);

        fs::create_dir_all(data_dir.clone())?;

        fs::write(data_dir.join(DATA_FILE), serde_json::to_string(task_store)?)?;
    }

    Ok(())
}
