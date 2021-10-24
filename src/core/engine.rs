use super::errors::AppError;
use crate::core::entities::{Coords, Stone, StoneColor};
use actix::prelude::*;
use gtp::{controller, Command, Entity, EntityBuilder, Response};
use log::{debug, warn};
use std::thread;
use std::time::{Duration, Instant};

pub struct OnStonesChangeMessage {}

impl Message for OnStonesChangeMessage {
    type Result = ();
}

pub struct EngineActor {
    gtp_engine: gtp::controller::Engine,
    default_timeout: Duration,
    genmove_timeout: Duration,
}

impl Actor for EngineActor {
    type Context = Context<Self>;
}

struct ResponseWrapper {
    cmd_name: String,
    response: gtp::Response,
}

enum ExpectedEntity {
    Vertex,
}

pub enum GenMoveResponse {
    Position(Coords),
    Resign,
    Pass,
}

pub struct QueryBoardSizeMessage {}

impl Message for QueryBoardSizeMessage {
    type Result = Result<u8, AppError>;
}

impl Handler<QueryBoardSizeMessage> for EngineActor {
    type Result = Result<u8, AppError>;

    fn handle(&mut self, msg: QueryBoardSizeMessage, _: &mut Context<Self>) -> Self::Result {
        let resp = self.send_and_await("query_boardsize", |e| e, self.default_timeout)?;

        let text: String = resp.success_text()?;

        text.parse().map_err(|_| AppError {
            message: format!("invalid board size: {}", text),
        })
    }
}

pub struct PlayMessage {
    pub color: StoneColor,
    pub position: Coords,
    pub listener: Recipient<OnStonesChangeMessage>,
}

impl Message for PlayMessage {
    type Result = Result<(), AppError>;
}

impl Handler<PlayMessage> for EngineActor {
    type Result = Result<(), AppError>;

    fn handle(&mut self, msg: PlayMessage, _: &mut Context<Self>) -> Self::Result {
        debug!("EngineActor [play-message]: started");
        let resp = self.send_and_await(
            "play",
            |e| {
                (match msg.color {
                    StoneColor::White => e.w(),
                    StoneColor::Black => e.b(),
                })
                .v(msg.position.vertex())
                .list()
            },
            self.default_timeout,
        )?;

        resp.success_text()?;

        debug!("EngineActor [play-message]: requesting on stones changed");

        msg.listener.do_send(OnStonesChangeMessage {});

        debug!("EngineActor [play-message]: finished");
        Ok(())
    }
}

pub struct ListStonesMessage {
    pub color: StoneColor,
}

impl Message for ListStonesMessage {
    type Result = Result<Vec<Stone>, AppError>;
}

impl Handler<ListStonesMessage> for EngineActor {
    type Result = Result<Vec<Stone>, AppError>;

    fn handle(&mut self, msg: ListStonesMessage, _: &mut Context<Self>) -> Self::Result {
        let resp = self.send_and_await(
            "list_stones",
            |e| match msg.color {
                StoneColor::White => e.w(),
                StoneColor::Black => e.b(),
            },
            self.default_timeout,
        )?;

        let entities = resp.success_entities(ExpectedEntity::Vertex)?;
        let mut stones: Vec<Stone> = vec![];

        for entity in entities {
            stones.push(self.parser_stone_with_color(msg.color, &entity)?)
        }

        Ok(stones)
    }
}

pub struct GenMoveMessage {
    pub color: StoneColor,
    pub listener: Recipient<OnStonesChangeMessage>,
}

impl Message for GenMoveMessage {
    type Result = Result<GenMoveResponse, AppError>;
}

impl Handler<GenMoveMessage> for EngineActor {
    type Result = Result<GenMoveResponse, AppError>;

    fn handle(&mut self, msg: GenMoveMessage, _: &mut Context<Self>) -> Self::Result {
        let resp = self.send_and_await(
            "genmove",
            |e| match msg.color {
                StoneColor::White => e.w(),
                StoneColor::Black => e.b(),
            },
            self.genmove_timeout,
        )?;

        let response = match resp.success_text()?.to_lowercase().as_str() {
            "pass" => Ok(GenMoveResponse::Pass),
            "resign" => Ok(GenMoveResponse::Resign),
            _ => Ok(GenMoveResponse::Position(resp.success_coords()?)),
        };

        msg.listener.do_send(OnStonesChangeMessage {});

        return response;
    }
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

    fn success_entity(&self, expected: ExpectedEntity) -> Result<Entity, AppError> {
        let mut entities = self.success_entities(expected)?;

        entities.pop().ok_or(AppError {
            message: "No entity found on response".to_string(),
        })
    }

    fn success_coords(&self) -> Result<Coords, AppError> {
        let entity = self.success_entity(ExpectedEntity::Vertex)?;

        match entity {
            Entity::Vertex((col, row)) => Ok(Coords {
                col: col.clone() as u8,
                row: row.clone() as u8,
            }),
            _ => Err(AppError {
                message: "Failed to parse vertex entity".to_string(),
            }),
        }
    }
}

impl EngineActor {
    pub fn new(bin_path: &str, additional_args: &Vec<String>) -> Result<Self, AppError> {
        let mut args: Vec<String> = vec!["--mode".to_string(), "gtp".to_string()];
        args.append(&mut additional_args.clone());

        let str_args: Vec<&str> = args.iter().map(|s| s as &str).collect();
        let default_timeout = Duration::from_millis(100);
        let genmove_timeout = Duration::from_millis(2000);
        let mut gtp_engine = controller::Engine::new(bin_path, &str_args);

        gtp_engine.start().map_err(|_| AppError {
            message: format!("Error starting engine '{}'", &bin_path),
        })?;
        Ok(EngineActor {
            gtp_engine,
            default_timeout,
            genmove_timeout,
        })
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

    fn send_and_await<T>(
        &mut self,
        cmd_name: &str,
        args: T,
        timeout: Duration,
    ) -> Result<ResponseWrapper, AppError>
    where
        T: Fn(&mut EntityBuilder) -> &mut EntityBuilder,
    {
        let start_instant = Instant::now();

        let cmd = Command::cmd(cmd_name, args);
        let cmd_string = cmd.to_string();

        debug!("EngineActor [send_and_await]: {}", cmd_name);

        self.gtp_engine.send(cmd);
        // the pool interval is the timeout / 4, and we never have the result in the first pool, so
        // it's better to delay the first pool (or change the gtp library, or even implment another)
        thread::sleep(Duration::from_millis(5));
        let response = self.gtp_engine.wait_response(timeout);

        Ok(ResponseWrapper {
            cmd_name: cmd_string.clone(),
            response: response
                .map_err(|_| {
                    let error_message = format!(
                        "Error calling command '{}', after {}ms",
                        &cmd_string,
                        start_instant.elapsed().as_millis()
                    );

                    warn!("{}", &error_message);

                    AppError {
                        message: error_message,
                    }
                })
                .map(|resp| {
                    debug!(
                        "cmd '{}' returned text: '{}', after {}ms",
                        &cmd_string,
                        resp.text(),
                        start_instant.elapsed().as_millis()
                    );
                    resp
                })?,
        })
    }
}
