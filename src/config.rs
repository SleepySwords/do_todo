use dirs;

use std::{error::Error, fs};

use crate::{app::TaskStore, theme::Theme, task::Task};

const CONFIG_PATH: &str = ".config/dotodo/config.yml";
const DATA_PATH: &str = ".config/dotodo/data.json";

pub fn get_data() -> Result<(Theme, TaskStore), Box<dyn Error>> {
    match dirs::home_dir() {
        Some(home_dir) => {
            let config_path = home_dir.join(CONFIG_PATH);
            let data_path = home_dir.join(DATA_PATH);

            let theme = if !config_path.exists() {
                Theme::default()
            } else {
                let config_contents = fs::read_to_string(&config_path);
                match config_contents {
                    Ok(file) => serde_yaml::from_str::<Theme>(&file)?,
                    Err(_) => {
                        let theme = Theme::default();
                        fs::write(&config_path, serde_yaml::to_string(&theme)?)?;
                        theme
                    }
                }
            };

            let task_store = if !data_path.exists() {
                TaskStore::default()
            } else {
                let data_contents = fs::read_to_string(&data_path);
                match data_contents {
                    Ok(file) => serde_json::from_str::<TaskStore>(&file)?,
                    Err(_) => {
                        let tasks: Vec<Task> = vec![];
                        fs::write(&data_path, serde_json::to_string(&tasks)?)?;
                        TaskStore::default()
                    }
                }
            };
            Ok((theme, task_store))
        }
        None => {
            println!("Not found");
            Ok((Theme::default(), TaskStore::default()))
        }
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
