//! Reading `themes.conf`: the file format, and the colours written in it.
//!
//! Split from the palette itself because they are two jobs. What a theme *is*
//! — nine colours named by what they do — is a question about the interface;
//! how a line of text becomes one is a question about a file the user typed by
//! hand, and it answers to different pressures. Everything here is forgiving
//! on purpose: one bad line costs that colour, never the file.

use ratatui::style::Color;

use super::{Theme, FIELDS};

/// Resolve a colour written in a theme file.
///
/// Terminal colour names come from the user's own palette, so a theme built
/// from them follows their terminal; `#rrggbb` is exact everywhere and ignores
/// it. Both are allowed because both are legitimate things to want.
pub(super) fn colour(value: &str) -> Option<Color> {
    let value = value.trim();

    Some(match value.to_ascii_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => return parse_hex(value),
    })
}

fn parse_hex(value: &str) -> Option<Color> {
    let digits = value.strip_prefix('#')?;
    if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).ok();
    Some(Color::Rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

/// Parse `[name]` blocks of `field = colour` into themes.
///
/// Returns the themes and a count of lines that meant nothing. Tolerant on
/// purpose — one bad line costs you that colour, not the whole file — but the
/// count is reported, so a typo isn't silent.
pub(super) fn parse(text: &str, base: &Theme) -> (Vec<Theme>, usize) {
    let mut themes: Vec<Theme> = Vec::new();
    let mut skipped = 0;

    for line in text.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }

        // `[name]` opens a new theme, starting from the default's colours so a
        // file only has to name the fields it wants to change.
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let name = name.trim();
            if name.is_empty() {
                skipped += 1;
                continue;
            }
            themes.push(Theme {
                name: name.to_string(),
                custom: true,
                ..base.clone()
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            skipped += 1;
            continue;
        };

        // A field before any `[name]` has no theme to belong to.
        let Some(theme) = themes.last_mut() else {
            skipped += 1;
            continue;
        };

        let key = key.trim().to_ascii_lowercase();
        match FIELDS.iter().find(|(field, _)| *field == key) {
            Some((_, get)) => match colour(value) {
                Some(colour) => *get(theme) = colour,
                None => skipped += 1,
            },
            None => skipped += 1,
        }
    }

    (themes, skipped)
}

/// Cut a trailing comment off a line.
///
/// `#` is both the comment marker and the prefix of a hex colour, so a naive
/// split at the first `#` silently eats every colour in the file. A `#` that
/// begins six hex digits is a value; anything else starts a comment.
fn strip_comment(line: &str) -> &str {
    for (at, _) in line.match_indices('#') {
        let rest = &line.as_bytes()[at + 1..];
        let starts_a_colour = rest.len() >= 6 && rest[..6].iter().all(u8::is_ascii_hexdigit);

        if !starts_a_colour {
            return &line[..at];
        }
    }

    line
}

/// What a fresh themes file contains.
pub(super) const EXAMPLE: &str = "\
# Your own themes for wasabi.
#
# One [name] block per theme. Every field is optional — anything you leave out
# keeps the default theme's colour, so a two-line theme is perfectly valid.
#
# Colours are either:
#   #rrggbb          an exact colour, identical in every terminal
#   a colour name    black red green yellow blue magenta cyan white
#                    gray darkgray lightred lightgreen lightyellow
#                    lightblue lightmagenta lightcyan
#                    — these come from your terminal's own palette, so they
#                      follow whatever colour scheme you already run
#
# Naming a block after a built-in theme replaces it, which is how you retune
# `nord` rather than inventing `nord2`.
#
# Save and press r in the theme picker to reload.

[example]
text    = #c8d3f5   # correctly typed characters, and the clock
muted   = #a9b8e8   # menu rows
dim     = #3b4261   # untyped characters, labels, panel borders
error   = #ff757f   # a character that doesn't match
extra   = #ff98a4   # characters typed past the end of a word
accent  = #82aaff   # numbers, markers, panel titles
good    = #c3e88d   # a personal best
warn    = #ffc777   # warnings
banner  = #82aaff   # the ASCII art above the test
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::theme::built_ins;

    /// What a file's blocks start from: the default theme, so a field left
    /// out keeps its colour.
    fn base() -> Theme {
        built_ins()[0].clone()
    }

    // -- colours ----------------------------------------------------------

    #[test]
    fn colour_names_and_hex_both_resolve() {
        assert_eq!(colour("cyan"), Some(Color::Cyan));
        assert_eq!(colour("LightGreen"), Some(Color::LightGreen));
        assert_eq!(colour("grey"), Some(Color::Gray));
        assert_eq!(colour("#ff8800"), Some(Color::Rgb(255, 136, 0)));
        assert_eq!(colour("  #000000  "), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn nonsense_colours_dont_resolve() {
        assert_eq!(colour("chartreuse"), None);
        assert_eq!(colour("#fff"), None);
        assert_eq!(colour("#gggggg"), None);
        assert_eq!(colour("ff8800"), None);
        assert_eq!(colour(""), None);
    }

    // -- parsing ----------------------------------------------------------

    #[test]
    fn a_block_becomes_a_theme() {
        let (themes, skipped) = parse("[mine]\ntext = #010203\n", &base());

        assert_eq!(skipped, 0);
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "mine");
        assert_eq!(themes[0].text, Color::Rgb(1, 2, 3));
    }

    #[test]
    fn omitted_fields_keep_the_default() {
        let (themes, _) = parse("[mine]\ntext = #010203\n", &base());

        assert_eq!(themes[0].accent, base().accent);
        assert_eq!(themes[0].banner, base().banner);
    }

    #[test]
    fn the_palette_is_every_settable_colour_in_file_order() {
        let mut theme = base();
        theme.text = Color::Rgb(1, 0, 0);
        theme.banner = Color::Rgb(0, 0, 9);

        let palette = theme.palette();

        // One chip per field the picker could otherwise silently drop, and
        // `text` first / `banner` last, as the file lists them.
        assert_eq!(palette.len(), FIELDS.len());
        assert_eq!(palette[0], Color::Rgb(1, 0, 0));
        assert_eq!(palette[FIELDS.len() - 1], Color::Rgb(0, 0, 9));
    }

    #[test]
    fn reading_the_palette_leaves_the_theme_alone() {
        // It goes through accessors written for the parser's `&mut`.
        let theme = base();
        let before = theme.clone();
        let _ = theme.palette();

        assert_eq!(theme, before);
    }

    #[test]
    fn every_field_can_be_set() {
        let body: String = FIELDS
            .iter()
            .map(|(name, _)| format!("{name} = #010203\n"))
            .collect();
        let (themes, skipped) = parse(&format!("[mine]\n{body}"), &base());

        assert_eq!(skipped, 0);
        let mine = &themes[0];
        for (_, get) in FIELDS {
            assert_eq!(*get(&mut mine.clone()), Color::Rgb(1, 2, 3));
        }
    }

    #[test]
    fn several_blocks_become_several_themes() {
        let (themes, _) = parse("[one]\ntext = red\n\n[two]\ntext = blue\n", &base());

        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].text, Color::Red);
        assert_eq!(themes[1].text, Color::Blue);
    }

    #[test]
    fn a_comment_never_eats_a_hex_colour() {
        // `#` opens a comment and prefixes a colour; telling them apart is the
        // whole job of `strip_comment`.
        assert_eq!(strip_comment("text = #010203"), "text = #010203");
        assert_eq!(strip_comment("text = #010203  # note"), "text = #010203  ");
        assert_eq!(strip_comment("# just a comment"), "");
        assert_eq!(strip_comment("text = red # note"), "text = red ");
        assert_eq!(strip_comment("# ff8800 is orange"), "");
        assert_eq!(strip_comment("no comment here"), "no comment here");
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let (themes, skipped) = parse(
            "# a comment\n\n[mine]  # trailing comment\ntext = red # and here\n",
            &base(),
        );

        assert_eq!(skipped, 0);
        assert_eq!(themes[0].text, Color::Red);
    }

    #[test]
    fn junk_is_counted_not_fatal() {
        let (themes, skipped) = parse(
            "[mine]\ntext = red\nnonsense\nbogus = red\naccent = notacolour\n",
            &base(),
        );

        // The good line survived; the three bad ones were counted.
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].text, Color::Red);
        assert_eq!(skipped, 3);
    }

    #[test]
    fn a_field_before_any_block_is_skipped() {
        let (themes, skipped) = parse("text = red\n", &base());

        assert!(themes.is_empty());
        assert_eq!(skipped, 1);
    }

    #[test]
    fn an_unnamed_block_is_skipped() {
        let (themes, skipped) = parse("[]\ntext = red\n", &base());

        assert!(themes.is_empty());
        // The empty block, and then the field with nothing to belong to.
        assert_eq!(skipped, 2);
    }

    #[test]
    fn an_empty_file_defines_nothing() {
        assert_eq!(parse("", &base()), (Vec::new(), 0));
    }
}
