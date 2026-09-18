use std::path::{Path, PathBuf};

use crate::config::storage;

/// The art shipped with the app, used until the user supplies their own.
const BUILT_IN: &str = include_str!("../../assets/wasabi.txt");

/// What a user's own art is called, inside [`storage::config_dir`].
const FILE: &str = "banner.txt";

/// The art drawn above the test.
pub struct Banner {
    /// Normalised: no `\r`, no trailing blank line.
    art: String,
    /// Whether `art` came from the user's file rather than the built-in.
    custom: bool,
    /// Where a custom banner is read from and written to. `None` when there is
    /// nowhere to persist to, which makes editing impossible but drawing fine.
    path: Option<PathBuf>,
}

impl Banner {
    /// The user's art if there is any, otherwise the built-in.
    ///
    /// Infallible, like the records: an unreadable banner file means "no
    /// custom art", never a failure to start.
    pub fn load() -> Self {
        let path = banner_path();

        let custom = path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            // A file of nothing but whitespace is someone who changed their
            // mind, not art. Treat it as absent so the app isn't left with an
            // invisible banner and no clue why.
            .filter(|text| !text.trim().is_empty());

        match custom {
            Some(art) => Self {
                art: normalise(&art),
                custom: true,
                path,
            },
            None => Self {
                art: normalise(BUILT_IN),
                custom: false,
                path,
            },
        }
    }

    pub fn art(&self) -> &str {
        &self.art
    }

    pub fn is_custom(&self) -> bool {
        self.custom
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Rows the banner occupies.
    pub fn height(&self) -> u16 {
        self.art.lines().count() as u16
    }

    /// Columns the widest line occupies.
    ///
    /// `chars().count()` is the display width for the built-in art, whose
    /// braille patterns (U+2800..) are all single-width. Art using wide CJK or
    /// emoji will measure narrower than it draws, and wrap.
    pub fn width(&self) -> u16 {
        self.art
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16
    }

    /// Make sure there is a file to edit, seeded with the current art.
    ///
    /// Seeded rather than empty so `$EDITOR` opens something to modify, and so
    /// the built-in is there as a worked example of the expected shape.
    pub fn seed(&self) -> std::io::Result<&Path> {
        let path = self
            .path
            .as_deref()
            .ok_or_else(|| std::io::Error::other("no data directory to save a banner in"))?;

        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, &self.art)?;
        }

        Ok(path)
    }

    /// A banner with the built-in art and nowhere to save to.
    ///
    /// For tests: `reset` and `load` on one of these can't reach the user's
    /// real banner file.
    #[cfg(test)]
    pub fn detached() -> Self {
        Self {
            art: normalise(BUILT_IN),
            custom: false,
            path: None,
        }
    }

    /// Delete the user's art and go back to the built-in.
    ///
    /// Sets the built-in directly rather than reloading: with the file gone
    /// that is what a reload would return anyway, and reloading would make a
    /// banner with no path read from the real one.
    pub fn reset(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                // Already gone is the state we wanted anyway.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }

        self.art = normalise(BUILT_IN);
        self.custom = false;
        Ok(())
    }
}

fn banner_path() -> Option<PathBuf> {
    Some(storage::config_dir()?.join(FILE))
}

/// Strip `\r` and any trailing blank line, so a file saved on Windows or with
/// a trailing newline measures and draws the same as one that isn't.
fn normalise(text: &str) -> String {
    text.lines().collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A banner over `art` with nowhere to save to, so tests never touch the
    /// user's real banner file.
    fn banner(art: &str) -> Banner {
        Banner {
            art: normalise(art),
            custom: true,
            path: None,
        }
    }

    #[test]
    fn size_is_measured_from_the_art() {
        let banner = banner("abc\nde\nf");

        assert_eq!(banner.height(), 3);
        assert_eq!(banner.width(), 3);
    }

    #[test]
    fn a_trailing_newline_is_not_a_row() {
        assert_eq!(banner("ab\ncd\n").height(), 2);
    }

    #[test]
    fn windows_line_endings_dont_widen_the_art() {
        let banner = banner("ab\r\ncd\r\n");

        assert_eq!(banner.width(), 2);
        assert_eq!(banner.art(), "ab\ncd");
    }

    #[test]
    fn width_follows_the_longest_line() {
        assert_eq!(banner("a\nlonger\nb").width(), 6);
    }

    #[test]
    fn empty_art_has_no_size() {
        let banner = banner("");

        assert_eq!(banner.height(), 0);
        assert_eq!(banner.width(), 0);
    }

    #[test]
    fn the_built_in_art_is_rectangular() {
        // Every line the same width, or the banner would look ragged once
        // centred. Guards the asset, not the code.
        let banner = Banner::detached();

        let widths: Vec<usize> = banner.art.lines().map(|l| l.chars().count()).collect();
        assert!(
            widths.iter().all(|width| *width == widths[0]),
            "ragged widths: {widths:?}"
        );
        assert_eq!(banner.height(), 20);
        assert_eq!(banner.width(), 41);
    }

    #[test]
    fn a_banner_with_nowhere_to_save_cant_be_seeded() {
        assert!(banner("art").seed().is_err());
    }

    #[test]
    fn resetting_a_detached_banner_is_harmless() {
        // No path, so there is no file to remove, nothing to fail, and — the
        // part that regressed once — nothing read from the real banner file.
        let mut banner = banner("MY ART");
        assert!(banner.is_custom());

        assert!(banner.reset().is_ok());
        assert!(!banner.is_custom());
        assert_eq!(banner.art(), normalise(BUILT_IN));
    }
}
