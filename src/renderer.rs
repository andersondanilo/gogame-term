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
use tui::widgets::{Block, Borders, Cell, Row, Table};
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
    pub intersection_star_char: String,
    pub intersection_star_color: Color,
    pub white_stone_char: String,
    pub black_stone_char: String,
    pub white_stone_color: Color,
    pub black_stone_color: Color,
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
            intersection_star_char: "╋".to_string(),
            intersection_star_color: parse_color("#7d6c4b").unwrap(),
            intersection_horiz_char: "─".to_string(),
            intersection_color: parse_color("#7d6c4b").unwrap(),
            white_stone_char: "●".to_string(),
            black_stone_char: "●".to_string(),
            white_stone_color: parse_color("#FFFFFF").unwrap(),
            black_stone_color: parse_color("#000000").unwrap(),
        }
    }
}

struct BoardTable<'a> {
    rows: Vec<BoardRow<'a>>,
    board_size: u8,
    number_column_size: u8,
    star_points: Vec<StarPoint>,
    white_stones: Vec<Stone>,
    black_stones: Vec<Stone>,
}

#[derive(Debug, Copy, Clone)]
struct StarPoint {
    row: u8,
    col: u8,
}

impl StarPoint {
    fn from(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

impl<'a> BoardTable<'a> {
    fn from_engine(engine: &mut Engine, theme: &Theme) -> Result<Self, AppError> {
        let board_size = engine.query_boardsize()?;
        let number_column_size = 3;

        let default_style = Style::default()
            .bg(theme.board_bg_color)
            .fg(theme.text_fg_color);
        let header_style = default_style.clone().add_modifier(theme.header_text_style);
        let intersection_style = default_style.clone().fg(theme.intersection_color);
        let star_points = Self::make_star_points(board_size);

        let mut white_stones = vec![
            Stone {
                color: StoneColor::White,
                row: 2,
                col: 2,
            },
            Stone {
                color: StoneColor::White,
                row: 2,
                col: 3,
            },
        ];
        let mut black_stones = vec![
            Stone {
                color: StoneColor::Black,
                row: 12,
                col: 8,
            },
            Stone {
                color: StoneColor::Black,
                row: 13,
                col: 8,
            },
        ];

        let mut rows: Vec<BoardRow> = vec![Self::make_header_row(board_size, header_style)];

        for line in (1..=board_size).rev() {
            let line_stones = white_stones
                .iter()
                .chain(black_stones.iter())
                .filter(|stone| stone.row == line)
                .copied()
                .collect::<Vec<Stone>>();

            let line_star_points = star_points
                .iter()
                .filter(|stone| stone.row == line)
                .copied()
                .collect::<Vec<StarPoint>>();

            rows.push(Self::make_line_row(
                board_size,
                header_style,
                theme,
                intersection_style,
                line,
                line_stones,
                line_star_points,
            ));
        }

        rows.push(Self::make_header_row(board_size, header_style));

        Ok(BoardTable {
            rows,
            board_size,
            number_column_size,
            white_stones,
            black_stones,
            star_points,
        })
    }

    fn make_star_points(board_size: u8) -> Vec<StarPoint> {
        let margin = if board_size >= 13 { 4u8 } else { 3u8 };
        let middle = board_size / 2;

        let mut points: Vec<StarPoint> = vec![
            StarPoint::from(margin, margin),                  // top left
            StarPoint::from(margin, board_size - margin + 1), // top right
            StarPoint::from(board_size - margin + 1, margin), // bottom left
            StarPoint::from(board_size - margin + 1, board_size - margin + 1), // bottom right
            StarPoint::from(middle, middle),                  // middle point
        ];

        if board_size >= 19 {
            points.push(StarPoint::from(margin, middle)); // top horiz middle
            points.push(StarPoint::from(board_size - margin + 1, middle)); // bottom horiz middle
            points.push(StarPoint::from(middle, margin)); // left vertical middle
            points.push(StarPoint::from(middle, board_size - margin + 1)); // right vertical middle
        }

        points
    }

    fn make_header_row(board_size: u8, header_style: Style) -> BoardRow<'a> {
        let mut header_cells: Vec<BoardCell> = vec![BoardCell::spacing()];
        for column in 1..=board_size {
            header_cells.push(BoardCell::spacing());
            header_cells.push(BoardCell::from(
                Cell::from(format!("{}", get_column_name(column))).style(header_style),
                None,
            ));
        }
        header_cells.push(BoardCell::spacing()); // spacing cell
        header_cells.push(BoardCell::spacing()); // number column

        BoardRow::from(header_cells)
    }

    fn make_line_row(
        board_size: u8,
        header_style: Style,
        theme: &Theme,
        default_intersection_style: Style,
        line_nr: u8,
        mut line_stones: Vec<Stone>,
        mut line_star_points: Vec<StarPoint>,
    ) -> BoardRow<'a> {
        let mut board_line: Vec<BoardCell> = vec![BoardCell::from(
            Cell::from(format!("{: >3}", line_nr)).style(header_style),
            None,
        )];

        line_stones.sort_by(|a, b| b.col.cmp(&a.col));
        line_star_points.sort_by(|a, b| b.col.cmp(&a.col));

        let mut line_stones_next: Option<Stone> = line_stones.pop();
        let mut line_star_next: Option<StarPoint> = line_star_points.pop();

        let white_stone_style = default_intersection_style.fg(theme.white_stone_color);
        let black_stone_style = default_intersection_style.fg(theme.black_stone_color);

        for column in 1..=board_size {
            if column > 1 {
                board_line.push(BoardCell::from(
                    Cell::from(theme.intersection_horiz_char.clone())
                        .style(default_intersection_style),
                    None,
                ));
            } else {
                board_line.push(BoardCell::spacing());
            }

            let stone: Option<Stone> = match line_stones_next {
                Some(s) => {
                    if s.col == column {
                        line_stones_next = line_stones.pop();
                        Some(s)
                    } else {
                        None
                    }
                }
                None => None,
            };

            let is_star_point: bool = match line_star_next {
                Some(s) => {
                    if s.col == column {
                        line_star_next = line_star_points.pop();
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };

            let (intersection_char, intersection_style) = if is_star_point {
                (
                    theme.intersection_star_char.clone(),
                    default_intersection_style.fg(theme.intersection_star_color),
                )
            } else {
                (theme.intersection_char.clone(), default_intersection_style)
            };

            let intersection_cell = match stone {
                Some(s) => match s.color {
                    StoneColor::White => {
                        Cell::from(theme.white_stone_char.clone()).style(white_stone_style)
                    }
                    StoneColor::Black => {
                        Cell::from(theme.black_stone_char.clone()).style(black_stone_style)
                    }
                },
                None => Cell::from(intersection_char).style(intersection_style),
            };

            board_line.push(BoardCell::from(intersection_cell, stone));
        }

        board_line.push(BoardCell::spacing()); // spacing cell
        board_line.push(BoardCell::from(
            Cell::from(format!("{: <3}", line_nr)).style(header_style),
            None,
        )); // number column

        BoardRow::from(board_line)
    }

    fn get_tui_rows(&mut self) -> Vec<Row> {
        return self
            .rows
            .iter()
            .map(|board_row| {
                Row::new(
                    board_row
                        .cells
                        .iter()
                        .map(|board_cell| board_cell.cell.clone())
                        .collect::<Vec<Cell>>(),
                )
            })
            .collect::<Vec<Row>>();
    }

    fn get_tui_widths(&mut self) -> Vec<Constraint> {
        let mut widths: Vec<Constraint> = vec![Constraint::Length(self.number_column_size as u16)];

        for _ in 1..=self.board_size {
            widths.push(Constraint::Length(1)); // padding
            widths.push(Constraint::Length(1)); // column
        }
        widths.push(Constraint::Length(1)); // spacing cell
        widths.push(Constraint::Length(self.number_column_size as u16)); // number_column_size

        widths
    }
}

struct BoardRow<'a> {
    cells: Vec<BoardCell<'a>>,
}

impl<'a> BoardRow<'a> {
    fn from(cells: Vec<BoardCell<'a>>) -> Self {
        BoardRow { cells }
    }
}

struct BoardCell<'a> {
    cell: Cell<'a>,
    stone: Option<Stone>,
}

impl<'a> BoardCell<'a> {
    fn from(cell: Cell<'a>, stone: Option<Stone>) -> Self {
        BoardCell { cell, stone: None }
    }

    fn spacing() -> Self {
        BoardCell {
            cell: Cell::from(""),
            stone: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum StoneColor {
    White,
    Black,
}

#[derive(Debug, Copy, Clone)]
struct Stone {
    color: StoneColor,
    row: u8,
    col: u8,
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
    let board_canvas_height_size = (board_size as u16) + 2;

    let mut board_table = BoardTable::from_engine(engine, &theme)?;

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

                let tui_widths = board_table.get_tui_widths();

                let t = Table::new(board_table.get_tui_rows())
                    .block(Block::default().style(board_default_style))
                    .column_spacing(0)
                    .widths(&tui_widths);

                f.render_widget(t, chunks[0]);
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
