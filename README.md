# wasabi

A terminal typing test, written in Rust with [ratatui](https://ratatui.rs).

<img width="1000" height="730" alt="wasabi" src="https://github.com/user-attachments/assets/1e95662d-88fd-4506-ba1d-f96974233567" />

- 15, 30 or 60 second tests, or 25, 50 or 100 words, from the 200 most
  common English words
- Your own word list, with optional punctuation and numbers
- Code tests in C and Rust (indentation is drawn, not typed)
- WPM, raw, accuracy, consistency, peak, and your most-missed keys
- Practice mode, which deals more words containing the keys you miss most
- A pace caret that races your personal best
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

<img width="1140" height="832" alt="wasabi" src="https://github.com/user-attachments/assets/3bf89dd5-3f49-4626-94c0-b7fdc2b9cec2" />

## Files

Config lives in `~/.config/wasabi` (`settings.tsv`, `words.txt`,
`banner.txt`, `themes.conf`); records, history and per-key stats in
`~/.local/share/wasabi` (`records.tsv`, `history.tsv`, `keys.tsv`).
Everything is plain text and safe to edit or delete.

## Licence

[MIT](LICENSE). The ASCII art is not mine — credits to whoever drew it.
