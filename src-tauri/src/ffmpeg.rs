use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Informações de uma faixa de legenda no vídeo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub index: usize,
    pub stream_index: usize,
    pub codec_name: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

/// Output do ffprobe em JSON
#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    index: usize,
    codec_name: Option<String>,
    codec_type: Option<String>,
    tags: Option<FfprobeTags>,
}

#[derive(Debug, Deserialize)]
struct FfprobeTags {
    language: Option<String>,
    title: Option<String>,
}

/// Lista faixas de legenda em um arquivo de vídeo
pub fn list_subtitle_tracks(video_path: &str) -> Result<Vec<SubtitleTrack>, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "s",
            "-show_entries",
            "stream=index,codec_name,codec_type:stream_tags=language,title",
            "-of",
            "json",
            video_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}. Is FFmpeg installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ffprobe_output: FfprobeOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let mut tracks = Vec::new();
    let mut sub_index = 0;

    for stream in ffprobe_output.streams {
        if stream.codec_type.as_deref() == Some("subtitle") {
            tracks.push(SubtitleTrack {
                index: sub_index,
                stream_index: stream.index,
                codec_name: stream.codec_name.unwrap_or_else(|| "unknown".to_string()),
                language: stream.tags.as_ref().and_then(|t| t.language.clone()),
                title: stream.tags.as_ref().and_then(|t| t.title.clone()),
            });
            sub_index += 1;
        }
    }

    Ok(tracks)
}

/// Extrai uma faixa de legenda do vídeo para arquivo
pub fn extract_subtitle_track(
    video_path: &str,
    track_index: usize,
    output_path: &str,
) -> Result<(), String> {
    // Detecta a extensão de saída para determinar o codec
    let output_ext = Path::new(output_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt")
        .to_lowercase();

    let codec = match output_ext.as_str() {
        "ass" | "ssa" => "ass",
        "srt" => "srt",
        "vtt" => "webvtt",
        _ => "copy", // Tenta manter original
    };

    let output = Command::new("ffmpeg")
        .args([
            "-y", // Sobrescreve arquivo existente
            "-i",
            video_path,
            "-map",
            &format!("0:s:{}", track_index),
            "-c:s",
            codec,
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}. Is FFmpeg installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg extraction failed: {}", stderr));
    }

    Ok(())
}

/// Adiciona uma faixa de legenda ao vídeo (mux)
pub fn mux_subtitle_track(
    video_path: &str,
    subtitle_path: &str,
    output_path: &str,
    language: Option<&str>,
    title: Option<&str>,
) -> Result<(), String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        video_path.to_string(),
        "-i".to_string(),
        subtitle_path.to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-map".to_string(),
        "0".to_string(),
        "-map".to_string(),
        "1".to_string(),
    ];

    // Adiciona metadados da faixa
    if let Some(lang) = language {
        args.push("-metadata:s:s:0".to_string());
        args.push(format!("language={}", lang));
    }

    if let Some(t) = title {
        args.push("-metadata:s:s:0".to_string());
        args.push(format!("title={}", t));
    }

    args.push(output_path.to_string());

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}. Is FFmpeg installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg muxing failed: {}", stderr));
    }

    Ok(())
}

/// Verifica se FFmpeg está instalado
pub fn check_ffmpeg() -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| {
            "FFmpeg not found. Please install FFmpeg and ensure it's in PATH.".to_string()
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.lines().next().unwrap_or("FFmpeg installed");
    Ok(version_line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_ffmpeg() {
        // Este teste só passa se FFmpeg estiver instalado
        match check_ffmpeg() {
            Ok(version) => {
                println!("FFmpeg version: {}", version);
                assert!(version.contains("ffmpeg"));
            }
            Err(e) => {
                println!("FFmpeg not installed: {}", e);
            }
        }
    }
}
