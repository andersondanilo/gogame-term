use super::errors::AppError;
use read_color::rgb;
use tui::style::Color;

pub fn get_column_name(col: u8) -> char {
    // skip I column
    let add = if col >= 9 { 1 } else { 0 };
    (64u8 + col + add) as char
}

pub fn get_column_number(col: char) -> u8 {
    let col_nr = col as u8 - 64u8;
    let remove = if col_nr >= 9 { 1 } else { 0 };
    col_nr - remove
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

#[cfg(test)]
mod tests {
    use super::*;
    use gtp::Entity;

    #[test]
    fn correct_column_name_number_mapping() {
        let char_table = [
            ('A', 1),
            ('B', 2),
            ('C', 3),
            ('D', 4),
            ('E', 5),
            ('F', 6),
            ('G', 7),
            ('H', 8),
            ('J', 9),
            ('K', 10),
            ('L', 11),
            ('M', 12),
            ('N', 13),
            ('O', 14),
            ('P', 15),
            ('Q', 16),
            ('R', 17),
            ('S', 18),
            ('T', 19),
        ];

        for (char_name, char_nr) in char_table {
            assert_eq!(char_name, get_column_name(char_nr));
            assert_eq!(char_nr, get_column_number(char_name));
            assert_eq!(
                format!("{}1", char_name),
                Entity::Vertex((char_nr.into(), 1)).to_string()
            );
        }
    }
}
