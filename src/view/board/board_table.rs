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
    theme: Theme,
    star_points: Vec<Coords>,
    white_stones: Vec<Stone>,
    black_stones: Vec<Stone>,
    highlight_coords: OptCoords,
    refresh_ui_request: bool,
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

    fn styled_spacing(style: Style) -> Self {
        BoardCell {
            cell: Cell::from("").style(style),
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

pub struct OptCoords {
    pub row: Option<u8>,
    pub col: Option<u8>,
}

#[derive(Debug, Copy, Clone)]
pub struct Coords {
    pub row: u8,
    pub col: u8,
}

impl Coords {
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

        let mut table = BoardTable {
            rows: vec![],
            board_size,
            number_column_size,
            white_stones,
            black_stones,
            star_points,
            theme: theme.clone(),
            highlight_coords: OptCoords {
                col: None,
                row: None,
            },
            refresh_ui_request: false,
        };

        table.refresh_ui_rows();

        Ok(table)
    }

    pub fn change_highlight_coord(&mut self, highlight_coords: OptCoords) {
        self.highlight_coords = highlight_coords;
        self.refresh_ui_request = true;
    }

    fn refresh_ui_rows(&mut self) {
        let default_style = Style::default()
            .bg(self.theme.board_bg_color)
            .fg(self.theme.text_fg_color);

        let header_style = default_style
            .clone()
            .add_modifier(self.theme.header_text_style);
        let intersection_style = default_style.clone().fg(self.theme.intersection_color);

        let mut rows: Vec<BoardRow> = vec![self.make_header_row(header_style)];

        for line in (1..=self.board_size).rev() {
            let line_stones = self
                .white_stones
                .iter()
                .chain(self.black_stones.iter())
                .filter(|stone| stone.row == line)
                .copied()
                .collect::<Vec<Stone>>();

            let line_star_points = self
                .star_points
                .iter()
                .filter(|stone| stone.row == line)
                .copied()
                .collect::<Vec<Coords>>();

            rows.push(self.make_line_row(
                header_style,
                intersection_style,
                line,
                line_stones,
                line_star_points,
            ));
        }

        rows.push(self.make_header_row(header_style));

        self.rows = rows
    }

    fn make_star_points(board_size: u8) -> Vec<Coords> {
        let margin = if board_size >= 13 { 4u8 } else { 3u8 };
        let middle = board_size / 2;

        let mut points: Vec<Coords> = vec![
            Coords::from(margin, margin),                  // top left
            Coords::from(margin, board_size - margin + 1), // top right
            Coords::from(board_size - margin + 1, margin), // bottom left
            Coords::from(board_size - margin + 1, board_size - margin + 1), // bottom right
            Coords::from(middle, middle),                  // middle point
        ];

        if board_size >= 19 {
            points.push(Coords::from(margin, middle)); // top horiz middle
            points.push(Coords::from(board_size - margin + 1, middle)); // bottom horiz middle
            points.push(Coords::from(middle, margin)); // left vertical middle
            points.push(Coords::from(middle, board_size - margin + 1)); // right vertical middle
        }

        points
    }

    fn make_header_row(&self, default_header_style: Style) -> BoardRow<'a> {
        let mut header_cells: Vec<BoardCell> = vec![BoardCell::spacing()];

        for column in 1..=self.board_size {
            let header_style = if self.is_column_highlighted(column) {
                default_header_style.bg(self.theme.board_bg_hl_color)
            } else {
                default_header_style
            };

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

    fn is_column_highlighted(&self, column: u8) -> bool {
        matches!(self.highlight_coords.col, Some(c) if c == column)
    }

    fn is_row_highlighted(&self, row: u8) -> bool {
        matches!(self.highlight_coords.row, Some(r) if r == row)
    }

    fn make_line_row(
        &mut self,
        header_style: Style,
        default_intersection_style: Style,
        line_nr: u8,
        mut line_stones: Vec<Stone>,
        mut line_star_points: Vec<Coords>,
    ) -> BoardRow<'a> {
        let (default_intersection_style, header_style) = if self.is_row_highlighted(line_nr) {
            (
                default_intersection_style.bg(self.theme.board_bg_hl_color),
                header_style.bg(self.theme.board_bg_hl_color),
            )
        } else {
            (default_intersection_style, header_style)
        };

        let mut board_line: Vec<BoardCell> = vec![BoardCell::from(
            Cell::from(format!("{: >3}", line_nr)).style(header_style),
            None,
        )];

        line_stones.sort_by(|a, b| b.col.cmp(&a.col));
        line_star_points.sort_by(|a, b| b.col.cmp(&a.col));

        let mut line_stones_next: Option<Stone> = line_stones.pop();
        let mut line_star_next: Option<Coords> = line_star_points.pop();

        let white_stone_style = default_intersection_style.fg(self.theme.white_stone_color);
        let black_stone_style = default_intersection_style.fg(self.theme.black_stone_color);

        for column in 1..=self.board_size {
            if column > 1 {
                board_line.push(BoardCell::from(
                    Cell::from(self.theme.intersection_horiz_char.clone())
                        .style(default_intersection_style),
                    None,
                ));
            } else {
                board_line.push(BoardCell::styled_spacing(default_intersection_style));
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
                    self.theme.intersection_star_char.clone(),
                    default_intersection_style.fg(self.theme.intersection_star_color),
                )
            } else {
                (
                    self.theme.intersection_char.clone(),
                    default_intersection_style,
                )
            };

            let mut selected_style = match stone {
                Some(s) => match s.color {
                    StoneColor::White => white_stone_style,
                    StoneColor::Black => black_stone_style,
                },
                None => intersection_style,
            };

            if self.is_row_highlighted(line_nr) || self.is_column_highlighted(column) {
                selected_style = selected_style.bg(self.theme.board_bg_hl_color);
            }

            let intersection_cell = match stone {
                Some(s) => match s.color {
                    StoneColor::White => {
                        Cell::from(self.theme.white_stone_char.clone()).style(selected_style)
                    }
                    StoneColor::Black => {
                        Cell::from(self.theme.black_stone_char.clone()).style(selected_style)
                    }
                },
                None => Cell::from(intersection_char).style(selected_style),
            };

            board_line.push(BoardCell::from(intersection_cell, stone));
        }

        board_line.push(BoardCell::styled_spacing(default_intersection_style)); // spacing cell
        board_line.push(BoardCell::from(
            Cell::from(format!("{: <3}", line_nr)).style(header_style),
            None,
        )); // number column

        BoardRow::from(board_line)
    }

    pub fn get_tui_rows(&mut self) -> Vec<Row> {
        if self.refresh_ui_request {
            self.refresh_ui_rows();
            self.refresh_ui_request = false;
        }

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
