use crate::core::entities::{Coords, OptCoords, Stone, StoneColor};
use crate::core::helpers::get_column_name;
use crate::core::theme::Theme;
use crate::gogame::game_message::GameMessage;
use iced_native::{Column, Container, Element, Row, Text};
use iced_tui::{Style, TuiRenderer};

#[derive(Debug, Clone)]
pub struct Board {
    board_size: u8,
    number_column_size: u8,
    theme: Theme,
    star_points: Vec<Coords>,
    white_stones: Vec<Stone>,
    black_stones: Vec<Stone>,
    highlight_coords: OptCoords,
}

impl Board {
    pub fn highlight_coords(&mut self, coords: OptCoords) {
        self.highlight_coords = coords;
    }

    pub fn set_stones(&mut self, black_stones: Vec<Stone>, white_stones: Vec<Stone>) {
        self.black_stones = black_stones;
        self.white_stones = white_stones;
    }

    pub fn get_valid_highlight_coords(&mut self) -> Option<Coords> {
        match self.highlight_coords {
            OptCoords {
                row: Some(row),
                col: Some(col),
            } => Some(Coords { row, col }),
            _ => None,
        }
    }
}

impl Board {
    pub fn new(board_size: u8, theme: Theme) -> Self {
        Board {
            board_size,
            number_column_size: 2,
            theme,
            star_points: gen_star_points(board_size),
            white_stones: vec![],
            black_stones: vec![],
            highlight_coords: OptCoords::default(),
        }
    }

    pub fn view(&self) -> Element<GameMessage, TuiRenderer> {
        let header_style = Style::default().bold();
        let mut column = Column::new();
        column = self.add_header_line(column, header_style);

        for line_number in 1..=self.board_size {
            let mut line_stones = self
                .white_stones
                .iter()
                .chain(self.black_stones.iter())
                .filter(|stone| stone.row == line_number)
                .copied()
                .collect::<Vec<Stone>>();

            let mut line_star_points = self
                .star_points
                .iter()
                .filter(|stone| stone.row == line_number)
                .copied()
                .collect::<Vec<Coords>>();

            line_stones.sort_by(|a, b| b.col.cmp(&a.col));
            line_star_points.sort_by(|a, b| b.col.cmp(&a.col));

            let mut line_stones_next: Option<Stone> = line_stones.pop();
            let mut line_star_next: Option<Coords> = line_star_points.pop();

            let line_focused = match self.highlight_coords.row {
                Some(row) => row == line_number,
                None => false,
            };

            let mut line_style = Style::default();

            if line_focused {
                line_style = line_style.bg(self.theme.board_bg_hl_color);
            }

            let mut board_line = Row::new().push(Text::new(" ")).push(
                Text::new(format!("{: >2} ", line_number)).font(line_style.merge(&header_style)),
            );

            for column_number in 1..=self.board_size {
                let stone: Option<Stone> = match line_stones_next {
                    Some(s) => {
                        if s.col == column_number {
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
                        if s.col == column_number {
                            line_star_next = line_star_points.pop();
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                };

                let column_focused = match self.highlight_coords.col {
                    Some(col) => col == column_number,
                    None => false,
                };

                let mut cell_style = Style::default().fg(self.theme.intersection_color);

                if line_focused || column_focused {
                    cell_style = cell_style.bg(self.theme.board_bg_hl_color);
                }

                if let Some(stone) = stone {
                    cell_style = match stone.color {
                        StoneColor::Black => cell_style.fg(self.theme.black_stone_color),
                        StoneColor::White => cell_style.fg(self.theme.white_stone_color),
                    };
                } else if is_star_point {
                    cell_style = cell_style.fg(self.theme.intersection_star_color);
                }

                board_line = board_line.push(
                    Text::new(match stone {
                        Some(stone) => match stone.color {
                            StoneColor::Black => self.theme.black_stone_char.clone(),
                            StoneColor::White => self.theme.white_stone_char.clone(),
                        },
                        None => {
                            if is_star_point {
                                self.theme.intersection_star_char.clone()
                            } else {
                                self.theme.intersection_char.clone()
                            }
                        }
                    })
                    .font(cell_style),
                );

                if column_number < self.board_size {
                    board_line = board_line.push(
                        Text::new(self.theme.intersection_horiz_char.clone())
                            .font(line_style.fg(self.theme.intersection_color)),
                    )
                }
            }

            board_line =
                board_line.push(Text::new(format!(" {: <2}", line_number)).font(header_style));

            column = column.push(board_line);
        }

        column = self.add_header_line(column, header_style);

        Container::new(column)
            .style(
                Style::default()
                    .bg(self.theme.board_bg_color)
                    .fg(self.theme.text_fg_color),
            )
            .into()
    }

    fn add_header_line<'a>(
        &self,
        column: Column<'a, GameMessage, TuiRenderer>,
        header_style: Style,
    ) -> Column<'a, GameMessage, TuiRenderer> {
        let line_column_space_left = "    ";
        let line_column_space_right = "   ";

        let mut header_line = Row::new().push(Text::new(line_column_space_left));
        for column_number in 1..=self.board_size {
            let mut style = header_style;

            if let Some(hl_col) = self.highlight_coords.col {
                if column_number == hl_col {
                    style = style.bg(self.theme.board_bg_hl_color);
                }
            }

            header_line = header_line
                .push(Text::new(get_column_name(column_number)).font(style))
                .push(Text::new(" "));
        }
        header_line = header_line.push(Text::new(line_column_space_right).font(header_style));

        column.push(header_line)
    }
}

fn gen_star_points(board_size: u8) -> Vec<Coords> {
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
