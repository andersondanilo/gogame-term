use super::theme::Theme;
use crate::core::entities::{Coords, OptCoords, Stone, StoneColor};
use crate::core::helpers::get_column_name;
use actix::prelude::*;
use std::sync::Arc;
use tui::layout::Constraint;
use tui::style::Style;
use tui::widgets::{Cell, Row};

pub struct BoardTableActor {
    rows: Vec<BoardRow>,
    board_size: u8,
    number_column_size: u8,
    theme: Theme,
    star_points: Vec<Coords>,
    white_stones: Vec<Stone>,
    black_stones: Vec<Stone>,
    highlight_coords: Arc<OptCoords>,
    refresh_ui_request: bool,
    next_move_input: String,
}

impl Actor for BoardTableActor {
    type Context = Context<Self>;
}

struct BoardCell {
    cell_text: String,
    cell_style: Option<Style>,
    stone: Option<Stone>,
}

impl BoardCell {
    fn new(cell_text: String, cell_style: Option<Style>, stone: Option<Stone>) -> Self {
        BoardCell {
            cell_text,
            cell_style,
            stone,
        }
    }

    fn from(cell_text: String, stone: Option<Stone>) -> Self {
        BoardCell::new(cell_text, None, stone)
    }

    fn style(&mut self, style: Style) {
        self.cell_style = Some(style);
    }

    fn spacing() -> Self {
        Self::from("".to_string(), None)
    }

    fn styled_spacing(style: Style) -> Self {
        BoardCell {
            cell_text: "".to_string(),
            cell_style: Some(style),
            stone: None,
        }
    }
}

struct BoardRow {
    cells: Vec<BoardCell>,
}

impl BoardRow {
    fn from(cells: Vec<BoardCell>) -> Self {
        BoardRow { cells }
    }
}

pub struct GetHighlightCoordsMessage {}

impl Message for GetHighlightCoordsMessage {
    type Result = Arc<OptCoords>;
}

impl Handler<GetHighlightCoordsMessage> for BoardTableActor {
    type Result = Arc<OptCoords>;

    fn handle(&mut self, _: GetHighlightCoordsMessage, _: &mut Context<Self>) -> Self::Result {
        self.highlight_coords.clone()
    }
}

pub struct GetNextMoveInput {}

impl Message for GetNextMoveInput {
    type Result = String;
}

impl Handler<GetNextMoveInput> for BoardTableActor {
    type Result = String;

    fn handle(&mut self, _: GetNextMoveInput, _: &mut Context<Self>) -> Self::Result {
        self.next_move_input.clone()
    }
}

pub struct SetNextMoveInput {
    pub text: String,
}

impl Message for SetNextMoveInput {
    type Result = ();
}

impl Handler<SetNextMoveInput> for BoardTableActor {
    type Result = ();

    fn handle(&mut self, msg: SetNextMoveInput, _: &mut Context<Self>) -> Self::Result {
        self.next_move_input = msg.text
    }
}

pub struct GetBoardSizeMessage {}

impl Message for GetBoardSizeMessage {
    type Result = u8;
}

impl Handler<GetBoardSizeMessage> for BoardTableActor {
    type Result = u8;

    fn handle(&mut self, _: GetBoardSizeMessage, _: &mut Context<Self>) -> Self::Result {
        self.board_size
    }
}

pub struct SetHightlightCoordsMessage {
    pub coords: OptCoords,
}

impl Message for SetHightlightCoordsMessage {
    type Result = ();
}

impl Handler<SetHightlightCoordsMessage> for BoardTableActor {
    type Result = ();

    fn handle(&mut self, msg: SetHightlightCoordsMessage, _: &mut Context<Self>) -> Self::Result {
        self.highlight_coords = Arc::new(msg.coords);
        self.refresh_ui_request = true;
    }
}

pub struct SetWhiteStonesMessage {
    pub stones: Vec<Stone>,
}

impl Message for SetWhiteStonesMessage {
    type Result = ();
}

impl Handler<SetWhiteStonesMessage> for BoardTableActor {
    type Result = ();

    fn handle(&mut self, msg: SetWhiteStonesMessage, _: &mut Context<Self>) -> Self::Result {
        self.white_stones = msg.stones;
        self.refresh_ui_request = true;
    }
}

pub struct SetBlackStonesMessage {
    pub stones: Vec<Stone>,
}

impl Message for SetBlackStonesMessage {
    type Result = ();
}

impl Handler<SetBlackStonesMessage> for BoardTableActor {
    type Result = ();

    fn handle(&mut self, msg: SetBlackStonesMessage, _: &mut Context<Self>) -> Self::Result {
        self.black_stones = msg.stones;
        self.refresh_ui_request = true;
    }
}

pub struct GetTuiRowsMessage {}

impl Message for GetTuiRowsMessage {
    type Result = Vec<Row<'static>>;
}

impl Handler<GetTuiRowsMessage> for BoardTableActor {
    type Result = Vec<Row<'static>>;

    fn handle(&mut self, _: GetTuiRowsMessage, _: &mut Context<Self>) -> Self::Result {
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
                        .map(|board_cell| {
                            Cell::from(board_cell.cell_text.clone())
                                .style(board_cell.cell_style.unwrap_or(Style::default()))
                        })
                        .collect::<Vec<Cell>>(),
                )
            })
            .collect::<Vec<Row>>();
    }
}

pub struct GetTuiWidthsMessage {}

impl Message for GetTuiWidthsMessage {
    type Result = Vec<Constraint>;
}

impl Handler<GetTuiWidthsMessage> for BoardTableActor {
    type Result = Vec<Constraint>;

    fn handle(&mut self, _: GetTuiWidthsMessage, _: &mut Context<Self>) -> Self::Result {
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

impl BoardTableActor {
    pub fn new(board_size: u8, theme: &Theme) -> Self {
        let number_column_size = 3;

        let star_points = Self::make_star_points(board_size);

        let mut table = BoardTableActor {
            rows: vec![],
            board_size,
            number_column_size,
            white_stones: vec![],
            black_stones: vec![],
            star_points,
            theme: theme.clone(),
            highlight_coords: Arc::new(OptCoords {
                col: None,
                row: None,
            }),
            refresh_ui_request: false,
            next_move_input: "".to_string(),
        };

        table.refresh_ui_rows();

        table
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

    fn make_header_row(&self, default_header_style: Style) -> BoardRow {
        let mut header_cells: Vec<BoardCell> = vec![BoardCell::spacing()];

        for column in 1..=self.board_size {
            let header_style = if self.is_column_highlighted(column) {
                default_header_style.bg(self.theme.board_bg_hl_color)
            } else {
                default_header_style
            };

            header_cells.push(BoardCell::spacing());
            header_cells.push(BoardCell::new(
                format!("{}", get_column_name(column)),
                Some(header_style),
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
    ) -> BoardRow {
        let (default_intersection_style, header_style) = if self.is_row_highlighted(line_nr) {
            (
                default_intersection_style.bg(self.theme.board_bg_hl_color),
                header_style.bg(self.theme.board_bg_hl_color),
            )
        } else {
            (default_intersection_style, header_style)
        };

        let mut board_line: Vec<BoardCell> = vec![BoardCell::new(
            format!("{: >3}", line_nr),
            Some(header_style),
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
                board_line.push(BoardCell::new(
                    self.theme.intersection_horiz_char.clone(),
                    Some(default_intersection_style),
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

            let (cell_text, cell_style) = match stone {
                Some(s) => match s.color {
                    StoneColor::White => {
                        (self.theme.white_stone_char.clone(), Some(selected_style))
                    }
                    StoneColor::Black => {
                        (self.theme.black_stone_char.clone(), Some(selected_style))
                    }
                },
                None => (intersection_char, Some(selected_style)),
            };

            board_line.push(BoardCell::new(cell_text, cell_style, stone));
        }

        board_line.push(BoardCell::styled_spacing(default_intersection_style)); // spacing cell
        board_line.push(BoardCell::new(
            format!("{: <3}", line_nr),
            Some(header_style),
            None,
        )); // number column

        BoardRow::from(board_line)
    }
}
