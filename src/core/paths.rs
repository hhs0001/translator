use std::fs;
use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};

/// Qualifier / organization / application used by `directories`.
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "translator";
const APPLICATION: &str = "translator";

/// App-data folder names used by the previous Tauri builds. Those stored their
/// files directly under the platform's app-data root, named after the bundle
/// identifier — not under an org/app pair like `directories` does.
const LEGACY_APP_DIRS: &[&str] = &[
    "com.translator",
    "com.translator.app",
    "com.heito.subtitle-translator",
];

const SETTINGS_FILE: &str = "settings.json";
const TEMPLATES_FILE: &str = "templates.json";

/// Overrides the data directory — handy for portable installs and for testing
/// against a throwaway configuration.
const DATA_DIR_ENV: &str = "TRANSLATOR_DATA_DIR";

/// Resolves and creates the application data directory.
pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = match std::env::var(DATA_DIR_ENV) {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => {
            let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
                .ok_or_else(|| "Failed to get app data dir".to_string())?;
            dirs.data_dir().to_path_buf()
        }
    };
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir)
}

pub fn settings_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join(SETTINGS_FILE))
}

pub fn templates_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join(TEMPLATES_FILE))
}

/// Every directory a previous version could have written to.
fn legacy_dirs() -> Vec<PathBuf> {
    let Some(base) = BaseDirs::new() else {
        return Vec::new();
    };

    // Roots the Tauri builds used, per platform.
    let mut roots: Vec<PathBuf> = vec![base.data_dir().to_path_buf()];
    let config_dir = base.config_dir().to_path_buf();
    if !roots.contains(&config_dir) {
        roots.push(config_dir);
    }
    if let Some(local) = base.data_local_dir().to_str() {
        let local = PathBuf::from(local);
        if !roots.contains(&local) {
            roots.push(local);
        }
    }

    let mut dirs = Vec::new();
    for root in roots {
        for name in LEGACY_APP_DIRS {
            let candidate = root.join(name);
            if candidate.is_dir() && !dirs.contains(&candidate) {
                dirs.push(candidate);
            }
        }
    }
    dirs
}

/// Copies `settings.json` / `templates.json` over from an older install when
/// the current ones are missing (or still untouched defaults).
pub fn migrate_old_settings() {
    let Ok(current_dir) = app_data_dir() else {
        return;
    };
    migrate_from_dirs(&current_dir, &legacy_dirs());
}

/// Picks the most recently modified candidate file.
fn newest(candidates: &[PathBuf], file: &str) -> Option<PathBuf> {
    candidates
        .iter()
        .map(|dir| dir.join(file))
        .filter(|path| path.is_file())
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        })
}

/// True when the file is absent or holds nothing but defaults, i.e. the user
/// has not configured this install yet and importing is safe.
fn is_unconfigured(path: &Path, file: &str) -> bool {
    if !path.exists() {
        return true;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };

    match file {
        SETTINGS_FILE => {
            let default = serde_json::to_value(super::settings::AppSettings::default())
                .unwrap_or(serde_json::Value::Null);
            value == default
        }
        TEMPLATES_FILE => value
            .get("templates")
            .and_then(|templates| templates.as_array())
            .is_none_or(|templates| templates.is_empty()),
        _ => false,
    }
}

pub(crate) fn migrate_from_dirs(current_dir: &Path, legacy_dirs: &[PathBuf]) {
    let legacy: Vec<PathBuf> = legacy_dirs
        .iter()
        .filter(|dir| dir.as_path() != current_dir)
        .cloned()
        .collect();
    if legacy.is_empty() {
        return;
    }

    for file in [SETTINGS_FILE, TEMPLATES_FILE] {
        let target = current_dir.join(file);
        if !is_unconfigured(&target, file) {
            continue;
        }
        let Some(source) = newest(&legacy, file) else {
            continue;
        };
        if target.exists() {
            let _ = fs::copy(&target, target.with_extension("json.bak"));
        }
        let _ = fs::copy(&source, &target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("translator-paths-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn migrate_copies_missing_settings_and_templates() {
        let root = unique_temp_dir();
        let current = root.join("translator/translator/data");
        let legacy = root.join("com.translator");
        fs::create_dir_all(&current).unwrap();
        fs::create_dir_all(&legacy).unwrap();

        fs::write(legacy.join("settings.json"), r#"{"baseUrl":"http://old"}"#).unwrap();
        fs::write(
            legacy.join("templates.json"),
            r#"{"templates":[{"id":"1"}]}"#,
        )
        .unwrap();

        migrate_from_dirs(&current, &[legacy]);

        assert!(fs::read_to_string(current.join("settings.json"))
            .unwrap()
            .contains("http://old"));
        assert!(fs::read_to_string(current.join("templates.json"))
            .unwrap()
            .contains("\"id\":\"1\""));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn migrate_keeps_configured_files() {
        let root = unique_temp_dir();
        let current = root.join("data");
        let legacy = root.join("com.translator");
        fs::create_dir_all(&current).unwrap();
        fs::create_dir_all(&legacy).unwrap();

        fs::write(
            current.join("settings.json"),
            r#"{"baseUrl":"http://mine"}"#,
        )
        .unwrap();
        fs::write(legacy.join("settings.json"), r#"{"baseUrl":"http://old"}"#).unwrap();

        migrate_from_dirs(&current, &[legacy]);

        assert!(fs::read_to_string(current.join("settings.json"))
            .unwrap()
            .contains("http://mine"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn migrate_replaces_untouched_defaults() {
        let root = unique_temp_dir();
        let current = root.join("data");
        let legacy = root.join("com.translator");
        fs::create_dir_all(&current).unwrap();
        fs::create_dir_all(&legacy).unwrap();

        // A freshly created config that the user never changed.
        let defaults =
            serde_json::to_string_pretty(&super::super::settings::AppSettings::default()).unwrap();
        fs::write(current.join("settings.json"), defaults).unwrap();
        fs::write(
            legacy.join("settings.json"),
            r#"{"baseUrl":"http://old","model":"gpt-old"}"#,
        )
        .unwrap();

        migrate_from_dirs(&current, &[legacy]);

        let migrated = fs::read_to_string(current.join("settings.json")).unwrap();
        assert!(
            migrated.contains("gpt-old"),
            "legacy settings were imported"
        );
        assert!(
            current.join("settings.json.bak").exists(),
            "the replaced file is kept as a backup"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn migrate_prefers_the_newest_legacy_file() {
        let root = unique_temp_dir();
        let current = root.join("data");
        let older = root.join("com.translator.app");
        let newer = root.join("com.translator");
        fs::create_dir_all(&current).unwrap();
        fs::create_dir_all(&older).unwrap();
        fs::create_dir_all(&newer).unwrap();

        fs::write(older.join("settings.json"), r#"{"model":"older"}"#).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        fs::write(newer.join("settings.json"), r#"{"model":"newer"}"#).unwrap();

        migrate_from_dirs(&current, &[older, newer]);

        assert!(fs::read_to_string(current.join("settings.json"))
            .unwrap()
            .contains("newer"));

        let _ = fs::remove_dir_all(&root);
    }
}
