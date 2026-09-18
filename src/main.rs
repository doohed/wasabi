mod app;
mod banner;
mod history;
mod misses;
mod modifiers;
mod records;
mod settings;
mod storage;
mod theme;
mod timeline;
mod tui;
mod ui;
mod word;
mod wordlist;

use color_eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?; // pretty panic/error reports

    // Before anything reads a file: older versions kept config in the data
    // directory, and those files are still the user's.
    storage::migrate();

    let app = App::new();
    tui::run(app)
}
