use std::path::{Path, PathBuf};

use ratatui::style::Color;

use crate::config::storage;

/// The theme a fresh install starts on, and the fallback for anything that
/// can't be resolved.
pub const DEFAULT: &str = "default";

/// Where a user's own themes live, inside [`storage::config_dir`].
const FILE: &str = "themes.conf";

/// Every colour the interface uses, named by its job rather than its hue.
///
/// Renderers ask for `accent` or `dim`, never for yellow or grey, which is
/// what lets a whole palette be swapped underneath them.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    /// Primary foreground: correctly typed characters, the clock.
    pub text: Color,
    /// Secondary foreground: menu rows.
    pub muted: Color,
    /// Tertiary foreground: untyped characters, labels, borders, key hints.
    pub dim: Color,
    /// A character that doesn't match the target.
    pub error: Color,
    /// A character typed past the end of a word.
    pub extra: Color,
    /// Numbers, markers and panel titles that should catch the eye.
    pub accent: Color,
    /// A personal best.
    pub good: Color,
    /// Something the user should know about but isn't broken.
    pub warn: Color,
    /// The art above the test.
    pub banner: Color,
    /// Came from the user's file rather than shipping with the app.
    pub custom: bool,
}

/// A settable colour: the name used in a theme file, and how to reach it.
type Field = (&'static str, fn(&mut Theme) -> &mut Color);

/// The fields a theme file may set.
///
/// One table rather than a `match` arm per field, so adding a colour to
/// [`Theme`] means adding exactly one line here and the parser follows.
const FIELDS: [Field; 9] = [
    ("text", |t| &mut t.text),
    ("muted", |t| &mut t.muted),
    ("dim", |t| &mut t.dim),
    ("error", |t| &mut t.error),
    ("extra", |t| &mut t.extra),
    ("accent", |t| &mut t.accent),
    ("good", |t| &mut t.good),
    ("warn", |t| &mut t.warn),
    ("banner", |t| &mut t.banner),
];

impl Theme {
    /// Every colour the theme sets, in the order a theme file lists them.
    ///
    /// Read through [`FIELDS`] — the same table the parser writes through — so
    /// a colour added to [`Theme`] shows up in the picker without anyone
    /// having to remember to put it there. The clone is what lets a read go
    /// through accessors written for the parser's `&mut`; against a list nine
    /// entries long, that is cheaper than a second table to keep in step.
    pub fn palette(&self) -> [Color; FIELDS.len()] {
        let mut theme = self.clone();

        FIELDS.map(|(_, field)| *field(&mut theme))
    }
}

/// Themes that ship with the app.
///
/// `default` is built from *named* terminal colours, so it inherits whatever
/// palette the user's terminal already uses. The rest are fixed RGB: they look
/// the same everywhere, which is the point of choosing one, but they ignore
/// the terminal's own theme.
pub(super) fn built_ins() -> Vec<Theme> {
    vec![
        Theme {
            name: DEFAULT.to_string(),
            text: Color::White,
            muted: Color::Gray,
            dim: Color::DarkGray,
            error: Color::Red,
            extra: Color::LightRed,
            accent: Color::Yellow,
            good: Color::LightGreen,
            warn: Color::LightRed,
            banner: Color::Green,
            custom: false,
        },
        Theme {
            name: "dracula".to_string(),
            text: Color::Rgb(248, 248, 242),
            muted: Color::Rgb(190, 190, 200),
            dim: Color::Rgb(98, 114, 164),
            error: Color::Rgb(255, 85, 85),
            extra: Color::Rgb(255, 121, 198),
            accent: Color::Rgb(189, 147, 249),
            good: Color::Rgb(80, 250, 123),
            warn: Color::Rgb(255, 184, 108),
            banner: Color::Rgb(139, 233, 253),
            custom: false,
        },
        Theme {
            name: "nord".to_string(),
            text: Color::Rgb(236, 239, 244),
            muted: Color::Rgb(216, 222, 233),
            dim: Color::Rgb(76, 86, 106),
            error: Color::Rgb(191, 97, 106),
            extra: Color::Rgb(208, 135, 112),
            accent: Color::Rgb(136, 192, 208),
            good: Color::Rgb(163, 190, 140),
            warn: Color::Rgb(235, 203, 139),
            banner: Color::Rgb(136, 192, 208),
            custom: false,
        },
        Theme {
            name: "gruvbox".to_string(),
            text: Color::Rgb(235, 219, 178),
            muted: Color::Rgb(189, 174, 147),
            dim: Color::Rgb(102, 92, 84),
            error: Color::Rgb(251, 73, 52),
            extra: Color::Rgb(254, 128, 25),
            accent: Color::Rgb(250, 189, 47),
            good: Color::Rgb(184, 187, 38),
            warn: Color::Rgb(215, 153, 33),
            banner: Color::Rgb(142, 192, 124),
            custom: false,
        },
        Theme {
            name: "matrix".to_string(),
            text: Color::Rgb(0, 255, 65),
            muted: Color::Rgb(0, 200, 50),
            dim: Color::Rgb(0, 95, 31),
            error: Color::Rgb(255, 51, 51),
            extra: Color::Rgb(255, 123, 123),
            accent: Color::Rgb(123, 255, 158),
            good: Color::Rgb(182, 255, 203),
            warn: Color::Rgb(255, 209, 102),
            banner: Color::Rgb(0, 255, 65),
            custom: false,
        },
    ]
}

/// Every theme available to the user: the built-ins, plus whatever their own
/// file defines.
pub struct Themes {
    all: Vec<Theme>,
    /// `None` for an in-memory instance, which makes editing impossible and
    /// keeps tests off the user's real file.
    path: Option<PathBuf>,
    /// How many of `all` came from the user's file.
    custom: usize,
    /// Lines of the file that meant nothing, so the picker can say so rather
    /// than leaving someone to wonder where their theme went.
    skipped: usize,
}

impl Themes {
    /// The built-ins, plus the user's file if it parses.
    ///
    /// Infallible: an unreadable or nonsense file means "no custom themes",
    /// never a failure to start.
    pub fn load() -> Self {
        let path = themes_path();
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        let mut themes = Self {
            all: built_ins(),
            path,
            custom: 0,
            skipped: 0,
        };
        themes.apply(&text);
        themes
    }

    /// The built-ins only, with nowhere to save to. For tests.
    #[cfg(test)]
    pub fn detached() -> Self {
        Self {
            all: built_ins(),
            path: None,
            custom: 0,
            skipped: 0,
        }
    }

    /// Replace the custom themes with whatever `text` defines.
    fn apply(&mut self, text: &str) {
        let base = self.get(DEFAULT).clone();
        let (custom, skipped) = parse(text, &base);

        self.all = built_ins();
        self.custom = custom.len();
        self.skipped = skipped;

        for theme in custom {
            // A custom theme may replace a built-in of the same name, which is
            // how someone retunes `nord` rather than inventing `nord2`.
            match self.all.iter().position(|old| old.name == theme.name) {
                Some(index) => self.all[index] = theme,
                None => self.all.push(theme),
            }
        }
    }

    /// Re-read the file, picking up anything just saved to it.
    pub fn reload(&mut self) {
        *self = Self::load_at(self.path.clone());
    }

    fn load_at(path: Option<PathBuf>) -> Self {
        let text = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default();

        let mut themes = Self {
            all: built_ins(),
            path,
            custom: 0,
            skipped: 0,
        };
        themes.apply(&text);
        themes
    }

    pub fn all(&self) -> &[Theme] {
        &self.all
    }

    pub fn custom_count(&self) -> usize {
        self.custom
    }

    pub fn skipped_lines(&self) -> usize {
        self.skipped
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The theme called `name`, or the default, or — if someone has replaced
    /// the default with something unnameable — whatever is first.
    ///
    /// Always returns something: an unreadable theme file must leave the app
    /// readable, not blank.
    pub fn get(&self, name: &str) -> &Theme {
        self.find(name)
            .or_else(|| self.find(DEFAULT))
            .unwrap_or(&self.all[0])
    }

    fn find(&self, name: &str) -> Option<&Theme> {
        self.all.iter().find(|theme| theme.name == name)
    }

    /// Where `name` sits in the list, or the first row.
    pub fn index_of(&self, name: &str) -> usize {
        self.all
            .iter()
            .position(|theme| theme.name == name)
            .unwrap_or(0)
    }

    /// Make sure there is a file to edit, seeded with a worked example.
    ///
    /// Seeded rather than empty so `$EDITOR` opens something that explains the
    /// format, instead of a blank buffer and a guess.
    pub fn seed(&self) -> std::io::Result<&Path> {
        let path = self
            .path
            .as_deref()
            .ok_or_else(|| std::io::Error::other("no data directory to save themes in"))?;

        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, EXAMPLE)?;
        }

        Ok(path)
    }
}

fn themes_path() -> Option<PathBuf> {
    Some(storage::config_dir()?.join(FILE))
}

mod format;

use format::{parse, EXAMPLE};

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Theme {
        built_ins()[0].clone()
    }

    #[test]
    fn the_default_theme_exists() {
        let themes = Themes::detached();

        assert_eq!(themes.all()[0].name, DEFAULT);
        assert_eq!(themes.get(DEFAULT).name, DEFAULT);
    }

    #[test]
    fn built_in_names_are_unique() {
        let themes = built_ins();
        for (index, theme) in themes.iter().enumerate() {
            let duplicate = themes.iter().skip(index + 1).any(|o| o.name == theme.name);
            assert!(!duplicate, "duplicate: {}", theme.name);
        }
    }

    #[test]
    fn an_unknown_theme_resolves_to_the_default() {
        let themes = Themes::detached();

        assert_eq!(themes.get("no-such-theme").name, DEFAULT);
        assert_eq!(themes.index_of("no-such-theme"), 0);
    }

    #[test]
    fn no_built_in_hides_text_against_its_own_dim() {
        for theme in built_ins() {
            assert_ne!(theme.text, theme.dim, "{}", theme.name);
            assert_ne!(theme.text, theme.error, "{}", theme.name);
            assert_ne!(theme.banner, theme.dim, "{}", theme.name);
        }
    }

    // -- the collection ---------------------------------------------------

    #[test]
    fn custom_themes_are_added_after_the_built_ins() {
        let mut themes = Themes::detached();
        let built_in_count = themes.all().len();
        themes.apply("[mine]\ntext = red\n");

        assert_eq!(themes.all().len(), built_in_count + 1);
        assert_eq!(themes.all().last().unwrap().name, "mine");
        assert_eq!(themes.custom_count(), 1);
    }

    #[test]
    fn a_custom_theme_replaces_a_built_in_of_the_same_name() {
        let mut themes = Themes::detached();
        let built_in_count = themes.all().len();
        themes.apply("[nord]\ntext = red\n");

        assert_eq!(themes.all().len(), built_in_count);
        assert_eq!(themes.get("nord").text, Color::Red);
    }

    #[test]
    fn reapplying_replaces_rather_than_accumulates() {
        let mut themes = Themes::detached();
        themes.apply("[mine]\ntext = red\n");
        themes.apply("[mine]\ntext = blue\n");

        assert_eq!(themes.custom_count(), 1);
        assert_eq!(themes.get("mine").text, Color::Blue);
    }

    #[test]
    fn removing_a_theme_from_the_file_removes_it_from_the_list() {
        let mut themes = Themes::detached();
        themes.apply("[mine]\ntext = red\n");
        themes.apply("");

        assert_eq!(themes.custom_count(), 0);
        assert_eq!(themes.get("mine").name, DEFAULT);
    }

    #[test]
    fn a_parsed_theme_is_marked_as_the_users() {
        let (themes, _) = parse("[mine]\ntext = red\n", &base());

        assert!(themes[0].custom);
        assert!(!base().custom);
    }

    #[test]
    fn skipped_lines_are_reported() {
        let mut themes = Themes::detached();
        themes.apply("[mine]\nbogus = red\n");

        assert_eq!(themes.skipped_lines(), 1);
    }

    #[test]
    fn a_detached_collection_cant_be_seeded() {
        assert!(Themes::detached().seed().is_err());
    }

    #[test]
    fn the_example_file_parses_cleanly() {
        // The seed file is the only documentation of the format, so it had
        // better be valid.
        let (themes, skipped) = parse(EXAMPLE, &base());

        assert_eq!(skipped, 0, "the example file has {skipped} bad lines");
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "example");
    }
}
