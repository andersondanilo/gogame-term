use crate::core::engine::Engine;
use crate::core::errors::AppError;
use crate::view::board::{BoardController, BoardTable, Theme};
use crate::view::events::{Config, Event, EventHandler, EventSideEffect, Events};
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Paragraph, Table};
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

    let board_table = BoardTable::new(engine.query_boardsize()?, &theme);
    let mut board_controller = BoardController::new(engine, board_table);

    // Setup event handlers
    let config = Config {
        tick_rate: Duration::from_millis(250),
        // ..Default::default()
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
                            Constraint::Min(20),
                        ]
                        .as_ref(),
                    )
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: f.size().width,
                        height: board_canvas_height_size,
                    });

                let board_table = board_controller.borrow_board_table();

                let tui_widths = board_table.get_tui_widths();

                let t = Table::new(board_table.get_tui_rows())
                    .block(Block::default().style(board_default_style))
                    .column_spacing(0)
                    .widths(&tui_widths);

                f.render_widget(t, chunks[0]);

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(20)].as_ref())
                    .split(Rect {
                        x: chunks[1].x + 2,
                        y: chunks[1].y,
                        width: chunks[1].width - 2,
                        height: chunks[1].height,
                    });

                let next_move_message =
                    Paragraph::new(format!("Next move: {}", board_controller.next_move_input));
                f.render_widget(next_move_message, right_chunks[0]);
            })
            .map_err(|_| AppError {
                message: "Can't draw terminal".to_string(),
            })?;

        let next_event = events.next().map_err(|_| AppError {
            message: "Can't get next event".to_string(),
        })?;

        match board_controller.on_event(next_event) {
            EventSideEffect::QuitApp => {
                break;
            }
            EventSideEffect::None => {}
        };
    }

    Ok(())
}
