use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::subtitle::{SubtitleFile, SubtitleFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size: u64,
    pub is_video: bool,
    pub is_subtitle: bool,
}

pub fn is_video(extension: &str) -> bool {
    matches!(
        normalize_extension(extension).as_str(),
        "mkv" | "mp4" | "avi" | "mov" | "webm" | "m4v" | "ts"
    )
}

pub fn is_subtitle(extension: &str) -> bool {
    matches!(
        normalize_extension(extension).as_str(),
        "srt" | "ass" | "ssa" | "vtt"
    )
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_lowercase()
}

pub fn get_file_info(path: impl AsRef<Path>) -> Result<FileInfo, String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    Ok(FileInfo {
        path: path.to_string_lossy().to_string(),
        filename,
        extension: extension.clone(),
        size: metadata.len(),
        is_video: is_video(&extension),
        is_subtitle: is_subtitle(&extension),
    })
}

pub fn delete_files(paths: &[String]) -> Result<Vec<String>, String> {
    let mut deleted = Vec::new();
    let mut errors = Vec::new();

    for path in paths {
        match fs::remove_file(path) {
            Ok(_) => deleted.push(path.clone()),
            Err(e) => errors.push(format!("{}: {}", path, e)),
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Some files failed to delete: {}",
            errors.join(", ")
        ));
    }

    Ok(deleted)
}

pub fn backup_file(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    let backup_path = format!("{}.bak", path.to_string_lossy());
    fs::copy(path, &backup_path).map_err(|e| format!("Failed to backup file: {}", e))?;
    Ok(backup_path)
}

pub fn replace_file(
    source_path: impl AsRef<Path>,
    target_path: impl AsRef<Path>,
) -> Result<(), String> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();

    let backup_path = format!("{}.bak", target_path.to_string_lossy());
    fs::copy(target_path, &backup_path).map_err(|e| format!("Failed to backup original: {}", e))?;

    fs::remove_file(target_path).map_err(|e| format!("Failed to remove original: {}", e))?;

    fs::rename(source_path, target_path).map_err(|e| format!("Failed to rename file: {}", e))?;

    Ok(())
}

pub fn open_folder(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err("Opening folders is not supported on this platform".to_string());
    }

    Ok(())
}

pub fn detect_subtitle_format(filename: &str) -> Option<String> {
    SubtitleFile::detect_format(filename).map(|f| match f {
        SubtitleFormat::Srt => "srt".to_string(),
        SubtitleFormat::Ass => "ass".to_string(),
        SubtitleFormat::Ssa => "ssa".to_string(),
        SubtitleFormat::Vtt => "vtt".to_string(),
    })
}

pub fn load_subtitle(path: impl AsRef<Path>) -> Result<SubtitleFile, String> {
    let path = path.as_ref();
    let content = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let (content, _, _) = encoding_rs::UTF_8.decode(&content);
    let content = content.to_string();

    let path_str = path.to_string_lossy();
    let format = SubtitleFile::detect_format(&path_str)
        .ok_or_else(|| "Unknown subtitle format".to_string())?;

    SubtitleFile::parse(&content, format)
}

pub fn save_subtitle(path: impl AsRef<Path>, file: &SubtitleFile) -> Result<(), String> {
    let content = file.serialize();
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
}

pub fn analyze_subtitle_clutter(file: &SubtitleFile) -> super::text_cleaner::AssClutterAnalysis {
    let entries: Vec<(String, Option<String>)> = file
        .entries
        .iter()
        .map(|e| {
            (
                e.text.clone(),
                e.metadata.as_ref().and_then(|m| m.style.clone()),
            )
        })
        .collect();
    super::text_cleaner::analyze_ass_clutter(&entries)
}

/// `(index, original, cleaned, should_skip)`
pub fn preview_cleaned_text(
    file: &SubtitleFile,
    config: &super::text_cleaner::TextCleanerConfig,
) -> Vec<(usize, String, String, bool)> {
    let entries: Vec<(usize, String, Option<String>)> = file
        .entries
        .iter()
        .map(|e| {
            (
                e.index,
                e.text.clone(),
                e.metadata.as_ref().and_then(|m| m.style.clone()),
            )
        })
        .collect();

    let cleaned = super::text_cleaner::clean_subtitle_entries(&entries, config);
    cleaned
        .mappings
        .iter()
        .map(|m| {
            (
                m.entry_index,
                m.original_text.clone(),
                m.clean_text.clone(),
                m.should_skip_translation,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn classifies_video_and_subtitle_extensions() {
        assert!(is_video("mkv"));
        assert!(is_video(".MP4"));
        assert!(is_video("webm"));
        assert!(!is_video("srt"));

        assert!(is_subtitle("srt"));
        assert!(is_subtitle("ASS"));
        assert!(is_subtitle(".vtt"));
        assert!(!is_subtitle("mp4"));
    }

    #[test]
    fn get_file_info_and_backup_replace_delete() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("translator-files-{nanos}"));
        fs::create_dir_all(&dir).unwrap();

        let target = dir.join("clip.srt");
        let source = dir.join("new.srt");
        fs::write(&target, "original").unwrap();
        fs::write(&source, "replacement").unwrap();

        let info = get_file_info(&target).unwrap();
        assert!(info.is_subtitle);
        assert!(!info.is_video);
        assert_eq!(info.extension, "srt");
        assert_eq!(info.size, 8);

        let bak = backup_file(&target).unwrap();
        assert!(Path::new(&bak).exists());

        replace_file(&source, &target).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "replacement");
        assert!(dir.join("clip.srt.bak").exists());

        let deleted = delete_files(&[bak]).unwrap();
        assert_eq!(deleted.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_save_subtitle_roundtrip() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("translator-sub-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.srt");

        let srt = "1\n00:00:00,000 --> 00:00:01,000\nHello\n\n";
        fs::write(&path, srt).unwrap();

        let file = load_subtitle(&path).unwrap();
        assert_eq!(file.entries.len(), 1);
        assert_eq!(file.entries[0].text, "Hello");

        let out = dir.join("out.srt");
        save_subtitle(&out, &file).unwrap();
        assert!(out.exists());

        let _ = fs::remove_dir_all(&dir);
    }
}
