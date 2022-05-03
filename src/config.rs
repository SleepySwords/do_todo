use std::{fs, error::Error, path::Path};

use crate::{task::Task, theme::Theme};

pub fn get_config() -> Result<(Theme, Vec<Task>), Box<dyn Error>> {
    match dirs::home_dir() {
        Some(home_dir) => {
            let config_path = Path::new(&home_dir).join(".config/dtb/config.yml");
            let data_path = Path::new(&home_dir).join(".config/dtb/data.json");
            let config_contents = fs::read_to_string(&config_path);
            let data_contents = fs::read_to_string(&data_path);
            Ok((
                match config_contents {
                    Ok(file) => {
                        serde_yaml::from_str::<Theme>(&file)?
                    },
                    Err(_) => {
                        let theme = Theme::default();
                        fs::write(&config_path, serde_yaml::to_string(&theme)?)?;
                        theme
                    }
                },
                match data_contents {
                    Ok(file) => {
                        serde_json::from_str::<Vec<Task>>(&file)?
                    },
                    Err(_) => {
                        let tasks: Vec<Task> = vec![];
                        fs::write(&data_path, serde_json::to_string(&tasks)?)?;
                        vec![]
                    }
                }
            ))
        }
        None => {
            println!("Not found");
            Ok((Theme::default(), vec![]))
        }
    }
}