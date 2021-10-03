use super::theme::Theme;
use crate::core::engine::Engine;
use crate::core::errors::AppError;
use crate::core::helpers::get_column_name;
use tui::layout::Constraint;
use tui::style::Style;
use tui::widgets::{Cell, Row};

pub struct BoardTable<'a> {
    rows: Vec<BoardRow<'a>>,
    board_size: u8,
    number_column_size: u8,
    star_points: Vec<StarPoint>,
    white_stones: Vec<Stone>,
    black_stones: Vec<Stone>,
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

struct BoardRow<'a> {
    cells: Vec<BoardCell<'a>>,
}

impl<'a> BoardRow<'a> {
    fn from(cells: Vec<BoardCell<'a>>) -> Self {
        BoardRow { cells }
    }
}

impl<'a> BoardTable<'a> {
    pub fn from_engine(engine: &mut Engine, theme: &Theme) -> Result<Self, AppError> {
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

    pub fn get_tui_rows(&mut self) -> Vec<Row> {
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

    pub fn get_tui_widths(&mut self) -> Vec<Constraint> {
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
