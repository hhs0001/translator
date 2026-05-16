use super::{SubtitleEntry, SubtitleFile, SubtitleFormat};
use regex::Regex;

/// Faz parse de um arquivo SRT
pub fn parse(content: &str) -> Result<SubtitleFile, String> {
    let mut entries = Vec::new();

    // Normaliza line endings
    let content = content.replace("\r\n", "\n").replace('\r', "\n");

    // Regex para timestamps SRT: 00:00:00,000 --> 00:00:00,000
    let timestamp_re = Regex::new(r"(\d{2}:\d{2}:\d{2},\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2},\d{3})")
        .map_err(|e| e.to_string())?;

    // Divide por blocos (separados por linha em branco)
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 2 {
            continue;
        }

        // Primeira linha: índice
        let index: usize = match lines[0].trim().parse() {
            Ok(i) => i,
            Err(_) => continue, // Pula blocos inválidos
        };

        // Segunda linha: timestamps
        let timestamp_line = lines[1].trim();
        let caps = match timestamp_re.captures(timestamp_line) {
            Some(c) => c,
            None => continue,
        };

        let start_time = caps[1].to_string();
        let end_time = caps[2].to_string();

        // Resto: texto (pode ter múltiplas linhas)
        let text = if lines.len() > 2 {
            lines[2..].join("\n")
        } else {
            String::new()
        };

        entries.push(SubtitleEntry {
            index,
            start_time,
            end_time,
            text,
            metadata: None,
        });
    }

    if entries.is_empty() {
        return Err("No valid subtitle entries found".to_string());
    }

    Ok(SubtitleFile {
        format: SubtitleFormat::Srt,
        entries,
        headers: None,
    })
}

/// Serializa para formato SRT
pub fn serialize(file: &SubtitleFile) -> String {
    let mut output = String::new();

    for entry in &file.entries {
        output.push_str(&format!("{}\n", entry.index));
        output.push_str(&format!("{} --> {}\n", entry.start_time, entry.end_time));
        output.push_str(&entry.text);
        output.push_str("\n\n");
    }

    output.trim_end().to_string() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt() {
        let content = r#"1
00:00:01,000 --> 00:00:04,000
Hello World

2
00:00:05,000 --> 00:00:08,000
This is a test
with multiple lines
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].text, "Hello World");
        assert_eq!(
            result.entries[1].text,
            "This is a test\nwith multiple lines"
        );
    }

    #[test]
    fn test_serialize_srt() {
        let file = SubtitleFile {
            format: SubtitleFormat::Srt,
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
        assert!(output.contains("1\n00:00:01,000 --> 00:00:04,000\nHello"));
    }

    #[test]
    fn test_parse_timestamps() {
        let content = r#"1
00:01:02,345 --> 00:02:03,456
First

2
01:23:45,678 --> 01:24:56,789
Second
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].start_time, "00:01:02,345");
        assert_eq!(result.entries[0].end_time, "00:02:03,456");
        assert_eq!(result.entries[1].start_time, "01:23:45,678");
        assert_eq!(result.entries[1].end_time, "01:24:56,789");
    }

    #[test]
    fn test_parse_special_characters() {
        let content = r#"1
00:00:00,000 --> 00:00:01,000
Hello, World! ¿Cómo estás? 日本語

2
00:00:01,000 --> 00:00:02,000
<b>Bold</b> & <i>italic</i> & "quotes" & 'apostrophes'

3
00:00:02,000 --> 00:00:03,000
Line1
Line2
Line3 with "quotes" and 'apostrophes'
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(result.entries[0].text, "Hello, World! ¿Cómo estás? 日本語");
        assert!(result.entries[1].text.contains("&quot;"));
        assert!(result.entries[2].text.contains("Line1\nLine2\nLine3"));
    }

    #[test]
    fn test_parse_empty_lines_skipped() {
        let content = r#"1
00:00:00,000 --> 00:00:01,000
Text with

blank line in middle

2
00:00:01,000 --> 00:00:02,000
Second entry
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert!(result.entries[0].text.contains("\n"));
    }

    #[test]
    fn test_parse_invalid_blocks_skipped() {
        let content = r#"1
00:00:00,000 --> 00:00:01,000
Valid entry

INVALID BLOCK HERE

3
00:00:02,000 --> 00:00:03,000
Another valid
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].index, 1);
        assert_eq!(result.entries[1].index, 3);
    }

    #[test]
    fn test_parse_windows_line_endings() {
        let content = "1\r\n00:00:00,000 --> 00:00:01,000\r\nFirst\r\n\r\n2\r\n00:00:01,000 --> 00:00:02,000\r\nSecond\r\n";
        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_parse_mac_line_endings() {
        let content = "1\r00:00:00,000 --> 00:00:01,000\rFirst\r\r2\r00:00:01,000 --> 00:00:02,000\rSecond\r";
        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_roundtrip_parse_serialize_parse() {
        let original = r#"1
00:00:01,000 --> 00:00:04,000
Hello World

2
00:00:05,000 --> 00:00:08,000
This is a test
with multiple lines
"#;

        let parsed = parse(original).unwrap();
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).unwrap();

        assert_eq!(parsed.entries.len(), reparsed.entries.len());
        assert_eq!(parsed.entries[0].text, reparsed.entries[0].text);
        assert_eq!(parsed.entries[1].text, reparsed.entries[1].text);
        assert_eq!(parsed.entries[0].start_time, reparsed.entries[0].start_time);
        assert_eq!(parsed.entries[0].end_time, reparsed.entries[0].end_time);
    }

    #[test]
    fn test_serialize_format() {
        let file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![
                SubtitleEntry {
                    index: 1,
                    start_time: "00:00:01,000".to_string(),
                    end_time: "00:00:04,000".to_string(),
                    text: "Line one".to_string(),
                    metadata: None,
                },
                SubtitleEntry {
                    index: 2,
                    start_time: "00:00:05,000".to_string(),
                    end_time: "00:00:08,000".to_string(),
                    text: "Line two".to_string(),
                    metadata: None,
                },
            ],
            headers: None,
        };

        let output = serialize(&file);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines[0], "1");
        assert_eq!(lines[1], "00:00:01,000 --> 00:00:04,000");
        assert_eq!(lines[2], "Line one");
        assert_eq!(lines[4], "2");
        assert_eq!(lines[5], "00:00:05,000 --> 00:00:08,000");
        assert_eq!(lines[6], "Line two");
    }

    #[test]
    fn test_parse_index_handling() {
        let content = r#"99
00:00:00,000 --> 00:00:01,000
Entry with non-sequential index
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries[0].index, 99);
    }

    #[test]
    fn test_parse_missing_index_still_works() {
        let content = r#"NotANumber
00:00:00,000 --> 00:00:01,000
This should be skipped
"#;

        let result = parse(content);
        assert!(result.is_err() || result.unwrap().entries.is_empty());
    }

    #[test]
    fn test_parse_no_timestamp_line() {
        let content = r#"1
This is not a timestamp line
Some text
"#;

        let result = parse(content);
        assert!(result.is_err() || result.unwrap().entries.is_empty());
    }

    #[test]
    fn test_parse_empty_content() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_only_whitespace() {
        let result = parse("   \n\n   \n\n   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_empty_text() {
        let file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "00:00:01,000".to_string(),
                end_time: "00:00:04,000".to_string(),
                text: "".to_string(),
                metadata: None,
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.contains("1\n00:00:01,000 --> 00:00:04,000\n\n\n"));
    }

    #[test]
    fn test_serialize_preserves_timestamp_format() {
        let file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![SubtitleEntry {
                index: 42,
                start_time: "01:23:45,678".to_string(),
                end_time: "01:24:56,789".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.contains("01:23:45,678 --> 01:24:56,789"));
    }

    #[test]
    fn test_parse_single_entry() {
        let content = r#"1
00:00:00,000 --> 00:00:01,000
Single entry
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Single entry");
        assert_eq!(result.entries[0].index, 1);
    }

    #[test]
    fn test_parse_consecutive_blocks_no_blank_line() {
        let content = r#"1
00:00:00,000 --> 00:00:01,000
First
2
00:00:01,000 --> 00:00:02,000
Second
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].text, "First");
        assert_eq!(result.entries[1].text, "Second");
    }
}
