use ratatui::style::Color;

/// Every colour the interface uses, named by its job rather than its hue.
///
/// Renderers ask for `accent` or `dim`, never for yellow or grey, which is
/// what lets a whole palette be swapped underneath them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub name: &'static str,
    /// Primary foreground: correctly typed characters, the clock.
    pub text: Color,
    /// Secondary foreground: menu rows.
    pub muted: Color,
    /// Tertiary foreground: untyped characters, field labels, key hints.
    pub dim: Color,
    /// A character that doesn't match the target.
    pub error: Color,
    /// A character typed past the end of a word.
    pub extra: Color,
    /// Numbers and markers that should catch the eye.
    pub accent: Color,
    /// A personal best.
    pub good: Color,
    /// Something the user should know about but isn't broken.
    pub warn: Color,
    /// The art above the test. Its own field rather than a reuse of `accent`:
    /// a big block of art wants a weight of its own, not the colour picked for
    /// small numbers and markers.
    pub banner: Color,
}

/// The theme a fresh install starts on.
pub const DEFAULT: &str = "default";

/// Themes offered in the picker.
///
/// `default` is built from *named* terminal colours, so it inherits whatever
/// palette the user's terminal already uses. The rest are fixed RGB: they look
/// the same everywhere, which is the point of choosing one, but they ignore
/// the terminal's own theme.
pub const THEMES: [Theme; 5] = [
    Theme {
        name: DEFAULT,
        text: Color::White,
        muted: Color::Gray,
        dim: Color::DarkGray,
        error: Color::Red,
        extra: Color::LightRed,
        accent: Color::Yellow,
        good: Color::LightGreen,
        warn: Color::LightRed,
        banner: Color::Green,
    },
    Theme {
        name: "dracula",
        text: Color::Rgb(248, 248, 242),
        muted: Color::Rgb(190, 190, 200),
        dim: Color::Rgb(98, 114, 164),
        error: Color::Rgb(255, 85, 85),
        extra: Color::Rgb(255, 121, 198),
        accent: Color::Rgb(189, 147, 249),
        good: Color::Rgb(80, 250, 123),
        warn: Color::Rgb(255, 184, 108),
        banner: Color::Rgb(139, 233, 253),
    },
    Theme {
        name: "nord",
        text: Color::Rgb(236, 239, 244),
        muted: Color::Rgb(216, 222, 233),
        dim: Color::Rgb(76, 86, 106),
        error: Color::Rgb(191, 97, 106),
        extra: Color::Rgb(208, 135, 112),
        accent: Color::Rgb(136, 192, 208),
        good: Color::Rgb(163, 190, 140),
        warn: Color::Rgb(235, 203, 139),
        banner: Color::Rgb(136, 192, 208),
    },
    Theme {
        name: "gruvbox",
        text: Color::Rgb(235, 219, 178),
        muted: Color::Rgb(189, 174, 147),
        dim: Color::Rgb(102, 92, 84),
        error: Color::Rgb(251, 73, 52),
        extra: Color::Rgb(254, 128, 25),
        accent: Color::Rgb(250, 189, 47),
        good: Color::Rgb(184, 187, 38),
        warn: Color::Rgb(215, 153, 33),
        banner: Color::Rgb(142, 192, 124),
    },
    Theme {
        name: "matrix",
        text: Color::Rgb(0, 255, 65),
        muted: Color::Rgb(0, 200, 50),
        dim: Color::Rgb(0, 95, 31),
        error: Color::Rgb(255, 51, 51),
        extra: Color::Rgb(255, 123, 123),
        accent: Color::Rgb(123, 255, 158),
        good: Color::Rgb(182, 255, 203),
        warn: Color::Rgb(255, 209, 102),
        banner: Color::Rgb(0, 255, 65),
    },
];

pub fn by_name(name: &str) -> Option<&'static Theme> {
    THEMES.iter().find(|theme| theme.name == name)
}

/// The theme called `name`, or the default.
///
/// Falls back rather than failing: a theme name from a newer version, or a
/// typo in the settings file, leaves the app readable instead of blank.
pub fn resolve(name: &str) -> &'static Theme {
    by_name(name).unwrap_or_else(|| by_name(DEFAULT).expect("the default theme always exists"))
}

/// Where `name` sits in [`THEMES`], or the default's position.
pub fn index_of(name: &str) -> usize {
    THEMES
        .iter()
        .position(|theme| theme.name == name)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_theme_exists() {
        assert!(by_name(DEFAULT).is_some());
        assert_eq!(THEMES[0].name, DEFAULT);
    }

    #[test]
    fn theme_names_are_unique() {
        for (index, theme) in THEMES.iter().enumerate() {
            let duplicate = THEMES
                .iter()
                .skip(index + 1)
                .any(|other| other.name == theme.name);
            assert!(!duplicate, "duplicate theme name: {}", theme.name);
        }
    }

    #[test]
    fn an_unknown_theme_resolves_to_the_default() {
        assert_eq!(resolve("no-such-theme").name, DEFAULT);
        assert_eq!(resolve("").name, DEFAULT);
        assert_eq!(index_of("no-such-theme"), 0);
    }

    #[test]
    fn a_known_theme_resolves_to_itself() {
        for (index, theme) in THEMES.iter().enumerate() {
            assert_eq!(resolve(theme.name), theme);
            assert_eq!(index_of(theme.name), index);
        }
    }

    #[test]
    fn every_theme_gives_the_banner_a_colour_of_its_own() {
        // Not required to be unique, but it must not be the background-ish
        // dim, or the art would be all but invisible.
        for theme in THEMES {
            assert_ne!(theme.banner, theme.dim, "{}", theme.name);
        }
    }

    #[test]
    fn no_theme_hides_text_against_its_own_dim() {
        // The states a typing test has to keep apart at a glance.
        for theme in THEMES {
            assert_ne!(theme.text, theme.dim, "{}", theme.name);
            assert_ne!(theme.text, theme.error, "{}", theme.name);
            assert_ne!(theme.error, theme.dim, "{}", theme.name);
        }
    }
}
