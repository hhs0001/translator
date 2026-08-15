use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Qualifier / organization / application used by `directories`.
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "translator";
const APPLICATION: &str = "translator";

/// Old Tauri identifier folder (sibling of the current app data dir).
const OLD_APP_DIR_NAME: &str = "com.translator.app";

/// Resolves and creates the application data directory.
pub fn app_data_dir() -> Result<PathBuf, String> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| "Failed to get app data dir".to_string())?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir)
}

pub fn settings_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("settings.json"))
}

pub fn templates_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("templates.json"))
}

/// Migrates `settings.json` and `templates.json` from the old
/// `com.translator.app` sibling folder when the new files are missing.
pub fn migrate_old_settings() {
    let current_dir = match app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };
    migrate_from_dir(&current_dir);
}

fn migrate_from_dir(current_dir: &Path) {
    let Some(parent) = current_dir.parent() else {
        return;
    };

    let old_dir = parent.join(OLD_APP_DIR_NAME);
    if !old_dir.exists() || old_dir == current_dir {
        return;
    }

    let old_settings = old_dir.join("settings.json");
    let new_settings = current_dir.join("settings.json");
    if old_settings.exists() && !new_settings.exists() {
        let _ = fs::copy(&old_settings, &new_settings);
    }

    let old_templates = old_dir.join("templates.json");
    let new_templates = current_dir.join("templates.json");
    if old_templates.exists() && !new_templates.exists() {
        let _ = fs::copy(&old_templates, &new_templates);
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
        let old_dir = root.join(OLD_APP_DIR_NAME);
        let new_dir = root.join("translator");
        fs::create_dir_all(&old_dir).unwrap();
        fs::create_dir_all(&new_dir).unwrap();

        fs::write(old_dir.join("settings.json"), r#"{"baseUrl":"http://old"}"#).unwrap();
        fs::write(old_dir.join("templates.json"), r#"{"templates":[]}"#).unwrap();

        migrate_from_dir(&new_dir);

        assert_eq!(
            fs::read_to_string(new_dir.join("settings.json")).unwrap(),
            r#"{"baseUrl":"http://old"}"#
        );
        assert_eq!(
            fs::read_to_string(new_dir.join("templates.json")).unwrap(),
            r#"{"templates":[]}"#
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn migrate_does_not_overwrite_existing_files() {
        let root = unique_temp_dir();
        let old_dir = root.join(OLD_APP_DIR_NAME);
        let new_dir = root.join("translator");
        fs::create_dir_all(&old_dir).unwrap();
        fs::create_dir_all(&new_dir).unwrap();

        fs::write(old_dir.join("settings.json"), "old").unwrap();
        fs::write(new_dir.join("settings.json"), "new").unwrap();

        migrate_from_dir(&new_dir);

        assert_eq!(
            fs::read_to_string(new_dir.join("settings.json")).unwrap(),
            "new"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
