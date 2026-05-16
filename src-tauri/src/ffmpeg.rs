use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String>;
}

pub struct CommandOutput {
    pub status: std::process::ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    pub fn success() -> Self {
        CommandOutput {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    pub fn success_with_stdout(stdout: Vec<u8>) -> Self {
        CommandOutput {
            status: std::process::ExitStatus::from_raw(0),
            stdout,
            stderr: Vec::new(),
        }
    }

    pub fn failure(stderr: Vec<u8>) -> Self {
        CommandOutput {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr,
        }
    }
}

struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String> {
        create_command(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run {}: {}", program, e))
            .map(|output| CommandOutput {
                status: output.status,
                stdout: output.stdout,
                stderr: output.stderr,
            })
    }
}

static COMMAND_RUNNER: once_cell::sync::Lazy<std::sync::RwLock<Box<dyn CommandRunner>>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(Box::new(RealCommandRunner)));

pub fn set_command_runner(runner: Box<dyn CommandRunner>) {
    *COMMAND_RUNNER.write().unwrap() = runner;
}

pub fn reset_command_runner() {
    *COMMAND_RUNNER.write().unwrap() = Box::new(RealCommandRunner);
}

fn run_command(program: &str, args: &[&str]) -> Result<CommandOutput, String> {
    COMMAND_RUNNER.read().unwrap().run(program, args)
}

/// Cria um Command que não abre janela de terminal no Windows
fn create_command(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

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
    let output = run_command("ffprobe", &["-v", "error", "-select_streams", "s", "-show_entries", "stream=index,codec_name,codec_type:stream_tags=language,title", "-of", "json", video_path])?;

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
    let output_ext = Path::new(output_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt")
        .to_lowercase();

    let codec = match output_ext.as_str() {
        "ass" | "ssa" => "ass",
        "srt" => "srt",
        "vtt" => "webvtt",
        _ => "copy",
    };

    let args = &["-y", "-i", video_path, "-map", &format!("0:s:{}", track_index), "-c:s", codec, output_path];

    let output = run_command("ffmpeg", args)?;

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
    let mut args: Vec<&str> = vec![
        "-y",
        "-i",
        video_path,
        "-i",
        subtitle_path,
        "-map",
        "0:v",
        "-map",
        "0:a?",
        "-map",
        "1:s",
        "-map",
        "0:s?",
        "-c:v",
        "copy",
        "-c:a",
        "copy",
        "-c:s:0",
        "ass",
        "-c:s",
        "copy",
        "-disposition:s:0",
        "default",
    ];

    if let Some(lang) = language {
        args.push("-metadata:s:s:0");
        args.push(&format!("language={}", lang));
    }

    if let Some(t) = title {
        args.push("-metadata:s:s:0");
        args.push(&format!("title={}", t));
    }

    args.push(output_path);

    let output = run_command("ffmpeg", &args)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg muxing failed: {}", stderr));
    }

    Ok(())
}

/// Verifica se FFmpeg está instalado
pub fn check_ffmpeg() -> Result<String, String> {
    let output = run_command("ffmpeg", &["-version"])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_line = stdout.lines().next().unwrap_or("FFmpeg installed");
    Ok(version_line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockCommandRunner {
        responses: Mutex<HashMap<(String, Vec<String>), Result<CommandOutput, String>>>,
    }

    impl MockCommandRunner {
        fn new() -> Self {
            MockCommandRunner {
                responses: Mutex::new(HashMap::new()),
            }
        }

        fn add_response(
            &self,
            program: &str,
            args: &[&str],
            response: Result<CommandOutput, String>,
        ) {
            let key = (
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            );
            self.responses.lock().unwrap().insert(key, response);
        }

        fn add_response_for_any(&self, program: &str, response: Result<CommandOutput, String>) {
            let mut guard = self.responses.lock().unwrap();
            let keys: Vec<_> = guard
                .keys()
                .filter(|(p, _)| p == program)
                .cloned()
                .collect();
            for key in keys {
                guard.remove(&key);
            }
            guard.insert((program.to_string(), vec!["__any__".to_string()]), response);
        }
    }

    impl CommandRunner for MockCommandRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String> {
            let key = (
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            );
            let responses = self.responses.lock().unwrap();
            if let Some(response) = responses.get(&key) {
                response.clone()
            } else if let Some(response) =
                responses.get(&(program.to_string(), vec!["__any__".to_string()]))
            {
                response.clone()
            } else {
                Err(format!("No mock response for {} {:?}", program, args))
            }
        }
    }

    fn with_mock_runner<F>(runner: Box<dyn CommandRunner>, f: F)
    where
        F: FnOnce(),
    {
        set_command_runner(runner);
        f();
        reset_command_runner();
    }

    #[test]
    fn test_check_ffmpeg_success() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffmpeg",
            &["-version"],
            Ok(CommandOutput::success_with_stdout(
                "ffmpeg version 6.0 Copyright (c) 2000-2023"
                    .as_bytes()
                    .to_vec(),
            )),
        );

        with_mock_runner(Box::new(mock), || {
            let result = check_ffmpeg();
            assert!(result.is_ok());
            assert!(result.unwrap().contains("ffmpeg version 6.0"));
        });
    }

    #[test]
    fn test_check_ffmpeg_not_installed() {
        let mock = MockCommandRunner::new();
        mock.add_response("ffmpeg", &["-version"], Err("FFmpeg not found".to_string()));

        with_mock_runner(Box::new(mock), || {
            let result = check_ffmpeg();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("FFmpeg not found"));
        });
    }

    #[test]
    fn test_list_subtitle_tracks_single_track() {
        let mock = MockCommandRunner::new();
        let ffprobe_output = r#"{
            "streams": [
                {
                    "index": 0,
                    "codec_name": "ass",
                    "codec_type": "subtitle",
                    "tags": {
                        "language": "eng",
                        "title": "English"
                    }
                }
            ]
        }"#;
        mock.add_response(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "s",
                "-show_entries",
                "stream=index,codec_name,codec_type:stream_tags=language,title",
                "-of",
                "json",
                "/path/to/video.mkv",
            ],
            Ok(CommandOutput::success_with_stdout(
                ffprobe_output.as_bytes().to_vec(),
            )),
        );

        with_mock_runner(Box::new(mock), || {
            let result = list_subtitle_tracks("/path/to/video.mkv");
            assert!(result.is_ok());
            let tracks = result.unwrap();
            assert_eq!(tracks.len(), 1);
            assert_eq!(tracks[0].stream_index, 0);
            assert_eq!(tracks[0].codec_name, "ass");
            assert_eq!(tracks[0].language, Some("eng".to_string()));
            assert_eq!(tracks[0].title, Some("English".to_string()));
        });
    }

    #[test]
    fn test_list_subtitle_tracks_multiple_tracks() {
        let mock = MockCommandRunner::new();
        let ffprobe_output = r#"{
            "streams": [
                {
                    "index": 1,
                    "codec_name": "subrip",
                    "codec_type": "subtitle",
                    "tags": {
                        "language": "eng"
                    }
                },
                {
                    "index": 2,
                    "codec_name": "ass",
                    "codec_type": "subtitle",
                    "tags": {
                        "language": "jpn",
                        "title": "Japanese"
                    }
                },
                {
                    "index": 0,
                    "codec_name": "h264",
                    "codec_type": "video"
                }
            ]
        }"#;
        mock.add_response(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "s",
                "-show_entries",
                "stream=index,codec_name,codec_type:stream_tags=language,title",
                "-of",
                "json",
                "/path/to/video.mkv",
            ],
            Ok(CommandOutput::success_with_stdout(
                ffprobe_output.as_bytes().to_vec(),
            )),
        );

        with_mock_runner(Box::new(mock), || {
            let result = list_subtitle_tracks("/path/to/video.mkv");
            assert!(result.is_ok());
            let tracks = result.unwrap();
            assert_eq!(tracks.len(), 2);
            assert_eq!(tracks[0].index, 0);
            assert_eq!(tracks[0].stream_index, 1);
            assert_eq!(tracks[0].codec_name, "subrip");
            assert_eq!(tracks[0].language, Some("eng".to_string()));
            assert_eq!(tracks[1].index, 1);
            assert_eq!(tracks[1].stream_index, 2);
            assert_eq!(tracks[1].codec_name, "ass");
            assert_eq!(tracks[1].language, Some("jpn".to_string()));
        });
    }

    #[test]
    fn test_list_subtitle_tracks_no_subtitles() {
        let mock = MockCommandRunner::new();
        let ffprobe_output = r#"{
            "streams": [
                {
                    "index": 0,
                    "codec_name": "h264",
                    "codec_type": "video"
                },
                {
                    "index": 1,
                    "codec_name": "aac",
                    "codec_type": "audio"
                }
            ]
        }"#;
        mock.add_response(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "s",
                "-show_entries",
                "stream=index,codec_name,codec_type:stream_tags=language,title",
                "-of",
                "json",
                "/path/to/video.mkv",
            ],
            Ok(CommandOutput::success_with_stdout(
                ffprobe_output.as_bytes().to_vec(),
            )),
        );

        with_mock_runner(Box::new(mock), || {
            let result = list_subtitle_tracks("/path/to/video.mkv");
            assert!(result.is_ok());
            let tracks = result.unwrap();
            assert!(tracks.is_empty());
        });
    }

    #[test]
    fn test_list_subtitle_tracks_ffprobe_failure() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "s",
                "-show_entries",
                "stream=index,codec_name,codec_type:stream_tags=language,title",
                "-of",
                "json",
                "/invalid/path.mkv",
            ],
            Ok(CommandOutput::failure(b"No such file".to_vec())),
        );

        with_mock_runner(Box::new(mock), || {
            let result = list_subtitle_tracks("/invalid/path.mkv");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("ffprobe failed"));
        });
    }

    #[test]
    fn test_extract_subtitle_track_ass() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffmpeg",
            &[
                "-y",
                "-i",
                "/path/to/video.mkv",
                "-map",
                "0:s:0",
                "-c:s",
                "ass",
                "/path/to/output.ass",
            ],
            Ok(CommandOutput::success()),
        );

        with_mock_runner(Box::new(mock), || {
            let result = extract_subtitle_track("/path/to/video.mkv", 0, "/path/to/output.ass");
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_extract_subtitle_track_srt() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffmpeg",
            &[
                "-y",
                "-i",
                "/path/to/video.mkv",
                "-map",
                "0:s:1",
                "-c:s",
                "srt",
                "/path/to/output.srt",
            ],
            Ok(CommandOutput::success()),
        );

        with_mock_runner(Box::new(mock), || {
            let result = extract_subtitle_track("/path/to/video.mkv", 1, "/path/to/output.srt");
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_extract_subtitle_track_vtt() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffmpeg",
            &[
                "-y",
                "-i",
                "/path/to/video.mkv",
                "-map",
                "0:s:0",
                "-c:s",
                "webvtt",
                "/path/to/output.vtt",
            ],
            Ok(CommandOutput::success()),
        );

        with_mock_runner(Box::new(mock), || {
            let result = extract_subtitle_track("/path/to/video.mkv", 0, "/path/to/output.vtt");
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_extract_subtitle_track_failure() {
        let mock = MockCommandRunner::new();
        mock.add_response(
            "ffmpeg",
            &[
                "-y",
                "-i",
                "/path/to/video.mkv",
                "-map",
                "0:s:0",
                "-c:s",
                "ass",
                "/path/to/output.ass",
            ],
            Ok(CommandOutput::failure(b"Invalid data".to_vec())),
        );

        with_mock_runner(Box::new(mock), || {
            let result = extract_subtitle_track("/path/to/video.mkv", 0, "/path/to/output.ass");
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("ffmpeg extraction failed"));
        });
    }

    #[test]
    fn test_mux_subtitle_track_basic() {
        let mock = MockCommandRunner::new();
        mock.add_response_for_any("ffmpeg", Ok(CommandOutput::success()));

        with_mock_runner(Box::new(mock), || {
            let result = mux_subtitle_track(
                "/path/to/video.mkv",
                "/path/to/subtitle.ass",
                "/path/to/output.mkv",
                None,
                None,
            );
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_mux_subtitle_track_with_language() {
        let mock = MockCommandRunner::new();
        mock.add_response_for_any("ffmpeg", Ok(CommandOutput::success()));

        with_mock_runner(Box::new(mock), || {
            let result = mux_subtitle_track(
                "/path/to/video.mkv",
                "/path/to/subtitle.ass",
                "/path/to/output.mkv",
                Some("eng"),
                None,
            );
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_mux_subtitle_track_with_title() {
        let mock = MockCommandRunner::new();
        mock.add_response_for_any("ffmpeg", Ok(CommandOutput::success()));

        with_mock_runner(Box::new(mock), || {
            let result = mux_subtitle_track(
                "/path/to/video.mkv",
                "/path/to/subtitle.ass",
                "/path/to/output.mkv",
                None,
                Some("Translated"),
            );
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_mux_subtitle_track_with_language_and_title() {
        let mock = MockCommandRunner::new();
        mock.add_response_for_any("ffmpeg", Ok(CommandOutput::success()));

        with_mock_runner(Box::new(mock), || {
            let result = mux_subtitle_track(
                "/path/to/video.mkv",
                "/path/to/subtitle.ass",
                "/path/to/output.mkv",
                Some("eng"),
                Some("English Translation"),
            );
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_mux_subtitle_track_failure() {
        let mock = MockCommandRunner::new();
        mock.add_response_for_any(
            "ffmpeg",
            Ok(CommandOutput::failure(b"Muxing error".to_vec())),
        );

        with_mock_runner(Box::new(mock), || {
            let result = mux_subtitle_track(
                "/path/to/video.mkv",
                "/path/to/subtitle.ass",
                "/path/to/output.mkv",
                None,
                None,
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("ffmpeg muxing failed"));
        });
    }

    #[test]
    fn test_subtitle_track_struct() {
        let track = SubtitleTrack {
            index: 0,
            stream_index: 1,
            codec_name: "ass".to_string(),
            language: Some("eng".to_string()),
            title: Some("Test".to_string()),
        };
        assert_eq!(track.index, 0);
        assert_eq!(track.stream_index, 1);
        assert_eq!(track.codec_name, "ass");
        assert_eq!(track.language, Some("eng".to_string()));
        assert_eq!(track.title, Some("Test".to_string()));
    }

    #[test]
    fn test_command_output_success() {
        let output = CommandOutput::success();
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_command_output_success_with_stdout() {
        let stdout = b"test output".to_vec();
        let output = CommandOutput::success_with_stdout(stdout.clone());
        assert!(output.status.success());
        assert_eq!(output.stdout, stdout);
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_command_output_failure() {
        let stderr = b"error message".to_vec();
        let output = CommandOutput::failure(stderr.clone());
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, stderr);
    }
}
