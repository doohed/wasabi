mod app;
mod banner;
mod records;
mod settings;
mod storage;
mod theme;
mod tui;
mod ui;
mod word;
mod wordlist;

use color_eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?; // pretty panic/error reports
    let app = App::new();
    tui::run(app)
}
