use log::LevelFilter;
use simplelog::*;
use std::{fs::File, io::Result};

mod app;
use app::App;

mod history;

const CELL_WIDTH: usize = 4;
const CELL_HEIGHT: usize = 1;

fn main() -> Result<()> {
    // init logging
    let log_file = File::create("log.txt").unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    let mut app = App::new();
    app.run()?;
    Ok(())
}
