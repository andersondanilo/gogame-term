use crate::core::helpers::parse_color;
use iced_native::Color;
use iced_tui::Style;

#[derive(Debug, Clone)]
pub struct Theme {
    pub board_bg_color: Color,
    pub board_bg_hl_color: Color,
    pub text_fg_color: Color,
    pub header_text_style: Style,
    pub intersection_char: String,
    pub intersection_star_char: String,
    pub intersection_star_color: Color,
    pub white_stone_char: String,
    pub black_stone_char: String,
    pub white_stone_color: Color,
    pub black_stone_color: Color,
    pub intersection_horiz_char: String,
    pub intersection_color: Color,
    pub error_message_fg: Color,
    pub error_message_bg: Color,
    pub loading_label_fg: Color,
    pub loading_label_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            board_bg_color: parse_color("#af9769").unwrap(),
            board_bg_hl_color: parse_color("#E3C388").unwrap(),
            text_fg_color: parse_color("#1c1f25").unwrap(),
            header_text_style: Style::default().bold(),
            intersection_char: "┼".to_string(),
            intersection_star_char: "╋".to_string(),
            intersection_star_color: parse_color("#7d6c4b").unwrap(),
            intersection_horiz_char: "─".to_string(),
            intersection_color: parse_color("#7d6c4b").unwrap(),
            white_stone_char: "●".to_string(),
            black_stone_char: "●".to_string(),
            white_stone_color: parse_color("#FFFFFF").unwrap(),
            black_stone_color: parse_color("#000000").unwrap(),
            error_message_fg: parse_color("#FFFFFF").unwrap(),
            error_message_bg: parse_color("#FF0000").unwrap(),
            loading_label_fg: parse_color("#FFFFFF").unwrap(),
            loading_label_bg: parse_color("#00FF00").unwrap(),
        }
    }
}
