use super::errors::AppError;
use colored::{Color, Colorize};
use read_color::rgb;

pub fn get_column_name(col: u8) -> char {
    // skip I column
    let add = if col >= 9 { 1 } else { 0 };
    (64u8 + col + add) as char
}

pub fn parse_color(text: &str) -> Result<Color, AppError> {
    if text.starts_with("#") {
        let mut chars = text.chars();
        chars.next();

        return match rgb(&mut chars) {
            Some([r, g, b]) => Ok(Color::TrueColor { r, g, b }),
            None => Err(AppError {
                message: format!("Can't parse hex color {}", text),
            }),
        };
    }

    text.parse().map_err(|_| AppError {
        message: format!("Can't parse named color {}", text),
    })
}
