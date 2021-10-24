mod core;
mod view;

use crate::core::engine::{EngineActor, QueryBoardSizeMessage};
use crate::core::errors::AppError;
use crate::view::board::{BoardControllerActor, BoardTableActor, Theme};
use actix::prelude::*;
use log::debug;
use std::sync::mpsc::channel;
use std::thread;

use clap::{App, Arg};

fn main() -> Result<(), AppError> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug-file")
                .short("d")
                .long("debug-file")
                .value_name("Debug file")
                .help("Output debug to a file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    core::logger::init_logger(
        matches.value_of("debug-file"),
        core::logger::get_logger_level_by_verbosity(matches.occurrences_of("v")),
    )?;

    let app_config = core::config::get_app_config(matches.value_of("config"))?;
    let theme = Theme::default();

    let (controller_tx, controller_rx) = channel::<Addr<BoardControllerActor>>();
    let (model_tx, model_rx) = channel::<Addr<BoardTableActor>>();
    let theme_copy = theme.clone();

    thread::spawn(move || {
        let sys = System::new();

        sys.block_on(async {
            let engine_arbiter = Arbiter::new();
            let bin = app_config.engine.bin.clone();
            let args = app_config.engine.args.clone();
            let engine_addr = EngineActor::start_in_arbiter(&engine_arbiter.handle(), move |_| {
                EngineActor::new(&bin, &args).unwrap()
            });

            let board_size = engine_addr
                .send(QueryBoardSizeMessage {})
                .await
                .unwrap()
                .unwrap();

            let board_arbiter = Arbiter::new();
            let board_addr =
                BoardTableActor::start_in_arbiter(&board_arbiter.handle(), move |_| {
                    BoardTableActor::new(board_size, &theme_copy)
                });

            let board_controller_arbiter = Arbiter::new();
            let board_controller_table_addr = board_addr.clone();
            let board_controller_addr = BoardControllerActor::start_in_arbiter(
                &board_controller_arbiter.handle(),
                move |_| {
                    BoardControllerActor::new(engine_addr, board_controller_table_addr.clone())
                },
            );

            controller_tx.send(board_controller_addr).unwrap();
            model_tx.send(board_addr).unwrap();
        });

        sys.run().unwrap();
    });

    debug!("waiting main actors");
    let board_controller_addr = controller_rx.recv().unwrap();
    let board_addr = model_rx.recv().unwrap();

    debug!("starting render");
    view::tui::render_app(&theme, board_controller_addr, board_addr)?;

    Ok(())
}
