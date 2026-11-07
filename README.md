# wasabi

A terminal typing test, written in Rust with [ratatui](https://ratatui.rs).

<img width="1000" height="725" alt="wasabie" src="https://github.com/user-attachments/assets/47af3ff5-9797-4ef0-964d-3b49a68411a0" />
<img width="1000" height="725" alt="code-test" src="https://github.com/user-attachments/assets/6d203a3d-3814-4746-b0ab-9c411a1c5ba7" />

- 15, 30 or 60 second tests from the 200 most common English words
- Your own word list, with optional punctuation and numbers
- Code tests in C and Rust (indentation is drawn, not typed)
- WPM, raw, accuracy, consistency, peak, and your most-missed keys
- Graphs of each run and of your progress over time
- Custom themes and ASCII banner

## Install

```sh
git clone https://github.com/doohed/wasabi.git
cd wasabi
cargo install --path .
```

Needs a recent Rust toolchain. `cargo uninstall wasabi` removes it (your
records stay). Or just `cargo run --release`.

## Settings

Press `esc` for settings. Words, banner and themes can all be customised:
`e` edits the file in `$EDITOR`, `r` reloads it, `x` resets to the built-in
(`b` toggles the banner).

<img width="1164" height="840" alt="settings" src="https://github.com/user-attachments/assets/f04e1500-febd-455f-b30b-c01152fa0197" />

## Files

Config lives in `~/.config/wasabi` (`settings.tsv`, `words.txt`,
`banner.txt`, `themes.conf`); records and history in `~/.local/share/wasabi`
(`records.tsv`, `history.tsv`). Everything is plain text and safe to edit or
delete.

## Licence

[MIT](LICENSE). The ASCII art is not mine — credits to whoever drew it.
