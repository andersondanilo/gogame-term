use super::errors::AppError;
use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, WriteLogger};
use std::fs::File;

pub fn get_logger_level_by_verbosity(verbosity: u64) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    }
}

pub fn init_logger(log_file_path: Option<&str>, log_level: LevelFilter) -> Result<(), AppError> {
    let mut loggers: Vec<Box<dyn SharedLogger>> =
        vec![SimpleLogger::new(log_level, Config::default())];

    if let Some(debug_file_path) = &log_file_path {
        loggers.push(WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(&debug_file_path).map_err(|e| AppError {
                message: format!(
                    "Error opening the debug file '{}': {}",
                    &debug_file_path,
                    &e.to_string()
                ),
            })?,
        ));
    }

    CombinedLogger::init(loggers).map_err(|e| AppError {
        message: format!("Error initializing the logger, {}", &e.to_string()),
    })?;

    Ok(())
}
