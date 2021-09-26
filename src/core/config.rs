use super::errors::AppError;
use directories::ProjectDirs;
use log::info;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Deserialize)]
pub struct AppConfig {
    #[serde(default = "get_default_general_config")]
    pub general: GeneralConfig,

    #[serde(default = "get_default_engine_config")]
    pub engine: EngineConfig,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct GeneralConfig {}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EngineConfig {
    #[serde(default = "get_default_engine_bin")]
    pub bin: String,
    #[serde(default = "get_default_engine_args")]
    pub args: Vec<String>,
}

pub fn get_app_config(config_path: Option<&str>) -> Result<AppConfig, AppError> {
    let config_file_path: PathBuf = if let Some(str_value) = config_path {
        PathBuf::from(&str_value)
    } else {
        get_default_app_config_path()?
    };

    info!(
        "loading config from '{}'",
        config_file_path.to_string_lossy()
    );

    let config_file_content: String = if config_file_path.exists() {
        let mut file = File::open(&config_file_path).map_err(|e| AppError {
            message: format!(
                "Error opening the file '{}': {}",
                &config_file_path.to_string_lossy(),
                &e.to_string()
            ),
        })?;
        let mut contents = String::new();

        file.read_to_string(&mut contents).map_err(|e| AppError {
            message: format!(
                "Error reading the file '{}': {}",
                &config_file_path.to_string_lossy(),
                &e.to_string()
            ),
        })?;

        contents
    } else {
        "empty: true".to_string()
    };

    serde_yaml::from_str(&config_file_content).map_err(|e| AppError {
        message: format!(
            "Error parsing content of the file '{}': {}",
            &config_file_path.to_string_lossy(),
            &e.to_string()
        ),
    })
}

fn get_default_app_config_path() -> Result<PathBuf, AppError> {
    match ProjectDirs::from("Com", "Anderson Danilo", env!("CARGO_PKG_NAME")) {
        Some(proj_dirs) => {
            let config_file_path = proj_dirs.config_dir().join("config.yml");

            Ok(config_file_path)
        }
        None => Err(AppError {
            message: "Can't get app project dir".to_string(),
        }),
    }
}

fn get_default_engine_config() -> EngineConfig {
    EngineConfig {
        bin: get_default_engine_bin(),
        args: get_default_engine_args(),
    }
}

fn get_default_general_config() -> GeneralConfig {
    GeneralConfig {}
}

fn get_default_engine_bin() -> String {
    "gnugo".to_string()
}

fn get_default_engine_args() -> Vec<String> {
    vec![]
}
