use std::{error::Error, fs, path::Path};

use crate::{app::TaskStore, task::Task, theme::Theme};

// FIX: Proper handling, data should not be stored in the config file and needes testing
// Add a swp file.
pub fn get_data() -> Result<(Theme, TaskStore), Box<dyn Error>> {
    match dirs::home_dir() {
        Some(home_dir) => {
            let config_path = Path::new(&home_dir).join(".config/dotodo/config.yml");
            let data_path = Path::new(&home_dir).join(".config/dotodo/data.json");

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
    let dotodo_path = dirs::home_dir().unwrap().join(".config/dotodo/");

    fs::create_dir_all(dotodo_path)?;
    fs::write(
        dirs::home_dir().unwrap().join(".config/dotodo/data.json"),
        serde_json::to_string(task_store)?,
    )?;
    fs::write(
        dirs::home_dir().unwrap().join(".config/dotodo/config.yml"),
        serde_yaml::to_string(theme)?,
    )?;
    Ok(())
}
