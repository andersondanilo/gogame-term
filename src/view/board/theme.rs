use crate::core::helpers::parse_color;
use tui::style::{Color, Modifier};

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
