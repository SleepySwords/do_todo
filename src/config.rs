use dirs;

use std::{error::Error, fs};

use crate::{app::TaskStore, theme::Theme};

const CONFIG_PATH: &str = ".config/dotodo/config.yml";
const DATA_PATH: &str = ".local/share/dotodo/data.json";

pub fn get_data() -> (Theme, TaskStore) {
    if let Some(home_dir) = dirs::home_dir() {
        let config_path = home_dir.join(CONFIG_PATH);
        let data_path = home_dir.join(DATA_PATH);

        let mut theme = Theme::default();

        if config_path.exists() {
            if let Ok(config) = fs::read_to_string(&config_path) {
                if let Ok(theme_config) =
                    serde_yaml::from_str::<Theme>(&config) {
                    theme = theme_config;
                }
            }
        }

        let mut task_store = TaskStore::default();

        if data_path.exists() {
            if let Ok(data) = fs::read_to_string(&data_path) {
                if let Ok(task_store_data)
                    = serde_json::from_str::<TaskStore>(&data) {
                    task_store = task_store_data;
                }
            }
        }

        (theme, task_store)
    } else {
        eprintln!("WARNING: Couldn't find home directory");

        Default::default()
    }
}

// FIX: proper error handling, pring data out if cannout save.
pub fn save_data(theme: &Theme, task_store: &TaskStore) -> Result<(), Box<dyn Error>> {
    let config_path = dirs::home_dir().unwrap().join(CONFIG_PATH);
    let data_path = dirs::home_dir().unwrap().join(DATA_PATH);

    fs::create_dir_all(config_path.parent().unwrap())?;
    fs::create_dir_all(data_path.parent().unwrap())?;

    fs::write(
        dirs::home_dir().unwrap().join(config_path),
        serde_yaml::to_string(theme)?,
    )?;

    fs::write(
        dirs::home_dir().unwrap().join(data_path),
        serde_json::to_string(task_store)?,
    )?;

    Ok(())
}
