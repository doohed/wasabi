# wasabi

A terminal typing test. Written in Rust with
[ratatui](https://ratatui.rs). (credits to the ascii art creator)

<img width="1000" height="725" alt="wasabie" src="https://github.com/user-attachments/assets/47af3ff5-9797-4ef0-964d-3b49a68411a0" />
<img width="1000" height="725" alt="code-test" src="https://github.com/user-attachments/assets/6d203a3d-3814-4746-b0ab-9c411a1c5ba7" />


I built this just to learn how to use [ratatui](https://ratatui.rs), and for a
hobby. I really liked the looks and decided to keep maintaining it.

- 15, 30 or 60 second tests from the 200 most common English words
- Your own word list, plus punctuation and numbers over whatever it holds
- A code test — pre-written C and Rust, typed as it was written
- WPM, raw, accuracy, consistency, peak, and the keys you keep missing
- A graph of the run you just typed, and a graph of every run before it
- Themes and ASCII art, both yours to replace, both plain text

## Install

```sh
git clone https://github.com/doohed/wasabi.git
cd wasabi
./install.sh
```

The script:

1. Checks you have `cargo`, and points you at [rustup](https://rustup.rs) if not
2. Runs `cargo build --release`
3. Copies the binary to `~/.cargo/bin`, or `~/.local/bin` if that doesn't exist
4. Tells you the size, and whether that directory is on your `PATH` — with the
   exact line to add if it isn't

It touches nothing else: no `sudo`, no system directories, no files outside
the install directory. It installs over a copy that's currently running, too —
the binary goes to a temporary name and is renamed into place.

```sh
INSTALL_DIR=/usr/local/bin ./install.sh   # somewhere else
./install.sh --uninstall                  # remove the binary
cargo run --release                       # don't install at all
```

Uninstalling removes only the binary. Your settings, records, themes and art
stay where they are — see [Files](#files) — so reinstalling picks up where you
left off.

Needs a recent Rust toolchain and nothing else: the art, word list and code
snippets are compiled into the binary.

## Settings

Press `esc`, to access settings.

<img width="1164" height="840" alt="image" src="https://github.com/user-attachments/assets/f04e1500-febd-455f-b30b-c01152fa0197" />


### Words

The pool the test draws from, replaceable like the art.

| key | |
|---|---|
| `e` | open `words.txt` in `$EDITOR`, seeded with the built-in list |
| `r` | reload it from disk |
| `x` | remove it and go back to the built-in |

One word per line or several to a line — the file is split on whitespace, so a
list and a paragraph of prose both work. Lines starting with `#` are comments.
Words are drawn with repetition, so a pool shorter than the test just repeats.
An empty file reads as absent and gets you the built-in list back.

### Punctuation and numbers

Two switches, applied to the words after they're dealt — so they work over
*your* pool, not a list of their own.

**Punctuation** marks about one word in four, mostly commas and full stops,
and capitalises the start of each sentence.

**Numbers** replaces about one word in eight with a number of up to four
digits, never the first.

### Code

Pre-written snippets instead of words. Pick C or Rust and you get a binary
search, a bubble sort or a gcd.

Code keeps its lines — it isn't reflowed to fill the column — and
**indentation is drawn, not typed**. `enter` ends a line, `space` does the
same, and neither is scored against the other.

Punctuation and numbers have no say over code, so they show no tick while a
language is selected. Turning either on switches back to the word test; your
word settings are remembered meanwhile.

### Records

The accuracy shown is *that run's*, not your best ever. Abandoned runs aren't
recorded. Each combination of length and setting keeps its own record — a
punctuated test and a page of Rust score nothing like plain words — and the
heading says which you're looking at. Swapping your word list doesn't split
them: the app can't tell a new list from an edited one.

### Progress

Under the table, every run you've finished at the current setting, oldest on
the left. The quiet line is the runs; the accented one is a ten-run trailing
average; the dot marks your best. The last fifty are shown, numbered by where
they sit in your whole history.

Needs two runs at that setting before there's a shape to draw, and it's the
first thing dropped when the terminal is short. Nothing is plotted from before
you upgraded — the history file starts when this version does, though your
bests carry over.

### Banner

The ASCII art above the test.

| key | |
|---|---|
| `e` | open `banner.txt` in `$EDITOR`, seeded with the current art |
| `b` | turn it off, or back on |
| `r` | reload it from disk |
| `x` | remove it and go back to the built-in |

Off is a preference, not a deletion — `b` brings it back, `x` removes it. Any
text works; the app measures whatever you save and lays out around it. If it
doesn't fit the terminal it's hidden rather than allowed to shove the test off
screen, and the screen tells you how many rows it needs.

### Themes

Moving the highlight applies the theme, so the interface is the preview.
`default` uses *named* terminal colours and inherits your terminal's palette;
the other four are fixed RGB.

#### Your own themes

`e` opens `themes.conf` in `$EDITOR` seeded with an example, `r` reloads it.

## Files

Yours, in `$XDG_CONFIG_HOME/wasabi` (or `~/.config/wasabi`):

| file | |
|---|---|
| `settings.tsv` | `key<TAB>value` preferences |
| `words.txt` | your word list |
| `banner.txt` | your ASCII art |
| `themes.conf` | your palettes |

The app's, in `$XDG_DATA_HOME/wasabi` (or `~/.local/share/wasabi`):

| file | |
|---|---|
| `records.tsv` | personal bests, one line per test |
| `history.tsv` | every finished run, oldest first |

All six are plain text, safe to edit or delete by hand, and absent until
there's something to put in them. `history.tsv` is append-only and never
trimmed — delete it and you lose the progress plot, not your records.

Reading them is infallible by design: a missing, unreadable or corrupt file
means "no records yet" or "default settings", never a failure to start.
Versions before this kept everything in the data directory; those files are
moved into place on first run.

## How it's put together

```
src/
├── main.rs        wiring
├── tui.rs         terminal setup, event loop, key routing
├── app/           all application state — no ratatui types anywhere
├── ui/            rendering — reads from App, never writes to it
├── typing/        what a test is made of, and how a run went
│   ├── word.rs        one word: its target, what was typed, per-char state
│   ├── wordlist.rs    the word pool, and its file
│   ├── modifiers.rs   what the test does to the words once dealt
│   ├── mode.rs        which test this is: words with settings, or a language
│   ├── snippets.rs    the code the code test deals
│   ├── timeline.rs    a reading a second, and the figures from them
│   └── misses.rs      which keys went wrong, and the worst of them
├── scores/        what finished runs leave behind
│   ├── records.rs     personal bests, and their file
│   └── history.rs     every finished run, and their file
└── config/        the user's own files, and where they live
    ├── storage.rs     where files live, and moving them when that changes
    ├── settings.rs    preferences, and their file
    ├── banner.rs      the ASCII art, and its file
    └── theme/         every colour the interface uses
        ├── mod.rs         the palette, the built-ins, and the collection
        └── format.rs      reading `themes.conf`
```

The three middle folders split by **how long a thing lives**: `typing/` belongs
to one run, `scores/` outlives the run that made it, `config/` outlives all of
them. That's why `wordlist.rs` is under `typing/` despite owning a config file
— it's the pool a test draws from.

The one rule worth knowing: **`app` holds no ratatui types and `ui` holds no
state.** That's what makes the typing logic testable without a terminal.
`theme` names colours by their job — `accent`, `dim`, `error` — never by hue,
which lets a whole palette swap underneath the renderers.

Every decision above has its reasoning in a comment next to the code that makes
it; this file says what it does, the source says why.

```sh
cargo test     # 256 tests, no terminal required
cargo clippy --all-targets
```

## Licence

[MIT](LICENSE). The ASCII art is not mine — credits to whoever drew it.
