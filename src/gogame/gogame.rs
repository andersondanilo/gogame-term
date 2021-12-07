use crate::core::engine::Engine;
use crate::core::entities::{Coords, Stone, StoneColor};
use crate::core::errors::AppError;
use crate::core::helpers::{parse_input_coords, TryPush};
use crate::core::theme::Theme;
use crate::core::{config, logger};
use crate::gogame::board::Board;
use crate::gogame::game_message::GameMessage;
use clap::{App, Arg};
use iced_futures::executor::Tokio;
use iced_native::{
    keyboard, subscription, Column, Command, Container, Element, Event, Length, Row, Subscription,
    Text,
};
use iced_tui::{Application, Style, TuiRenderer};
use std::sync::{Arc, Mutex};

const INPUT_CHAR_RANGE: [char; 19] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
];
const INPUT_NUMBER_RANGE: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(PartialEq)]
enum GtpStatus {
    Loading,
    Idle,
    Error,
}

pub struct GoGame {
    should_exit: Option<u8>,
    board: Option<Board>,
    next_move_input: String,
    player_color: StoneColor,
    gtp_engine: Arc<Mutex<Engine>>,
    gtp_status: GtpStatus,
    gtp_error: Option<String>,
    theme: Theme,
}

impl Application for GoGame {
    type Message = GameMessage;
    type Executor = Tokio;

    fn new() -> (GoGame, Command<Self::Message>) {
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("debug-file")
                    .short("d")
                    .long("debug-file")
                    .value_name("Debug file")
                    .help("Output debug to a file")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("v")
                    .short("v")
                    .multiple(true)
                    .help("Sets the level of verbosity"),
            )
            .get_matches();

        logger::init_logger(
            matches.value_of("debug-file"),
            logger::get_logger_level_by_verbosity(matches.occurrences_of("v")),
        )
        .unwrap();

        let app_config = config::get_app_config(matches.value_of("config")).unwrap();
        let gtp_engine = Arc::new(Mutex::new(
            Engine::new(&app_config.engine.bin, &app_config.engine.args).unwrap(),
        ));

        let theme = Theme::default();

        let state = GoGame {
            should_exit: None,
            board: None,
            next_move_input: "".to_string(),
            gtp_engine: gtp_engine.clone(),
            gtp_status: GtpStatus::Idle,
            player_color: StoneColor::Black,
            gtp_error: None,
            theme: theme.clone(),
        };

        (
            state,
            Command::perform(GoGame::load_board(gtp_engine, theme), |board| {
                GameMessage::BoardLoaded(board)
            }),
        )
    }

    fn should_exit(&self) -> Option<u8> {
        return self.should_exit;
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events().map(Self::Message::EventOccurred)
    }

    fn view(&self) -> Element<Self::Message, TuiRenderer> {
        Container::new(
            Row::new()
                .spacing(2)
                .width(Length::Shrink)
                .push(match &self.board {
                    Some(board) => board.view(),
                    None => Text::new("Empty board").into(),
                })
                .push(
                    Column::new()
                        .spacing(1)
                        .push(Row::new().push(Text::new("Next move: ")).push(
                            if self.gtp_status == GtpStatus::Loading {
                                Text::new("Loading").width(Length::Units(7)).font(
                                    Style::default()
                                        .bg(self.theme.loading_label_bg)
                                        .fg(self.theme.loading_label_fg),
                                )
                            } else {
                                Text::new(self.next_move_input.clone()).width(Length::Units(7))
                            },
                        ))
                        .try_push(self.gtp_error.clone().map(|message| {
                            Container::new(Text::new(message))
                                .style(
                                    Style::default()
                                        .bg(self.theme.error_message_bg)
                                        .fg(self.theme.error_message_fg),
                                )
                                .width(Length::Units(18))
                        })),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            GameMessage::BoardLoaded(board) => {
                self.board = Some(board);
                Command::none()
            }
            GameMessage::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased {
                key_code,
                modifiers,
            })) => {
                if key_code == keyboard::KeyCode::C && modifiers.control {
                    // exit on ctrl+c (status 1 = error)
                    self.should_exit = Some(1);
                }

                if key_code == keyboard::KeyCode::Backspace && self.gtp_status == GtpStatus::Idle {
                    if !self.next_move_input.is_empty() {
                        self.next_move_input.pop();
                        self.refresh_highlight_coords();
                    }
                }

                if key_code == keyboard::KeyCode::Enter && self.gtp_status == GtpStatus::Idle {
                    if let Some(board) = &mut self.board {
                        if let Some(coords) = board.get_valid_highlight_coords() {
                            self.gtp_status = GtpStatus::Loading;
                            self.next_move_input = "".to_string();
                            self.refresh_highlight_coords();

                            return Command::perform(
                                GoGame::play_move(
                                    self.gtp_engine.clone(),
                                    coords,
                                    self.player_color,
                                ),
                                |result| match result {
                                    Ok((black_stones, white_stones)) => {
                                        GameMessage::AfterStonePlayed(black_stones, white_stones)
                                    }
                                    Err(app_error) => GameMessage::GtpError(app_error.message),
                                },
                            );
                        }
                    }
                }

                Command::none()
            }
            GameMessage::EventOccurred(Event::Keyboard(keyboard::Event::CharacterReceived(c))) => {
                if INPUT_CHAR_RANGE.contains(&c) && self.gtp_status == GtpStatus::Idle {
                    if self.next_move_input.is_empty() {
                        self.next_move_input.push(c);
                        self.refresh_highlight_coords();
                    }
                }

                if INPUT_NUMBER_RANGE.contains(&c) && self.gtp_status == GtpStatus::Idle {
                    if !self.next_move_input.is_empty() && self.next_move_input.len() < 3 {
                        self.next_move_input.push(c);
                        self.refresh_highlight_coords();
                    }
                }

                Command::none()
            }
            GameMessage::AfterStonePlayed(black_stones, white_stones) => {
                if let Some(board) = &mut self.board {
                    board.set_stones(black_stones, white_stones);
                }

                Command::perform(
                    GoGame::gen_next_move(self.gtp_engine.clone(), self.player_color),
                    |result| match result {
                        Ok((black_stones, white_stones)) => {
                            GameMessage::AfterGenMove(black_stones, white_stones)
                        }
                        Err(app_error) => GameMessage::GtpError(app_error.message),
                    },
                )
            }
            GameMessage::AfterGenMove(black_stones, white_stones) => {
                if let Some(board) = &mut self.board {
                    board.set_stones(black_stones, white_stones);
                }
                self.gtp_status = GtpStatus::Idle;
                Command::none()
            }
            GameMessage::GtpError(message) => {
                self.gtp_error = Some(message);
                self.gtp_status = GtpStatus::Error;
                Command::none()
            }
            GameMessage::EventOccurred(_) => Command::none(),
        }
    }
}

impl GoGame {
    async fn load_board(gtp_engine: Arc<Mutex<Engine>>, theme: Theme) -> Board {
        let board_size = gtp_engine.lock().unwrap().query_board_size().unwrap();
        Board::new(board_size, theme)
    }

    async fn play_move(
        gtp_engine: Arc<Mutex<Engine>>,
        coords: Coords,
        color: StoneColor,
    ) -> Result<(Vec<Stone>, Vec<Stone>), AppError> {
        let mut gtp_engine = gtp_engine.lock().unwrap();
        gtp_engine.play(color, coords)?;

        let black_stones = gtp_engine.list_stones(StoneColor::Black)?;
        let white_stones = gtp_engine.list_stones(StoneColor::White)?;

        Ok((black_stones, white_stones))
    }

    async fn gen_next_move(
        gtp_engine: Arc<Mutex<Engine>>,
        player_color: StoneColor,
    ) -> Result<(Vec<Stone>, Vec<Stone>), AppError> {
        let mut gtp_engine = gtp_engine.lock().unwrap();
        gtp_engine.gen_move(player_color.inverse())?;

        let black_stones = gtp_engine.list_stones(StoneColor::Black)?;
        let white_stones = gtp_engine.list_stones(StoneColor::White)?;

        Ok((black_stones, white_stones))
    }

    fn refresh_highlight_coords(&mut self) {
        if let Some(board) = &mut self.board {
            board.highlight_coords(parse_input_coords(self.next_move_input.clone()));
        }
    }
}
