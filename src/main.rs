use log::info;
use std::error::Error;

mod audio;
mod input;
mod queue;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting termitune");

    ui::run()?;

    Ok(())
}
