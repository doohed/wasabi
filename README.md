# wasabi

A terminal typing test. Written in Rust with
[ratatui](https://ratatui.rs). (credits to the ascii art creator)

<img width="1447" height="930" alt="image" src="https://github.com/user-attachments/assets/e22d2e87-eeac-491b-836b-34c0a1d52128" />


I build this just to learn how to use [ratatui](https://ratatui.rs) and for hobbie, I really liked the looks and decided to keep maintaining it.

## Install

```sh
git clone https://github.com/doohed/wasabi.git
cd wasabi
cargo run --release
```

Requires a recent Rust toolchain. No other system dependencies.

## Keys

Every printable character is test input, so there are no bare letter shortcuts
while typing — `q` types a `q`. That leaves Esc, Tab, and modifiers.

| | |
|---|---|
| `space` | commit the current word |
| `backspace` | delete a character, or step back a word |
| `tab` | restart with a fresh word list |
| `esc` | open the menu |
| `ctrl-c` | quit |

In the menu, `↑↓` (or `j`/`k`) moves, `enter` selects, `esc` goes back.

## What it measures

**WPM** is the standard definition: correctly typed characters divided by five,
over elapsed minutes. Incorrect and overflow characters score nothing, so
mistakes drag the number down.

**Accuracy** is charged at keystroke time. Fixing a typo still costs you — the
mistake happened, and a typing test that forgives it is measuring the wrong
thing.

Neither figure appears in the first second of a test. WPM divides by elapsed
time, and one character extrapolates to a six-figure score; there is no honest
number to show that early, so it shows `-- wpm`.

## Settings

Press `esc`, then pick a row.

<img width="493" height="345" alt="image" src="https://github.com/user-attachments/assets/cd9fae4f-a904-4ad0-b81f-921b6a8f6bb8" />

### Test length

15, 30 or 60 seconds. Changing it always restarts — a half-typed test measured
against a different clock would be meaningless.

### Records

A personal best per test length, kept between sessions.

<img width="536" height="302" alt="image" src="https://github.com/user-attachments/assets/5fa2fb35-ab6d-4068-a3ab-5c6d884f3db8" />

The accuracy shown is *that run's*, not your best ever — a personal best is one
result, and splitting it across runs would flatter you. Abandoned runs aren't
recorded.

### Banner

The ASCII art above the test is yours to replace.

| key | |
|---|---|
| `e` | open it in `$EDITOR`, seeded with the current art |
| `r` | reload it from disk |
| `x` | remove it and go back to the built-in |

Art is read from `banner.txt` (see [Files](#files)). Anything goes as long as
it's text — the app measures whatever you save and lays out around it.

Large art needs a large terminal. The banner screen tells you exactly how many
rows yours needs and how many you have; if it doesn't fit, the art is hidden
rather than allowed to shove the test off screen.

### Themes

<img width="476" height="297" alt="image" src="https://github.com/user-attachments/assets/963ef956-1ad7-4228-b09e-eeeeac385e9d" />

Moving the highlight applies the theme immediately, so the whole interface is
the preview. There is nothing to confirm and nothing to cancel.

`default` is built from *named* terminal colours, so it inherits whatever
palette your terminal already uses. The other four are fixed RGB: identical
everywhere, which is the point of choosing one, but they ignore your terminal's
own theme.

#### Your own themes

| key | |
|---|---|
| `e` | open `themes.conf` in `$EDITOR`, seeded with a worked example |
| `r` | reload it from disk |

One `[name]` block per theme. Every field is optional — anything you leave out
keeps the default theme's colour, so a two-line theme is perfectly valid:

```ini
[tokyonight]
text    = #c8d3f5   # correctly typed characters, and the clock
muted   = #a9b8e8   # menu rows
dim     = #3b4261   # untyped characters, labels, panel borders
error   = #ff757f   # a character that doesn't match
extra   = #ff98a4   # characters typed past the end of a word
accent  = #82aaff   # numbers, markers, panel titles
good    = #c3e88d   # a personal best
warn    = #ffc777   # warnings
banner  = #82aaff   # the ASCII art above the test
```

Colours are `#rrggbb`, or a terminal colour name — `black red green yellow blue
magenta cyan white gray darkgray lightred lightgreen lightyellow lightblue
lightmagenta lightcyan`. Names come from your terminal's palette, so a theme
built from them follows whatever colour scheme you already run; hex is exact
everywhere and ignores it.

Naming a block after a built-in **replaces** it, which is how you retune `nord`
rather than inventing `nord2`. Your themes are marked with a `·` in the picker.

A line that means nothing costs you that one colour, not the whole file — but
the picker says how many were ignored, so a typo is never silent.

## Files

State lives in `$XDG_DATA_HOME/wasabi`, falling back to
`~/.local/share/wasabi`:

| file | |
|---|---|
| `records.tsv` | personal bests, one tab-separated line per test length |
| `settings.tsv` | `key<TAB>value` preferences — currently just the theme |
| `banner.txt` | your ASCII art, absent until you make one |
| `themes.conf` | your own palettes, absent until you press `e` in the picker |

All three are plain text and safe to edit or delete by hand. Reading them is
infallible by design: a missing, unreadable or corrupt file means "no records
yet" or "default settings", never a failure to start. Refusing to open a typing
test because a scoreboard wouldn't parse would be the wrong trade.

## How it's put together

```
src/
├── main.rs        wiring
├── tui.rs         terminal setup, event loop, key routing
├── app/           all application state — no ratatui types anywhere
├── word.rs        one word: its target, what was typed, per-character state
├── wordlist.rs    the word pool
├── records.rs     personal bests, and their file
├── settings.rs    preferences, and their file
├── banner.rs      the ASCII art, and its file
├── theme.rs       every colour the interface uses, and the theme file
├── storage.rs     where files live
└── ui/            rendering — reads from App, never writes to it
```

The one rule worth knowing: **`app` holds no ratatui types and `ui` holds no
state.** The UI reads from `App`, never the other way round, which is what
makes the typing logic testable without a terminal. `theme.rs` names colours by
their job — `accent`, `dim`, `error` — never by hue, which is what lets a whole
palette swap underneath the renderers.

```sh
cargo test     # 117 tests, no terminal required
cargo clippy --all-targets
```
