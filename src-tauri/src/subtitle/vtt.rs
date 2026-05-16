use super::{SubtitleEntry, SubtitleFile, SubtitleFormat, SubtitleMetadata};
use regex::Regex;
use std::sync::LazyLock;

static TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{2}:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}\.\d{3})")
        .expect("invalid timestamp regex")
});

static CUE_SETTING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([\w-]+):([^\s]+)")
        .expect("invalid cue setting regex")
});

fn parse_timestamp(ts: &str) -> String {
    ts.replace('.', ",")
}

fn strip_vtt_tags(text: &str) -> String {
    let mut result = text.to_string();
    let tag_patterns = [
        r"<c\.[^>]*>",
        r"</c>",
        r"<v\.[^>]*>",
        r"</v>",
        r"<v>",
        r"</v>",
        r"<lang\.[^>]*>",
        r"</lang>",
        r"<b>",
        r"</b>",
        r"<i>",
        r"</i>",
        r"<u>",
        r"</u>",
        r"<\/?ruby[^>]*>",
        r"<\/?rt[^>]*>",
        r"<\/?rt>",
    ];

    for pattern in &tag_patterns {
        let re = Regex::new(pattern).expect("invalid pattern");
        result = re.replace_all(&result, "").to_string();
    }

    result.trim().to_string()
}

fn parse_cue_settings(line: &str) -> Option<SubtitleMetadata> {
    let mut metadata = SubtitleMetadata::default();

    for cap in CUE_SETTING_RE.captures_iter(line) {
        let key = &cap[1];
        let value = &cap[2];
        match key {
            "align" => {
                if metadata.style.is_none() {
                    metadata.style = Some(value.to_string());
                }
            }
            "position" => {}
            "line" => {}
            "size" => {}
            "region" => {}
            _ => {}
        }
    }

    if metadata.style.is_some() || metadata.name.is_some() {
        Some(metadata)
    } else {
        None
    }
}

pub fn parse(content: &str) -> Result<SubtitleFile, String> {
    let mut entries = Vec::new();

    let content = content.replace("\r\n", "\n").replace('\r', "\n");

    let mut lines_iter = content.lines().peekable();
    let mut index = 0;

    while let Some(line) = lines_iter.next() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line == "WEBVTT" {
            continue;
        }

        if line.starts_with("NOTE") || line.starts_with("STYLE") || line.starts_with("REGION") {
            while let Some(&next) = lines_iter.peek() {
                if next.is_empty() {
                    break;
                }
                lines_iter.next();
            }
            continue;
        }

        let ts_caps = TIMESTAMP_RE.captures(line);
        if ts_caps.is_none() {
            continue;
        }

        let ts_caps = ts_caps.unwrap();
        let start_time = parse_timestamp(&ts_caps[1]);
        let end_time = parse_timestamp(&ts_caps[2]);

        index += 1;

        let mut text_lines = Vec::new();
        let mut metadata = None;

        while let Some(&next) = lines_iter.peek() {
            let next = next.trim();
            if next.is_empty() {
                break;
            }

            if TIMESTAMP_RE.is_match(next) {
                break;
            }

            lines_iter.next();

            if next.contains(':') && !next.contains("<") {
                metadata = parse_cue_settings(next);
                continue;
            }

            text_lines.push(next);
        }

        let text = strip_vtt_tags(&text_lines.join("\n"));

        if text.is_empty() {
            continue;
        }

        entries.push(SubtitleEntry {
            index,
            start_time,
            end_time,
            text,
            metadata,
        });
    }

    if entries.is_empty() {
        return Err("No valid VTT subtitle entries found".to_string());
    }

    Ok(SubtitleFile {
        format: SubtitleFormat::Vtt,
        entries,
        headers: None,
    })
}

pub fn serialize(file: &SubtitleFile) -> String {
    let mut output = String::from("WEBVTT\n\n");

    for entry in &file.entries {
        let start = entry.start_time.replace(',', ".");
        let end = entry.end_time.replace(',', ".");

        output.push_str(&format!("{}\n", entry.index));
        output.push_str(&format!("{} --> {}\n", start, end));

        if let Some(ref meta) = entry.metadata {
            let mut settings = Vec::new();
            if let Some(ref style) = meta.style {
                settings.push(format!("align:{}", style));
            }
            if !settings.is_empty() {
                output.push_str(&format!("{}\n", settings.join(" ")));
            }
        }

        output.push_str(&entry.text);
        output.push_str("\n\n");
    }

    output.trim_end().to_string() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vtt() {
        let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
Hello World

00:00:05.000 --> 00:00:10.000
This is a test
with multiple lines
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].text, "Hello World");
        assert_eq!(
            result.entries[1].text,
            "This is a test with multiple lines"
        );
    }

    #[test]
    fn test_parse_vtt_with_cue_settings() {
        let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000 align:start position:10%
Hello World
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Hello World");
    }

    #[test]
    fn test_parse_vtt_with_tags() {
        let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
<c.red>This is red</c> and <i>italic</i>
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries[0].text, "This is red and italic");
    }

    #[test]
    fn test_parse_vtt_with_voice() {
        let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
<v John>Hello World</v>
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries[0].text, "Hello World");
    }

    #[test]
    fn test_parse_vtt_comments() {
        let content = r#"WEBVTT

NOTE This is a comment

00:00:00.000 --> 00:00:05.000
Hello World
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Hello World");
    }

    #[test]
    fn test_serialize_vtt() {
        let file = SubtitleFile {
            format: SubtitleFormat::Vtt,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "00:00:01,000".to_string(),
                end_time: "00:00:04,000".to_string(),
                text: "Hello".to_string(),
                metadata: None,
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.starts_with("WEBVTT"));
        assert!(output.contains("00:00:01.000 --> 00:00:04.000"));
        assert!(output.contains("Hello"));
    }
}