use super::BoardTableActor;
use crate::core::engine::{
    EngineActor, GenMoveMessage, ListStonesMessage, OnStonesChangeMessage, PlayMessage,
};
use crate::core::entities::{OptCoords, StoneColor};
use crate::core::errors::AppError;
use crate::core::helpers::get_column_number;
use crate::view::board::{
    GetHighlightCoordsMessage, GetNextMoveInput, SetBlackStonesMessage, SetHightlightCoordsMessage,
    SetNextMoveInput, SetWhiteStonesMessage,
};
use crate::view::events::{Event, EventSideEffect, OnEventMessage};
use actix::prelude::*;
use actix::Addr;
use log::{debug, error};
use std::convert::{TryFrom, TryInto};
use termion::event::Key;

pub struct BoardControllerActor {
    board_table: Addr<BoardTableActor>,
    engine_addr: Addr<EngineActor>,
    char_range: [char; 19],
    number_range: [char; 10],
    player_color: StoneColor,
    ai_color: StoneColor,
}

impl Actor for BoardControllerActor {
    type Context = Context<Self>;
}

impl Handler<OnStonesChangeMessage> for BoardControllerActor {
    type Result = ();

    fn handle(&mut self, _: OnStonesChangeMessage, ctx: &mut Context<Self>) -> Self::Result {
        ctx.notify(RefreshStonesMessage {});
    }
}

pub struct RefreshStonesMessage {}

impl Message for RefreshStonesMessage {
    type Result = Result<(), AppError>;
}

impl Handler<RefreshStonesMessage> for BoardControllerActor {
    type Result = ResponseActFuture<Self, Result<(), AppError>>;

    fn handle(&mut self, _: RefreshStonesMessage, _: &mut Context<Self>) -> Self::Result {
        let list_white_stones_fut = self.engine_addr.send(ListStonesMessage {
            color: StoneColor::White,
        });

        let list_black_stones_fut = self.engine_addr.send(ListStonesMessage {
            color: StoneColor::Black,
        });

        let wrapped_future = actix::fut::wrap_future::<_, Self>(async move {
            (list_white_stones_fut.await, list_black_stones_fut.await)
        });

        Box::pin(wrapped_future.map(
            |wrapped_future_result, actor, _ctx| match wrapped_future_result {
                (Ok(Ok(white_stones)), Ok(Ok(black_stones))) => {
                    actor.board_table.do_send(SetWhiteStonesMessage {
                        stones: white_stones,
                    });

                    actor.board_table.do_send(SetBlackStonesMessage {
                        stones: black_stones,
                    });

                    Ok(())
                }
                _ => Err(AppError {
                    message: "cant refresh stones".to_string(),
                }),
            },
        ))
    }
}

pub struct UpdateHightlightCoordsMessage {}

impl Message for UpdateHightlightCoordsMessage {
    type Result = ();
}

impl Handler<UpdateHightlightCoordsMessage> for BoardControllerActor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _: UpdateHightlightCoordsMessage, _: &mut Context<Self>) -> Self::Result {
        let get_next_move_input = self.board_table.send(GetNextMoveInput {});

        let wrapped_future = actix::fut::wrap_future::<_, Self>(get_next_move_input);

        Box::pin(wrapped_future.map(|next_move_input_result, actor, _ctx| {
            match next_move_input_result {
                Ok(next_move_input) => {
                    actor.board_table.do_send(SetHightlightCoordsMessage {
                        coords: parse_input_coords(next_move_input),
                    });
                    ()
                }
                Err(_) => (),
            }
        }))
    }
}

impl BoardControllerActor {
    pub fn new(engine_addr: Addr<EngineActor>, board_table: Addr<BoardTableActor>) -> Self {
        BoardControllerActor {
            board_table,
            engine_addr,
            player_color: StoneColor::Black,
            ai_color: StoneColor::White,
            char_range: [
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
                'R', 'S', 'T',
            ],
            number_range: ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
        }
    }

    fn set_next_move_input(&self, text: String) {
        self.board_table.do_send(SetNextMoveInput { text });
    }
}

impl Handler<OnEventMessage<Key>> for BoardControllerActor {
    type Result = ResponseActFuture<Self, EventSideEffect>;

    fn handle(&mut self, msg: OnEventMessage<Key>, ctx: &mut Context<Self>) -> Self::Result {
        let reply = |actor: &BoardControllerActor, e: EventSideEffect| {
            Box::pin(async {}.into_actor(actor).map(|_, _, _| e))
        };

        match msg.event {
            Event::Input(input) => match input {
                Key::Char('q') | Key::Ctrl('c') => reply(self, EventSideEffect::QuitApp),
                Key::Char(char) if self.char_range.contains(&char) => {
                    let next_move_input_fut = self.board_table.send(GetNextMoveInput {});

                    let wrapped_future = actix::fut::wrap_future::<_, Self>(next_move_input_fut);

                    Box::pin(wrapped_future.map(
                        move |next_move_input_result, actor, ctx_wrapped| {
                            if let Ok(next_move_input) = next_move_input_result {
                                let mut next_move_input = next_move_input.clone();

                                if next_move_input.is_empty() {
                                    next_move_input.push(char);
                                    actor.set_next_move_input(next_move_input);
                                }

                                ctx_wrapped.notify(UpdateHightlightCoordsMessage {});
                            }

                            EventSideEffect::None
                        },
                    ))
                }
                Key::Char(char) if self.number_range.contains(&char) => {
                    let next_move_input_fut = self.board_table.send(GetNextMoveInput {});

                    let wrapped_future = actix::fut::wrap_future::<_, Self>(next_move_input_fut);

                    Box::pin(wrapped_future.map(
                        move |next_move_input_result, actor, ctx_wrapped| {
                            if let Ok(next_move_input) = next_move_input_result {
                                let mut next_move_input = next_move_input.clone();

                                if !next_move_input.is_empty() && next_move_input.len() < 3 {
                                    next_move_input.push(char);
                                    actor.set_next_move_input(next_move_input);
                                }

                                ctx_wrapped.notify(UpdateHightlightCoordsMessage {});
                            }

                            EventSideEffect::None
                        },
                    ))
                }
                Key::Backspace => {
                    let next_move_input_fut = self.board_table.send(GetNextMoveInput {});

                    let wrapped_future = actix::fut::wrap_future::<_, Self>(next_move_input_fut);

                    Box::pin(wrapped_future.map(
                        move |next_move_input_result, actor, ctx_wrapped| {
                            if let Ok(next_move_input) = next_move_input_result {
                                let mut next_move_input = next_move_input.clone();

                                if !next_move_input.is_empty() {
                                    next_move_input.pop();
                                    actor.set_next_move_input(next_move_input);
                                }

                                ctx_wrapped.notify(UpdateHightlightCoordsMessage {});
                            }

                            EventSideEffect::None
                        },
                    ))
                }
                Key::Char('\n') => {
                    let highlight_coords_fut = self.board_table.send(GetHighlightCoordsMessage {});

                    let wrapped_future = actix::fut::wrap_future::<_, Self>(highlight_coords_fut);

                    Box::pin(wrapped_future.map(|highlight_coords, actor, ctx_wrapped| {
                        actor.engine_addr.do_send(PlayMessage {
                            color: actor.player_color,
                            position: highlight_coords
                                .unwrap()
                                .as_ref()
                                .try_into()
                                .map_err(|_| AppError {
                                    message: "invalid color".to_string(),
                                })
                                .unwrap(),
                            listener: ctx_wrapped.address().recipient(),
                        });

                        ctx_wrapped.notify(RefreshStonesMessage {});
                        actor.set_next_move_input("".to_string());
                        ctx_wrapped.notify(UpdateHightlightCoordsMessage {});

                        actor.engine_addr.do_send(GenMoveMessage {
                            color: actor.ai_color,
                            listener: ctx_wrapped.address().recipient(),
                        });

                        EventSideEffect::None
                    }))
                }
                _ => reply(self, EventSideEffect::None),
            },
            Event::Tick => reply(self, EventSideEffect::None),
        }
    }
}

fn parse_input_coords(mut input: String) -> OptCoords {
    let col: Option<u8> = if !input.is_empty() {
        Some(get_column_number(input.remove(0)))
    } else {
        None
    };

    let row: Option<u8> = if !input.is_empty() {
        input.parse().ok()
    } else {
        None
    };

    OptCoords { col, row }
}
