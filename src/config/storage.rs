use std::path::{Path, PathBuf};

/// The directory name used under each XDG base directory.
const APP: &str = "wasabi";

/// Files that older versions kept in the data directory, and that now belong
/// in the config directory.
const MOVED: [&str; 3] = ["settings.tsv", "themes.conf", "banner.txt"];

/// `$XDG_BASE/wasabi`, or `$HOME/<fallback>/wasabi`.
///
/// `None` when neither is set, which every caller treats as "this session has
/// nowhere to persist to" rather than as an error.
fn base(variable: &str, fallback: &str) -> Option<PathBuf> {
    let base = std::env::var_os(variable)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))?;

    Some(base.join(APP))
}

/// Where things the user writes live: preferences, themes, banner art.
///
/// `$XDG_CONFIG_HOME/wasabi`, falling back to `~/.config/wasabi`.
pub fn config_dir() -> Option<PathBuf> {
    base("XDG_CONFIG_HOME", ".config")
}

/// Where things the app writes live: records.
///
/// `$XDG_DATA_HOME/wasabi`, falling back to `~/.local/share/wasabi`. Separate
/// from [`config_dir`] because losing your personal bests is losing history,
/// not losing a setting, and the two deserve different backup habits.
pub fn data_dir() -> Option<PathBuf> {
    base("XDG_DATA_HOME", ".local/share")
}

/// Move config files left behind by an older version into the config
/// directory.
///
/// Called once at startup, before anything reads them. Best-effort and quiet:
/// a file that can't be moved is left exactly where it is and keeps working
/// from nowhere — better a missed migration than a lost theme.
pub fn migrate() {
    let (Some(data), Some(config)) = (data_dir(), config_dir()) else {
        return;
    };

    migrate_between(&data, &config);
}

fn migrate_between(data: &Path, config: &Path) {
    if data == config || !data.is_dir() {
        return;
    }

    for name in MOVED {
        let from = data.join(name);
        let to = config.join(name);

        // Nothing to move, or the new location already has one — never
        // overwrite a file the user has already made in the right place.
        if !from.is_file() || to.exists() {
            continue;
        }

        if std::fs::create_dir_all(config).is_err() {
            return;
        }

        // `rename` is atomic but fails across filesystems, which `~/.config`
        // and `~/.local/share` can easily be on.
        if std::fs::rename(&from, &to).is_err() && std::fs::copy(&from, &to).is_ok() {
            let _ = std::fs::remove_file(&from);
        }
    }
}

/// A path with the home directory written as `~`, for display only.
pub fn tilde(path: &Path) -> String {
    let home = std::env::var_os("HOME").map(PathBuf::from);

    match home.and_then(|home| path.strip_prefix(home).ok().map(PathBuf::from)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A pair of empty directories, unique to this test.
    ///
    /// Real directories rather than mocks, because the thing under test is
    /// whether files actually move.
    fn dirs() -> (PathBuf, PathBuf) {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);

        let root = std::env::temp_dir().join(format!("wasabi-migrate-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        let (data, config) = (root.join("data"), root.join("config"));
        std::fs::create_dir_all(&data).unwrap();
        (data, config)
    }

    fn write(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(name), body).unwrap();
    }

    fn read(dir: &Path, name: &str) -> Option<String> {
        std::fs::read_to_string(dir.join(name)).ok()
    }

    #[test]
    fn config_files_move_and_the_originals_go() {
        let (data, config) = dirs();
        write(&data, "settings.tsv", "theme\tnord\n");
        write(&data, "themes.conf", "[mine]\n");
        write(&data, "banner.txt", "art\n");

        migrate_between(&data, &config);

        assert_eq!(
            read(&config, "settings.tsv").as_deref(),
            Some("theme\tnord\n")
        );
        assert_eq!(read(&config, "themes.conf").as_deref(), Some("[mine]\n"));
        assert_eq!(read(&config, "banner.txt").as_deref(), Some("art\n"));
        assert_eq!(read(&data, "settings.tsv"), None);
    }

    #[test]
    fn records_stay_in_the_data_directory() {
        let (data, config) = dirs();
        write(&data, "records.tsv", "30\t80.0\t97.0\t1\t2\n");

        migrate_between(&data, &config);

        assert!(read(&data, "records.tsv").is_some());
        assert_eq!(read(&config, "records.tsv"), None);
    }

    #[test]
    fn a_file_already_in_the_new_place_is_never_overwritten() {
        let (data, config) = dirs();
        write(&data, "settings.tsv", "theme\told\n");
        write(&config, "settings.tsv", "theme\tnew\n");

        migrate_between(&data, &config);

        // The one the user already has wins, and the stale copy is left
        // rather than silently deleted.
        assert_eq!(
            read(&config, "settings.tsv").as_deref(),
            Some("theme\tnew\n")
        );
        assert_eq!(read(&data, "settings.tsv").as_deref(), Some("theme\told\n"));
    }

    #[test]
    fn migrating_twice_changes_nothing_the_second_time() {
        let (data, config) = dirs();
        write(&data, "settings.tsv", "theme\tnord\n");

        migrate_between(&data, &config);
        migrate_between(&data, &config);

        assert_eq!(
            read(&config, "settings.tsv").as_deref(),
            Some("theme\tnord\n")
        );
    }

    #[test]
    fn nothing_to_move_is_not_an_error() {
        let (data, config) = dirs();
        migrate_between(&data, &config);

        assert!(
            !config.exists(),
            "an empty migration shouldn't create the directory"
        );
    }

    #[test]
    fn a_missing_data_directory_is_not_an_error() {
        let (data, config) = dirs();
        std::fs::remove_dir_all(&data).unwrap();

        migrate_between(&data, &config);
        assert!(!config.exists());
    }

    #[test]
    fn identical_directories_are_left_alone() {
        // Someone with XDG_CONFIG_HOME and XDG_DATA_HOME pointing at the same
        // place: moving a file onto itself must not delete it.
        let (data, _) = dirs();
        write(&data, "settings.tsv", "theme\tnord\n");

        migrate_between(&data, &data);

        assert_eq!(
            read(&data, "settings.tsv").as_deref(),
            Some("theme\tnord\n")
        );
    }
}
