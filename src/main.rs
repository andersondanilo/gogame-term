mod core;
mod renderer;

use crate::core::engine::Engine;
use crate::core::errors::AppError;

use clap::{App, Arg};
use log::info;

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
    let mut engine = Engine::start(&app_config.engine.bin, &app_config.engine.args)?;

    info!("started engine: {}", engine.get_name()?);

    renderer::render_board(&mut engine)?;

    Ok(())
}
