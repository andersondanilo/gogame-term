use crate::core::engine::Engine;
use crate::core::errors::AppError;
use crate::core::helpers::{get_column_name, parse_color};
use log::info;
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Map, MapResolution, Rectangle};
use tui::widgets::{Block, Borders};
use tui::Terminal;

use std::sync::mpsc;
use std::{io, thread};

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if let Err(err) = tx.send(Event::Tick) {
                    eprintln!("{}", err);
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

pub struct Theme {
    pub board_bg_color: Color,
    pub text_fg_color: Color,
    pub header_text_style: Modifier,
    pub intersection_char: String,
    pub intersection_horiz_char: String,
    pub intersection_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            board_bg_color: parse_color("#af9769").unwrap(),
            text_fg_color: parse_color("#1c1f25").unwrap(),
            header_text_style: Modifier::BOLD,
            intersection_char: "┼".to_string(),
            intersection_horiz_char: "─".to_string(),
            intersection_color: parse_color("#7d6c4b").unwrap(),
        }
    }
}

pub fn render_app(engine: &mut Engine) -> Result<(), AppError> {
    let stdout = std::io::stdout().into_raw_mode().map_err(|_| AppError {
        message: "Can't get stdout".to_string(),
    })?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| AppError {
        message: "Can't get terminal".to_string(),
    })?;

    let board_size = engine.query_boardsize()?;
    let theme = Theme::default();

    let board_canvas_width_size = 4 + (((board_size as u16) * 2) - 1) + 4;
    let board_canvas_height_size = 2 + (board_size as u16) + 2;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Default::default()
    };
    let events = Events::with_config(config);

    let board_default_style = Style::default()
        .bg(theme.board_bg_color)
        .fg(theme.text_fg_color);

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(board_canvas_width_size),
                            Constraint::Max(50),
                        ]
                        .as_ref(),
                    )
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: board_canvas_width_size + 50,
                        height: board_canvas_height_size,
                    });

                let board_canvas = Canvas::default()
                    .block(Block::default().style(board_default_style))
                    .background_color(theme.board_bg_color)
                    .paint(|ctx| {
                        ctx.print(0_f64, 0_f64, "A", Color::Yellow);
                        ctx.print(
                            board_canvas_width_size as f64,
                            board_canvas_height_size as f64,
                            "A",
                            Color::Red,
                        );
                    })
                    .x_bounds([0_f64, board_canvas_width_size as f64])
                    .y_bounds([0_f64, board_canvas_height_size as f64]);

                f.render_widget(board_canvas, chunks[0]);
            })
            .map_err(|_| AppError {
                message: "Can't draw terminal".to_string(),
            })?;

        match events.next().map_err(|_| AppError {
            message: "Can't get next event".to_string(),
        })? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }

                _ => {}
            },
            Event::Tick => {
                // app.update();
            }
        }
    }

    Ok(())
}

// pub fn render_board(engine: &mut Engine) -> Result<(), AppError> {
//     let board_size = engine.query_boardsize()?;
//     let theme = Theme::default();
//
//     info!("rendering board {}x{}", board_size, board_size);
//
//     // HEADER
//     render_column_padding_cell(&theme);
//     render_line_column_cell("", false, &theme);
//     for column in 1..=board_size {
//         render_column_padding_cell(&theme);
//         render_column_header_cell(&format!("{}", get_column_name(column)), &theme);
//     }
//     render_column_padding_cell(&theme);
//     render_line_column_cell("", false, &theme);
//     render_column_padding_cell(&theme);
//     render_newline();
//
//     for line in (1..=board_size).rev() {
//         render_column_padding_cell(&theme);
//         render_line_column_cell(&format!("{}", &line), false, &theme);
//         for column in 1..=board_size {
//             render_intersection_horiz_cell(&theme);
//             render_intersection_cell(&theme);
//         }
//         render_column_padding_cell(&theme);
//         render_line_column_cell(&format!("{}", &line), true, &theme);
//         render_column_padding_cell(&theme);
//         render_newline();
//     }
//
//     // FOOTER
//     render_column_padding_cell(&theme);
//     render_line_column_cell("", false, &theme);
//     for column in 1..=board_size {
//         render_column_padding_cell(&theme);
//         render_column_header_cell(&format!("{}", get_column_name(column)), &theme);
//     }
//     render_column_padding_cell(&theme);
//     render_line_column_cell("", false, &theme);
//     render_column_padding_cell(&theme);
//     render_newline();
//
//     Ok(())
// }
//
// pub fn render_column_header_cell(column_name: &str, theme: &Theme) {
//     render_cell(
//         column_name,
//         Some(theme.text_fg_color),
//         Some(theme.board_bg_color),
//         &theme.header_text_style,
//     );
// }
//
// pub fn render_intersection_cell(theme: &Theme) {
//     render_cell(
//         &theme.intersection_char,
//         Some(theme.intersection_color),
//         Some(theme.board_bg_color),
//         &[],
//     );
// }
//
// pub fn render_intersection_horiz_cell(theme: &Theme) {
//     render_cell(
//         &theme.intersection_horiz_char,
//         Some(theme.intersection_color),
//         Some(theme.board_bg_color),
//         &[],
//     );
// }
//
// pub fn render_column_padding_cell(theme: &Theme) {
//     render_cell(
//         " ",
//         Some(theme.text_fg_color),
//         Some(theme.board_bg_color),
//         &[],
//     );
// }
//
// pub fn render_line_column_cell(str_num: &str, is_right_column: bool, theme: &Theme) {
//     if is_right_column {
//         render_cell(
//             &format!("{: <2}", str_num),
//             Some(theme.text_fg_color),
//             Some(theme.board_bg_color),
//             &theme.header_text_style,
//         );
//     } else {
//         render_cell(
//             &format!("{: >2}", str_num),
//             Some(theme.text_fg_color),
//             Some(theme.board_bg_color),
//             &theme.header_text_style,
//         );
//     }
// }
//
// pub fn render_cell(
//     str_value: &str,
//     fg_color: Option<Color>,
//     bg_color: Option<Color>,
//     styles: &[Styles],
// ) {
//     let mut colored_str_value: ColoredString = str_value.into();
//
//     if let Some(color) = fg_color {
//         colored_str_value = colored_str_value.color(color);
//     }
//
//     if let Some(color) = bg_color {
//         colored_str_value = colored_str_value.on_color(color);
//     }
//
//     for style in styles {
//         colored_str_value = match &style {
//             Styles::Bold => colored_str_value.bold(),
//             Styles::Dimmed => colored_str_value.dimmed(),
//             Styles::Italic => colored_str_value.italic(),
//             Styles::Underline => colored_str_value.underline(),
//             Styles::Blink => colored_str_value.blink(),
//             Styles::Reversed => colored_str_value.reversed(),
//             Styles::Hidden => colored_str_value.hidden(),
//             Styles::Strikethrough => colored_str_value.strikethrough(),
//             Styles::Clear => colored_str_value.clear(),
//         };
//     }
//
//     print!("{}", &colored_str_value);
// }
//
// pub fn render_newline() {
//     println!("");
// }
