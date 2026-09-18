mod app;
mod config;
mod scores;
mod tui;
mod typing;
mod ui;

use color_eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?; // pretty panic/error reports

    // Before anything reads a file: older versions kept config in the data
    // directory, and those files are still the user's.
    config::storage::migrate();

    let app = App::new();
    tui::run(app)
}
