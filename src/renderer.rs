use crate::core::engine::Engine;
use crate::core::errors::AppError;
use crate::core::helpers::{get_column_name, parse_color};
use colored::ColoredString;
use colored::Styles;
use colored::{Color, Colorize};
use log::info;

pub struct Theme {
    pub board_bg_color: Color,
    pub text_fg_color: Color,
    pub header_text_style: Vec<Styles>,
    pub intersection_char: String,
    pub intersection_horiz_char: String,
    pub intersection_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            board_bg_color: parse_color("#af9769").unwrap(),
            text_fg_color: parse_color("#1c1f25").unwrap(),
            header_text_style: vec![Styles::Bold],
            intersection_char: "┼".to_string(),
            intersection_horiz_char: "─".to_string(),
            intersection_color: parse_color("#7d6c4b").unwrap(),
        }
    }
}

pub fn render_board(engine: &mut Engine) -> Result<(), AppError> {
    let board_size = engine.query_boardsize()?;
    let theme = Theme::default();

    info!("rendering board {}x{}", board_size, board_size);

    // HEADER
    render_column_padding_cell(&theme);
    render_line_column_cell("", false, &theme);
    for column in 1..=board_size {
        render_column_padding_cell(&theme);
        render_column_header_cell(&format!("{}", get_column_name(column)), &theme);
    }
    render_column_padding_cell(&theme);
    render_line_column_cell("", false, &theme);
    render_column_padding_cell(&theme);
    render_newline();

    for line in (1..=board_size).rev() {
        render_column_padding_cell(&theme);
        render_line_column_cell(&format!("{}", &line), false, &theme);
        for column in 1..=board_size {
            render_intersection_horiz_cell(&theme);
            render_intersection_cell(&theme);
        }
        render_column_padding_cell(&theme);
        render_line_column_cell(&format!("{}", &line), true, &theme);
        render_column_padding_cell(&theme);
        render_newline();
    }

    // FOOTER
    render_column_padding_cell(&theme);
    render_line_column_cell("", false, &theme);
    for column in 1..=board_size {
        render_column_padding_cell(&theme);
        render_column_header_cell(&format!("{}", get_column_name(column)), &theme);
    }
    render_column_padding_cell(&theme);
    render_line_column_cell("", false, &theme);
    render_column_padding_cell(&theme);
    render_newline();

    Ok(())
}

pub fn render_column_header_cell(column_name: &str, theme: &Theme) {
    render_cell(
        column_name,
        Some(theme.text_fg_color),
        Some(theme.board_bg_color),
        &theme.header_text_style,
    );
}

pub fn render_intersection_cell(theme: &Theme) {
    render_cell(
        &theme.intersection_char,
        Some(theme.intersection_color),
        Some(theme.board_bg_color),
        &[],
    );
}

pub fn render_intersection_horiz_cell(theme: &Theme) {
    render_cell(
        &theme.intersection_horiz_char,
        Some(theme.intersection_color),
        Some(theme.board_bg_color),
        &[],
    );
}

pub fn render_column_padding_cell(theme: &Theme) {
    render_cell(
        " ",
        Some(theme.text_fg_color),
        Some(theme.board_bg_color),
        &[],
    );
}

pub fn render_line_column_cell(str_num: &str, is_right_column: bool, theme: &Theme) {
    if is_right_column {
        render_cell(
            &format!("{: <2}", str_num),
            Some(theme.text_fg_color),
            Some(theme.board_bg_color),
            &theme.header_text_style,
        );
    } else {
        render_cell(
            &format!("{: >2}", str_num),
            Some(theme.text_fg_color),
            Some(theme.board_bg_color),
            &theme.header_text_style,
        );
    }
}

pub fn render_cell(
    str_value: &str,
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    styles: &[Styles],
) {
    let mut colored_str_value: ColoredString = str_value.into();

    if let Some(color) = fg_color {
        colored_str_value = colored_str_value.color(color);
    }

    if let Some(color) = bg_color {
        colored_str_value = colored_str_value.on_color(color);
    }

    for style in styles {
        colored_str_value = match &style {
            Styles::Bold => colored_str_value.bold(),
            Styles::Dimmed => colored_str_value.dimmed(),
            Styles::Italic => colored_str_value.italic(),
            Styles::Underline => colored_str_value.underline(),
            Styles::Blink => colored_str_value.blink(),
            Styles::Reversed => colored_str_value.reversed(),
            Styles::Hidden => colored_str_value.hidden(),
            Styles::Strikethrough => colored_str_value.strikethrough(),
            Styles::Clear => colored_str_value.clear(),
        };
    }

    print!("{}", &colored_str_value);
}

pub fn render_newline() {
    println!("");
}
