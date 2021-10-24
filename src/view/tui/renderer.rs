use crate::core::errors::AppError;
use crate::view::board::{
    BoardControllerActor, GetBoardSizeMessage, GetNextMoveInput, GetTuiRowsMessage,
    GetTuiWidthsMessage, Theme,
};
use crate::view::events::{Config, EventSideEffect, Events, OnEventMessage};
use crate::BoardTableActor;
use actix::prelude::*;
use log::debug;
use std::time::Duration;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tokio::runtime::Runtime;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Paragraph, Table};
use tui::Terminal;

pub fn render_app(
    theme: &Theme,
    board_controller_addr: Addr<BoardControllerActor>,
    board_table_addr: Addr<BoardTableActor>,
) -> Result<(), AppError> {
    let stdout = std::io::stdout().into_raw_mode().map_err(|_| AppError {
        message: "Can't get stdout".to_string(),
    })?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| AppError {
        message: "Can't get terminal".to_string(),
    })?;

    let tokio_rt = Runtime::new().unwrap();

    let board_size: u8 = tokio_rt.block_on(board_table_addr.send(GetBoardSizeMessage {}))?;

    let board_canvas_width_size = 4 + (((board_size as u16) * 2) - 1) + 4;
    let board_canvas_height_size = (board_size as u16) + 2;

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

                let tui_widths = tokio_rt
                    .block_on(board_table_addr.send(GetTuiWidthsMessage {}))
                    .unwrap();
                let tui_rows = tokio_rt
                    .block_on(board_table_addr.send(GetTuiRowsMessage {}))
                    .unwrap();

                let t = Table::new(tui_rows)
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

                let next_move_input = tokio_rt
                    .block_on(board_table_addr.send(GetNextMoveInput {}))
                    .unwrap();

                let next_move_message = Paragraph::new(format!("Next move: {}", next_move_input));
                f.render_widget(next_move_message, right_chunks[0]);
            })
            .map_err(|_| AppError {
                message: "Can't draw terminal".to_string(),
            })?;

        let next_event = events.next().map_err(|_| AppError {
            message: "Can't get next event".to_string(),
        })?;

        let event_response = tokio_rt
            .block_on(board_controller_addr.send(OnEventMessage { event: next_event }))
            .unwrap();

        match event_response {
            EventSideEffect::QuitApp => {
                break;
            }
            EventSideEffect::None => {}
        }
    }

    Ok(())
}
