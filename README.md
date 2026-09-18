# wasabi

A terminal typing test. Written in Rust with
[ratatui](https://ratatui.rs). (credits to the ascii art creator)

<img width="1000" height="725" alt="wasabie" src="https://github.com/user-attachments/assets/47af3ff5-9797-4ef0-964d-3b49a68411a0" />


I built this just to learn how to use [ratatui](https://ratatui.rs), and for a
hobby. I really liked the looks and decided to keep maintaining it.

- 15, 30 or 60 second tests, drawn from the 200 most common English words
- Your own word list, and punctuation and numbers over whatever it holds
- A code test — pre-written C and Rust, typed as it was written
- WPM, raw, accuracy, consistency, peak, and the keys you keep missing
- A graph of the run you just typed, and a graph of every run before it
- Themes and ASCII art, both yours to replace, both hand-editable text

## Install

```sh
git clone https://github.com/doohed/wasabi.git
cd wasabi
./install.sh
```

That builds it and drops the binary somewhere on your `PATH`, so you can run
`wasabi` from anywhere. `./install.sh --uninstall` removes it again, leaving
your records and settings alone. Set `INSTALL_DIR` to choose where it goes.

To run it without installing, `cargo run --release`.

Requires a recent Rust toolchain. No other system dependencies — the built-in
art, word list and code snippets are compiled into the binary, so there is
nothing to ship beside it.

## Keys

Every printable character is test input, so there are no bare letter shortcuts
while typing — `q` types a `q`. That leaves Esc, Tab, and modifiers.

| | |
|---|---|
| `space` | commit the current word |
| `enter` | the same, and how you end a line of code |
| `backspace` | delete a character, or step back a word |
| `tab` | restart, with a fresh deal |
| `esc` | open the menu |
| `ctrl-c` | quit |

In the menu, `↑↓` (or `j`/`k`) moves, `enter` selects, `esc` goes back. A `✓`
marks a setting that's in force. There can be several — the length is a choice
of one, but the modifiers are independent switches — and a row with no tick
beside it is either a screen to open or a setting the current test can't use.

## What it measures

**WPM** is the standard definition: correctly typed characters divided by five,
over elapsed minutes. Incorrect and overflow characters score nothing, so
mistakes drag the number down.

**Raw WPM** is the same sum counting every keystroke, right or wrong — spaces
included, exactly like WPM. It's the speed of your fingers where WPM is the
speed of your typing, and the gap between the two is what the mistakes cost
you. It can never be the lower of the pair: both count the same characters,
and raw just stops short of asking whether they were the right ones.

**Accuracy** is charged at keystroke time. Fixing a typo still costs you — the
mistake happened, and a typing test that forgives it is measuring the wrong
thing.

**Consistency** is how even your pace was, as a percentage: the spread of your
per-second speeds relative to their average, turned the right way up, so 100%
is a metronome. Measured against your own average, a steady 40 wpm scores as
well as a steady 100 — it's a question about evenness, not speed.

**Peak** is your fastest single second.

**Worst keys** are the three characters you got wrong most, worst first, with
the count beside each. Charged to the key you *meant* to hit rather than the
one you actually pressed — typing `b` where the word wanted `v` is a `v` you
can go and practise, where the `b` is only where your finger landed that time.
Characters typed past the end of a word belong to no key at all: there was
nothing to get right, so they cost you accuracy and name nobody.

They're ranked by what each key cost you over the run, not by how often it went
wrong per attempt. A rate would put a letter fumbled once out of one above one
fumbled six times out of sixty, and over a test this short that's the wrong
thing to send you away to practise. Where two keys cost the same, the rarer one
ranks first — the same judgement, applied to the one figure left to separate
them.

None of the speeds appear in the first second of a test. They divide by elapsed
time, and one character extrapolates to a six-figure score; there is no honest
number to show that early, so it shows `-- wpm`.

## Results

When the clock stops, the test is replaced by the run's own history: a reading
a second, plotted.

The accented line is your score as it stood at each moment — the number the
clock was showing. The quiet line is each second on its own, which is jagged
where the score is smooth, because it isn't averaged over everything that came
before. Where the two diverge you can see the run's history weighing on it: a
bad patch keeps costing you long after you've typed past it. Red dots mark the
seconds you made a mistake in, on the line that shows those seconds.

Both lines are in wpm on one scale, so the distance between them means
something. Under the graph are the six figures above, and under those the keys
that cost you — the one row that isn't a score. The rest of the screen says how
the run went; that row says what to do about it.

A clean run shows `—` there rather than an empty row, so a blank is never
something that failed to draw.

The graph is the first thing dropped when the terminal is small, for the same
reason the art is: the numbers are the result, and the picture is a nicer way
of looking at them. Below 64 columns or 21 rows you get the figures alone.

## Settings

Press `esc`, then pick a row.

<img width="493" height="345" alt="image" src="https://github.com/user-attachments/assets/cd9fae4f-a904-4ad0-b81f-921b6a8f6bb8" />

### Test length

15, 30 or 60 seconds, remembered between sessions. Changing it always restarts
— a half-typed test measured against a different clock would be meaningless.

### Words

The pool the test draws from is yours to replace, the same way the art is.

| key | |
|---|---|
| `e` | open it in `$EDITOR`, seeded with the built-in list |
| `r` | reload it from disk |
| `x` | remove it and go back to the built-in |

Words are read from `words.txt` (see [Files](#files)). One word per line, or
several to a line — the file is split on whitespace, so a list and a paragraph
of prose are both valid. Lines starting with `#` are comments, which is what
lets the seeded file explain itself; the cost is words beginning with `#`, and
a typing test is a fair place to spend those.

Words are drawn at random *with* repetition, so a pool shorter than the test is
fine — a ten-word list just repeats a lot, and the screen says so rather than
refusing it. A file with no words in it reads as absent, so emptying the file
gets you the built-in list back rather than a test with nothing to type.

The built-in list is the 200 most common English words, which is what
MonkeyType's default test draws from: short and overwhelmingly ASCII, so lines
wrap predictably and the test measures typing rather than reading.

### Punctuation and numbers

Two switches in the menu, each with a `✓` when it's on.

**Punctuation** attaches marks to about one word in four — mostly commas and
full stops, sometimes a colon or semicolon, occasionally a pair of quotes or
brackets — and capitalises the first word of every sentence, looking past a
closing quote so `end."` still ends one.

**Numbers** replaces about one word in eight with a number of up to four
digits. Never the first word: a test that opens on a bare number reads as a
rendering fault rather than a setting you turned on.

Both are transforms over the words after they're dealt, not word lists of their
own — which is what lets them apply to *your* pool. Turn punctuation on over a
Spanish list and you get punctuated Spanish, with nothing shipped to make that
work. Changing either always restarts, for the same reason changing the length
does: the words on screen were dealt under the old setting.

### Code

Type pre-written code instead of words. Pick a language — C or Rust — and the
test deals whole snippets of it: a binary search, a bubble sort, a gcd. Short
and deliberately ordinary, so what it measures is your hands on the punctuation
rather than your memory of a clever algorithm.

Code keeps its lines. A snippet isn't reflowed to fill the column — reflowed
code stops looking like code, which is the only reason to type it — so the
break happens where the author put it.

**Indentation is drawn, not typed.** Lining code up is the editor's job in real
life, and a test that charged you for leading spaces would be measuring your
patience. `enter` ends a line, the way it would in an editor; `space` does the
same thing, and neither is scored against the other.

Blank lines are dropped — there is nothing to type on one — and the snippets
are compiled into the binary rather than read from a file. Unlike the word list
a snippet has to actually *be* valid code, and checking that is not something
this app can do for a file someone hands it.

A code test is its own test for records: a snippet full of braces scores
nothing like a page of common words, so it doesn't have to compete with your
word-test personal best and lose.

Punctuation and numbers have no say over code that was written with its own, so
they're greyed out of force while a language is selected — the menu shows no
tick beside them. Turning either back on switches you back to the word test,
because asking for punctuation is asking for the test that can have it; your
word settings are remembered untouched in the meantime.

### Records

A personal best per test, kept between sessions.

<img width="536" height="302" alt="image" src="https://github.com/user-attachments/assets/5fa2fb35-ab6d-4068-a3ab-5c6d884f3db8" />

The accuracy shown is *that run's*, not your best ever — a personal best is one
result, and splitting it across runs would flatter you. Abandoned runs aren't
recorded.

Each combination of length and setting keeps its own record, and the heading
says which one you're looking at. A punctuated test is a different test — the
same argument the length makes, and a stronger one for a code test — so neither
has to compete with the plain one and lose.

Swapping your word list doesn't split them, though: the app can't tell a new
list from an edited one, so what you practise on is on you.

#### Progress

Under the table is every run you've finished at the setting the test is
currently on, oldest on the left. The quiet line is the runs themselves and the
accented one is a ten-run trailing average — the same pairing as the results
graph, where the jagged line is the moment and the smooth one is what it adds
up to. The dot marks your best.

One setting rather than all of them, because a 15s run and a 60s run are
different tests, and so are a plain one, a punctuated one and a page of Rust:
plotting them on one line would put a step in it every time you changed the
setting and call that improvement. Change anything in the menu and the plot
follows.

The last fifty runs are shown — a line squeezing a thousand runs into forty
columns is a texture, not a trend — but the axis numbers them by where they sit
in your whole history, so it agrees with the `runs` column above.

A trailing average rather than one taken from your very first run: a cumulative
average stops moving once there are a few hundred runs behind it, which answers
"how fast have you ever been" when the question is "how fast are you now". The
speed axis starts at zero, so a good week looks like a good week rather than a
cliff.

The plot needs two runs at that setting before there's a shape to draw, and it's
the first thing dropped when the terminal is short — the bests are the record,
and the picture is a nicer way of looking at what came after them. Nothing is
plotted from before you upgraded: the history file starts when this version
does, though your bests and run counts carry over untouched.

### Banner

The ASCII art above the test is yours to replace.

| key | |
|---|---|
| `e` | open it in `$EDITOR`, seeded with the current art |
| `b` | turn the banner off, or back on |
| `r` | reload it from disk |
| `x` | remove it and go back to the built-in |

With the banner off the test sits in the middle of the screen instead of
hanging below the art. Off is a preference, not a deletion — your art stays on
disk, and `b` brings it straight back. `x` is what removes it.

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
keeps the default theme's colour, so a two-line theme is perfectly valid. The
picker shows each theme's nine colours in the order below, so the row and the
file you edit read the same way round:

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

Things you write live in `$XDG_CONFIG_HOME/wasabi`, falling back to
`~/.config/wasabi`:

| file | |
|---|---|
| `settings.tsv` | `key<TAB>value` preferences — theme, length, banner, modifiers, language |
| `banner.txt` | your ASCII art, absent until you make one |
| `words.txt` | your word list, absent until you make one |
| `themes.conf` | your own palettes, absent until you press `e` in the picker |

Things the app writes live in `$XDG_DATA_HOME/wasabi`, falling back to
`~/.local/share/wasabi`:

| file | |
|---|---|
| `records.tsv` | personal bests, one tab-separated line per test |
| `history.tsv` | every finished run, one tab-separated line each, oldest first |

`history.tsv` is append-only and never trimmed: a run is forty bytes, the file
is read once at startup, and the early runs are the only part of it that shows
how far you've come. Delete it and you lose the plot, not your records.

The split is the XDG convention, and it earns its keep: losing your personal
bests is losing history, losing a setting is not, so the two deserve different
backup habits. Versions before this kept everything in the data directory —
those files are moved into place automatically on first run.

All six are plain text and safe to edit or delete by hand. Reading them is
infallible by design: a missing, unreadable or corrupt file means "no records
yet" or "default settings", never a failure to start. Refusing to open a typing
test because a scoreboard wouldn't parse would be the wrong trade.

## How it's put together

```
src/
├── main.rs        wiring
├── tui.rs         terminal setup, event loop, key routing
├── app/           all application state — no ratatui types anywhere
├── ui/            rendering — reads from App, never writes to it
├── typing/        what a test is made of, and how a run went
│   ├── word.rs        one word: its target, what was typed, per-character state
│   ├── wordlist.rs    the word pool, and its file
│   ├── modifiers.rs   what the test does to the words once they're dealt
│   ├── mode.rs        which test this is: words with settings, or a language
│   ├── snippets.rs    the code the code test deals
│   ├── timeline.rs    a reading a second, and the figures derived from them
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

The three middle folders are split by **how long a thing lives**. Everything in
`typing/` belongs to one run and is thrown away by the next; `scores/` is what
outlives the run that made it; `config/` is what the user chose, which outlives
all of them. That is also why `wordlist.rs` sits under `typing/` despite owning
a file in the config directory: what it *is* is the pool a test draws from, and
which directory its file lands in is a detail of the loading.

The one rule worth knowing: **`app` holds no ratatui types and `ui` holds no
state.** The UI reads from `App`, never the other way round, which is what
makes the typing logic testable without a terminal. `theme` names colours by
their job — `accent`, `dim`, `error` — never by hue, which is what lets a whole
palette swap underneath the renderers.

Every loader in `config/` and `scores/` is infallible, for the reason given
under [Files](#files) — which is why none of them returns a `Result`.

```sh
cargo test     # 256 tests, no terminal required
cargo clippy --all-targets
```

## Licence

[MIT](LICENSE). The ASCII art is not mine — credits to whoever drew it.
