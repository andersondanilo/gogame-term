use crate::core::engine::Engine;
use crate::core::errors::AppError;
use crate::view::board::{BoardTable, Theme};
use crate::view::events::{Config, Event, Events};
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Table};
use tui::Terminal;

use termion::event::Key;

pub fn render_app(engine: &mut Engine) -> Result<(), AppError> {
    let stdout = std::io::stdout().into_raw_mode().map_err(|_| AppError {
        message: "Can't get stdout".to_string(),
    })?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| AppError {
        message: "Can't get terminal".to_string(),
    })?;

    let board_size = engine.query_boardsize()?;
    let theme = Theme::default();

    let board_canvas_width_size = 4 + (((board_size as u16) * 2) - 1) + 4;
    let board_canvas_height_size = (board_size as u16) + 2;

    let mut board_table = BoardTable::from_engine(engine, &theme)?;

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(250),
        ..Default::default()
    };
    let events = Events::with_config(config);

    let board_default_style = Style::default()
        .bg(theme.board_bg_color)
        .fg(theme.text_fg_color);

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(board_canvas_width_size),
                            Constraint::Max(50),
                        ]
                        .as_ref(),
                    )
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: board_canvas_width_size + 50,
                        height: board_canvas_height_size,
                    });

                let tui_widths = board_table.get_tui_widths();

                let t = Table::new(board_table.get_tui_rows())
                    .block(Block::default().style(board_default_style))
                    .column_spacing(0)
                    .widths(&tui_widths);

                f.render_widget(t, chunks[0]);
            })
            .map_err(|_| AppError {
                message: "Can't draw terminal".to_string(),
            })?;

        match events.next().map_err(|_| AppError {
            message: "Can't get next event".to_string(),
        })? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }

                _ => {}
            },
            Event::Tick => {
                // app.update();
            }
        }
    }

    Ok(())
}
