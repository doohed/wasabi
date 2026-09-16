use std::path::Path;
use std::process::Command;
use std::time::Duration;

use color_eyre::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::app::{App, EditTarget, Screen};
use crate::ui;

/// How long to wait for a key before redrawing anyway.
///
/// The countdown has to tick down with no input at all, so the loop can't just
/// block on `event::read`. 100ms is well under the one-second resolution the
/// header displays, and idles at ~10 frames a second.
const TICK: Duration = Duration::from_millis(100);

/// Sets up the terminal, runs the loop, and always restores the terminal.
pub fn run(mut app: App) -> Result<()> {
    let terminal = ratatui::init();
    let result = event_loop(terminal, &mut app);
    ratatui::restore();
    result
}

fn event_loop(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        // the only place ui:: is touched
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(TICK)? {
            if let Some(key) = event::read()?.as_key_press_event() {
                handle_key(app, key);
            }
        }

        // Handing the terminal to another program is the event loop's job:
        // nothing else in the app knows there is a terminal to hand over.
        if let Some(target) = app.take_edit() {
            terminal = edit_file(terminal, app, target)?;
        }

        // After the input, so a keystroke that lands on the last millisecond
        // still counts, and unconditional, so the test can end untouched.
        app.tick();
    }
    Ok(())
}

/// Translate a keypress into an `App` method call.
fn handle_key(app: &mut App, key: KeyEvent) {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Ctrl-C is the only way out, and works from every screen. Esc can't be:
    // it has to mean "leave this screen", or the menu would be a trap.
    if ctrl && key.code == KeyCode::Char('c') {
        app.quit();
        return;
    }

    match app.screen {
        Screen::Test => test_key(app, key, ctrl),
        Screen::Menu => menu_key(app, key),
        Screen::Banner => banner_key(app, key),
        Screen::Theme => theme_key(app, key),
        Screen::Records => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace) {
                app.back();
            }
        }
    }
}

/// Keys during the test.
///
/// Every printable character is test input, so there are no bare letter
/// shortcuts: `q` types a `q`. That leaves Esc, Tab, and modifiers.
fn test_key(app: &mut App, key: KeyEvent, ctrl: bool) {
    match key.code {
        KeyCode::Esc => app.open_menu(),
        KeyCode::Tab => app.restart(),
        KeyCode::Backspace => app.backspace(),
        KeyCode::Char(' ') => app.type_space(),
        // Guarding on `ctrl` keeps chords (Ctrl-A and friends) from being
        // typed as plain letters.
        KeyCode::Char(c) if !ctrl && !c.is_control() => app.type_char(c),
        _ => {}
    }
}

/// Keys in the menu. `j`/`k` alongside the arrows, since the letters are free
/// here and the fingers are already on the home row.
fn menu_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.back(),
        KeyCode::Up | KeyCode::Char('k') => app.menu_move(-1),
        KeyCode::Down | KeyCode::Char('j') => app.menu_move(1),
        KeyCode::Enter | KeyCode::Char(' ') => app.menu_select(),
        _ => {}
    }
}

/// Keys on the banner screen.
fn banner_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.back(),
        KeyCode::Char('e') => app.request_edit(EditTarget::Banner),
        KeyCode::Char('r') => app.reload_banner(),
        KeyCode::Char('x') => app.reset_banner(),
        _ => {}
    }
}

/// Keys in the theme picker. Moving applies the theme, so the whole interface
/// is the preview; there is nothing to confirm and nothing to cancel.
fn theme_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.back(),
        KeyCode::Up | KeyCode::Char('k') => app.theme_move(-1),
        KeyCode::Down | KeyCode::Char('j') => app.theme_move(1),
        KeyCode::Char('e') => app.request_edit(EditTarget::Themes),
        KeyCode::Char('r') => app.reload_themes(),
        _ => {}
    }
}

/// Hand the terminal to `$EDITOR`, then take it back and reload what changed.
///
/// The terminal is moved in and a fresh one returned rather than restored in
/// place: the old one is invalid the moment we leave the alternate screen, and
/// making that a move means the type system says so.
fn edit_file(
    terminal: DefaultTerminal,
    app: &mut App,
    target: EditTarget,
) -> Result<DefaultTerminal> {
    // VISUAL before EDITOR: the former is conventionally the full-screen one,
    // which is what drawing pictures wants.
    let editor = std::env::var_os("VISUAL")
        .or_else(|| std::env::var_os("EDITOR"))
        .filter(|editor| !editor.is_empty());

    let Some(editor) = editor else {
        app.set_status("no $EDITOR set — edit the file above, then press r");
        return Ok(terminal);
    };

    let seeded = match target {
        EditTarget::Banner => app.banner().seed(),
        EditTarget::Themes => app.themes().seed(),
    };

    let path = match seeded {
        Ok(path) => path.to_path_buf(),
        Err(error) => {
            app.set_status(format!("couldn't create the file: {error}"));
            return Ok(terminal);
        }
    };

    Ok(run_editor(terminal, app, &editor, &path, target))
}

fn run_editor(
    terminal: DefaultTerminal,
    app: &mut App,
    editor: &std::ffi::OsStr,
    path: &Path,
    target: EditTarget,
) -> DefaultTerminal {
    drop(terminal);
    ratatui::restore();

    let status = Command::new(editor).arg(path).status();

    // Whatever happened to the editor, the terminal comes back. `clear`
    // because the editor left its own drawing on the alternate screen.
    let mut terminal = ratatui::init();
    let _ = terminal.clear();

    match status {
        Ok(status) if status.success() => match target {
            EditTarget::Banner => app.reload_banner(),
            EditTarget::Themes => app.reload_themes(),
        },
        // A non-zero exit is usually a deliberate abort (`:cq`), so don't
        // reload — but say so, rather than looking like nothing happened.
        Ok(_) => app.set_status("editor exited with an error — nothing reloaded"),
        Err(error) => app.set_status(format!(
            "couldn't run {}: {error}",
            editor.to_string_lossy()
        )),
    }

    terminal
}
