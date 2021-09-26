use super::errors::AppError;
use gtp::controller;
use gtp::Command;
use gtp::Response;
use log::{debug, warn};
use std::time::Duration;

pub struct Engine {
    gtp_engine: gtp::controller::Engine,
    cmd_timeout: Duration,
}

struct ResponseWrapper {
    cmd_name: String,
    response: gtp::Response,
}

impl ResponseWrapper {
    fn success_text(&self) -> Result<String, AppError> {
        match &self.response {
            Response::Error((_, text)) => {
                let err_msg = format!("cmd '{}' returned '{}'", &self.cmd_name, &text);

                warn!("{}", &err_msg);

                Err(AppError { message: err_msg })
            }
            Response::Result((_, text)) => Ok(text.clone()),
        }
    }
}

impl Engine {
    pub fn start(bin_path: &str, additional_args: &Vec<String>) -> Result<Self, AppError> {
        let mut args: Vec<String> = vec!["--mode".to_string(), "gtp".to_string()];
        args.append(&mut additional_args.clone());

        let str_args: Vec<&str> = args.iter().map(|s| s as &str).collect();
        let cmd_timeout = Duration::from_millis(500);
        let mut gtp_engine = controller::Engine::new(bin_path, &str_args);

        gtp_engine.start().map_err(|_| AppError {
            message: format!("Error starting engine '{}'", &bin_path),
        })?;

        Ok(Engine {
            gtp_engine,
            cmd_timeout,
        })
    }

    pub fn get_name(&mut self) -> Result<String, AppError> {
        let resp = self.send_and_await("name")?;

        resp.success_text()
    }

    pub fn showboard(&mut self) -> Result<String, AppError> {
        let resp = self.send_and_await("showboard")?;

        resp.success_text()
    }

    pub fn query_boardsize(&mut self) -> Result<u8, AppError> {
        let resp = self.send_and_await("query_boardsize")?;

        let text: String = resp.success_text()?;

        text.parse().map_err(|_| AppError {
            message: format!("invalid board size: {}", text),
        })
    }

    fn send_and_await(&mut self, cmd_name: &str) -> Result<ResponseWrapper, AppError> {
        let cmd = Command::cmd(cmd_name, |e| e);

        self.gtp_engine.send(cmd);

        debug!("sending command: {}", cmd_name);

        Ok(ResponseWrapper {
            cmd_name: String::from(cmd_name),
            response: self
                .gtp_engine
                .wait_response(self.cmd_timeout)
                .map_err(|_| {
                    let error_message = format!("Error calling comand '{}'", &cmd_name);

                    warn!("{}", &error_message);

                    AppError {
                        message: error_message,
                    }
                })
                .map(|resp| {
                    debug!("received text: {}", resp.text());
                    resp
                })?,
        })
    }
}
