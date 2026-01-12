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
}
