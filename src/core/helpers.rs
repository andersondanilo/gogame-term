use super::errors::AppError;
use read_color::rgb;
use tui::style::Color;

pub fn get_column_name(col: u8) -> char {
    // skip I column
    let add = if col >= 9 { 1 } else { 0 };
    (64u8 + col + add) as char
}

pub fn get_column_number(col: char) -> u8 {
    return col as u8 - 64u8;
}

pub fn parse_color(text: &str) -> Result<Color, AppError> {
    if text.starts_with("#") {
        let mut chars = text.chars();
        chars.next();

        return match rgb(&mut chars) {
            Some([r, g, b]) => Ok(Color::Rgb(r, g, b)),
            None => Err(AppError {
                message: format!("Can't parse hex color {}", text),
            }),
        };
    }

    match text {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" => Ok(Color::Gray),
        "darkgray" => Ok(Color::DarkGray),
        "lightred" => Ok(Color::LightRed),
        "lightgreen" => Ok(Color::LightGreen),
        "lightyellow" => Ok(Color::LightYellow),
        "lightblue" => Ok(Color::LightBlue),
        "lightmagenta" => Ok(Color::LightMagenta),
        "lightcyan" => Ok(Color::LightCyan),
        _ => Err(AppError {
            message: format!("Can't parse named color {}", text),
        }),
    }
}
