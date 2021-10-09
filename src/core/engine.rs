use super::errors::AppError;
use crate::core::entities::{Coords, Stone, StoneColor};
use gtp::{controller, Command, Entity, EntityBuilder, Response};
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

enum ExpectedEntity {
    Vertex,
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

    fn success_entities(&self, expected: ExpectedEntity) -> Result<Vec<Entity>, AppError> {
        match &self.response {
            Response::Error((_, text)) => {
                let err_msg = format!("cmd '{}' returned '{}'", &self.cmd_name, &text);

                warn!("{}", &err_msg);

                Err(AppError { message: err_msg })
            }
            Response::Result((_, text)) => self
                .response
                .entities(|ep| {
                    while !ep.is_eof() {
                        match expected {
                            ExpectedEntity::Vertex => ep.vertex(),
                        };
                    }
                    ep
                })
                .map_err(|e| {
                    let err_msg = format!("cmd '{}' returned '{}'", &self.cmd_name, &text);

                    warn!("{}", &err_msg);

                    AppError { message: err_msg }
                }),
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
        let resp = self.send_and_await("name", |e| e)?;

        resp.success_text()
    }

    pub fn query_boardsize(&mut self) -> Result<u8, AppError> {
        let resp = self.send_and_await("query_boardsize", |e| e)?;

        let text: String = resp.success_text()?;

        text.parse().map_err(|_| AppError {
            message: format!("invalid board size: {}", text),
        })
    }

    pub fn play(&mut self, color: StoneColor, position: Coords) -> Result<(), AppError> {
        let resp = self.send_and_await("play", |e| {
            (match color {
                StoneColor::White => e.w(),
                StoneColor::Black => e.b(),
            })
            .v(position.vertex())
            .list()
        })?;

        resp.success_text()?;

        Ok(())
    }

    pub fn list_stones(&mut self, color: StoneColor) -> Result<Vec<Stone>, AppError> {
        let resp = self.send_and_await("list_stones", |e| match color {
            StoneColor::White => e.w(),
            StoneColor::Black => e.b(),
        })?;

        let entities = resp.success_entities(ExpectedEntity::Vertex)?;
        let mut stones: Vec<Stone> = vec![];

        for entity in entities {
            stones.push(self.parser_stone_with_color(color, &entity)?)
        }

        Ok(stones)
    }

    fn parser_stone_with_color(
        &self,
        color: StoneColor,
        entity: &Entity,
    ) -> Result<Stone, AppError> {
        match entity {
            Entity::Vertex((col, row)) => Ok(Stone {
                color,
                col: col.clone() as u8,
                row: row.clone() as u8,
            }),
            _ => Err(AppError {
                message: format!("Can't parse stone: {:?}", entity),
            }),
        }
    }

    fn send_and_await<T>(&mut self, cmd_name: &str, args: T) -> Result<ResponseWrapper, AppError>
    where
        T: Fn(&mut EntityBuilder) -> &mut EntityBuilder,
    {
        let cmd = Command::cmd(cmd_name, args);
        let cmd_string = cmd.to_string();

        self.gtp_engine.send(cmd);

        debug!("sending command: {}", &cmd_string);

        Ok(ResponseWrapper {
            cmd_name: cmd_string.clone(),
            response: self
                .gtp_engine
                .wait_response(self.cmd_timeout)
                .map_err(|_| {
                    let error_message = format!("Error calling comand '{}'", &cmd_string);

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
