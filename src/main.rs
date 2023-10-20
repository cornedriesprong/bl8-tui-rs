use log::LevelFilter;
use simplelog::*;
use std::{fs::File, result::Result};

mod app;
use app::App;

mod engine;
mod history;
mod limiter;
mod sequencer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    let log_file = File::create("log.txt").unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    let mut app = App::new();
    app.run()?;
    Ok(())
}
