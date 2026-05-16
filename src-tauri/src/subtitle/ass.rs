use super::{AssHeaders, SubtitleEntry, SubtitleFile, SubtitleFormat, SubtitleMetadata};

/// Faz parse de um arquivo ASS/SSA
/// Preserva completamente: [Script Info], [V4+ Styles], [Fonts], [Graphics]
/// Extrai apenas os diálogos de [Events] para tradução
pub fn parse(content: &str) -> Result<SubtitleFile, String> {
    let content = content.replace("\r\n", "\n").replace('\r', "\n");

    let mut headers = AssHeaders::default();
    let mut entries = Vec::new();
    let mut current_section = String::new();
    let mut dialogue_format: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Detecta seção
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_lowercase();
            // Salva o header da seção também
            match current_section.as_str() {
                "script info" => headers.script_info.push(line.to_string()),
                "v4+ styles" | "v4 styles" | "v4 styles+" => headers.styles.push(line.to_string()),
                "fonts" => headers.fonts.push(line.to_string()),
                "graphics" => headers.graphics.push(line.to_string()),
                _ => {}
            }
            continue;
        }

        // Processa linha baseado na seção atual
        match current_section.as_str() {
            "script info" => {
                if !line.is_empty() {
                    headers.script_info.push(line.to_string());
                }
            }
            "v4+ styles" | "v4 styles" | "v4 styles+" => {
                if !line.is_empty() {
                    headers.styles.push(line.to_string());
                }
            }
            "fonts" => {
                if !line.is_empty() {
                    headers.fonts.push(line.to_string());
                }
            }
            "graphics" => {
                if !line.is_empty() {
                    headers.graphics.push(line.to_string());
                }
            }
            "events" => {
                // Captura o formato dos diálogos
                if let Some(format_part) = line.strip_prefix("Format:") {
                    dialogue_format = format_part
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .collect();
                } else if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                    if let Some(entry) = parse_dialogue_line(line, &dialogue_format, entries.len())
                    {
                        entries.push(entry);
                    }
                }
            }
            _ => {}
        }
    }

    if entries.is_empty() {
        return Err("No dialogue entries found in ASS file".to_string());
    }

    Ok(SubtitleFile {
        format: SubtitleFormat::Ass,
        entries,
        headers: Some(headers),
    })
}

/// Faz parse de uma linha de Dialogue
fn parse_dialogue_line(line: &str, format: &[String], index: usize) -> Option<SubtitleEntry> {
    // Remove "Dialogue: " ou "Comment: "
    let is_comment = line.starts_with("Comment:");
    let content = if is_comment {
        &line["Comment:".len()..]
    } else {
        &line["Dialogue:".len()..]
    };

    // ASS usa vírgulas como separador, mas o texto pode conter vírgulas
    // O texto é sempre o último campo, então dividimos em N-1 partes
    let parts: Vec<&str> = content.splitn(format.len(), ',').collect();

    if parts.len() < format.len() {
        return None;
    }

    let mut start_time = String::new();
    let mut end_time = String::new();
    let mut style = None;
    let mut name = None;
    let mut margin_l = None;
    let mut margin_r = None;
    let mut margin_v = None;
    let mut effect = None;
    let mut layer = None;
    let mut text = String::new();

    for (i, field_name) in format.iter().enumerate() {
        let value = parts.get(i).map(|s| s.trim()).unwrap_or("");

        match field_name.as_str() {
            "layer" => layer = value.parse().ok(),
            "start" => start_time = value.to_string(),
            "end" => end_time = value.to_string(),
            "style" => style = Some(value.to_string()),
            "name" | "actor" => name = Some(value.to_string()),
            "marginl" => margin_l = value.parse().ok(),
            "marginr" => margin_r = value.parse().ok(),
            "marginv" => margin_v = value.parse().ok(),
            "effect" => effect = Some(value.to_string()),
            "text" => text = value.to_string(),
            _ => {}
        }
    }

    // Se não encontrou o campo text, pega o último elemento
    if text.is_empty() && !parts.is_empty() {
        text = parts.last().unwrap().trim().to_string();
    }

    Some(SubtitleEntry {
        index: index + 1, // 1-indexed para consistência com SRT
        start_time,
        end_time,
        text,
        metadata: Some(SubtitleMetadata {
            style,
            name,
            margin_l,
            margin_r,
            margin_v,
            effect,
            layer,
        }),
    })
}

/// Serializa para formato ASS
pub fn serialize(file: &SubtitleFile) -> String {
    let mut output = String::new();

    // Escreve headers preservados
    if let Some(headers) = &file.headers {
        // Script Info
        if !headers.script_info.is_empty() {
            for line in &headers.script_info {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Styles
        if !headers.styles.is_empty() {
            for line in &headers.styles {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Fonts
        if !headers.fonts.is_empty() {
            for line in &headers.fonts {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }

        // Graphics
        if !headers.graphics.is_empty() {
            for line in &headers.graphics {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
    }

    // Events section
    output.push_str("[Events]\n");
    output.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for entry in &file.entries {
        let meta = entry.metadata.as_ref();

        let layer = meta.and_then(|m| m.layer).unwrap_or(0);
        let style = meta
            .and_then(|m| m.style.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Default");
        let name = meta
            .and_then(|m| m.name.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");
        let margin_l = meta.and_then(|m| m.margin_l).unwrap_or(0);
        let margin_r = meta.and_then(|m| m.margin_r).unwrap_or(0);
        let margin_v = meta.and_then(|m| m.margin_v).unwrap_or(0);
        let effect = meta
            .and_then(|m| m.effect.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("");

        // Converte \n (newline real) para \N (formato ASS)
        let text_for_ass = entry.text.replace('\n', "\\N");

        output.push_str(&format!(
            "Dialogue: {},{},{},{},{},{:04},{:04},{:04},{},{}\n",
            layer,
            entry.start_time,
            entry.end_time,
            style,
            name,
            margin_l,
            margin_r,
            margin_v,
            effect,
            text_for_ass
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ass() {
        let content = r#"[Script Info]
Title: Test
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Hello World
Dialogue: 0,0:00:05.00,0:00:08.00,Default,,0000,0000,0000,,{\i1}Italic text{\i0}
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.format, SubtitleFormat::Ass);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].text, "Hello World");
        assert_eq!(result.entries[1].text, r"{\i1}Italic text{\i0}");

        // Verifica que os headers foram preservados
        let headers = result.headers.as_ref().unwrap();
        assert!(headers
            .script_info
            .iter()
            .any(|l| l.contains("Title: Test")));
        assert!(headers.styles.iter().any(|l| l.contains("Style: Default")));
    }

    #[test]
    fn test_serialize_ass() {
        let file = SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "0:00:01.00".to_string(),
                end_time: "0:00:04.00".to_string(),
                text: "Hello".to_string(),
                metadata: Some(SubtitleMetadata {
                    style: Some("Default".to_string()),
                    layer: Some(0),
                    ..Default::default()
                }),
            }],
            headers: Some(AssHeaders {
                script_info: vec!["[Script Info]".to_string(), "Title: Test".to_string()],
                styles: vec![
                    "[V4+ Styles]".to_string(),
                    "Format: Name, Fontname".to_string(),
                    "Style: Default,Arial".to_string(),
                ],
                ..Default::default()
            }),
        };

        let output = serialize(&file);
        assert!(output.contains("[Script Info]"));
        assert!(output.contains("Title: Test"));
        assert!(output.contains("[Events]"));
        assert!(output.contains("Dialogue:"));
    }

    #[test]
    fn test_parse_ass_with_styles() {
        let content = r#"[Script Info]
Title: Styled Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, BorderStyle, Alignment
Style: Default,Arial,20,&H00FFFFFF, &H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,1,2
Style: CustomStyle,Verdana,24,&H00FFFF00,-1,-1,-1,1,1,0,0,100,100,3,5

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Default style text
Dialogue: 1,0:00:05.00,0:00:08.00,CustomStyle,,0000,0000,0000,,Custom styled text
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.format, SubtitleFormat::Ass);
        assert_eq!(result.entries.len(), 2);
        
        let entry1 = &result.entries[0];
        let entry2 = &result.entries[1];
        
        assert_eq!(entry1.text, "Default style text");
        assert_eq!(entry1.metadata.as_ref().unwrap().style.as_ref().unwrap(), "Default");
        assert_eq!(entry1.metadata.as_ref().unwrap().layer, Some(0));
        
        assert_eq!(entry2.text, "Custom styled text");
        assert_eq!(entry2.metadata.as_ref().unwrap().style.as_ref().unwrap(), "CustomStyle");
        assert_eq!(entry2.metadata.as_ref().unwrap().layer, Some(1));
    }

    #[test]
    fn test_parse_ass_position_tags() {
        let content = r#"[Script Info]
Title: Position Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.00,Default,,0100,0200,0300,,Normal margins
Dialogue: 0,0:00:03.00,0:00:06.00,Default,,1000,2000,0000,,Large margins
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        
        let entry1 = &result.entries[0];
        assert_eq!(entry1.metadata.as_ref().unwrap().margin_l, Some(100));
        assert_eq!(entry1.metadata.as_ref().unwrap().margin_r, Some(200));
        assert_eq!(entry1.metadata.as_ref().unwrap().margin_v, Some(300));
        
        let entry2 = &result.entries[1];
        assert_eq!(entry2.metadata.as_ref().unwrap().margin_l, Some(1000));
        assert_eq!(entry2.metadata.as_ref().unwrap().margin_r, Some(2000));
        assert_eq!(entry2.metadata.as_ref().unwrap().margin_v, Some(0));
    }

    #[test]
    fn test_parse_ass_basic_formatting_tags() {
        let content = r#"[Script Info]
Title: Formatting Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,{\i1}Italic text{\i0}
Dialogue: 0,0:00:04.00,0:00:07.00,Default,,0000,0000,0000,,{\b1}Bold text{\b0}
Dialogue: 0,0:00:07.00,0:00:10.00,Default,,0000,0000,0000,,{\u1}Underline text{\u0}
Dialogue: 0,0:00:10.00,0:00:13.00,Default,,0000,0000,0000,,{\i1\b1}Bold and italic{\b0\i0}
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 4);
        
        assert!(result.entries[0].text.contains(r"{\i1}"));
        assert!(result.entries[1].text.contains(r"{\b1}"));
        assert!(result.entries[2].text.contains(r"{\u1}"));
        assert!(result.entries[3].text.contains(r"{\i1\b1}"));
    }

    #[test]
    fn test_parse_ass_effect_field() {
        let content = r#"[Script Info]
Title: Effects Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Banner;Fade Center;50;
Dialogue: 0,0:00:04.00,0:00:07.00,Default,,0000,0000,0000,,No effect
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        
        assert_eq!(result.entries[0].metadata.as_ref().unwrap().effect.as_ref().unwrap(), "Banner;Fade Center;50;");
        assert!(result.entries[1].metadata.as_ref().unwrap().effect.is_none());
    }

    #[test]
    fn test_parse_ass_comment_lines() {
        let content = r#"[Script Info]
Title: Comment Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Real dialogue
Comment: 0,0:00:00.00,0:00:00.00,Default,,0000,0000,0000,,This is a comment
Comment: 0,0:00:00.00,0:00:00.00,Default,,0000,0000,0000,,Another comment
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(result.entries[0].text, "Real dialogue");
        assert!(result.entries[1].text.contains("comment") || result.entries[1].text.is_empty());
        assert!(result.entries[2].text.contains("comment") || result.entries[2].text.is_empty());
    }

    #[test]
    fn test_parse_ass_windows_line_endings() {
        let content = "[Script Info]\r\nTitle: Test\r\n\r\n[V4+ Styles]\r\nFormat: Name, Fontname\r\nStyle: Default,Arial,20\r\n\r\n[Events]\r\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\r\nDialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Hello\r\n";
        
        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Hello");
    }

    #[test]
    fn test_parse_ass_headers_preserved() {
        let content = r#"[Script Info]
Title: Test Script
ScriptType: v4.00+
WrapStyle: 0
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF
Style: Alt,Arial,18,&H00FFFF00

[Fonts]
Name: Arial
Name: Verdana

[Graphics]
Filename: bg.png

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Hello
"#;

        let result = parse(content).unwrap();
        let headers = result.headers.as_ref().unwrap();
        
        assert!(headers.script_info.iter().any(|l| l.contains("Title: Test Script")));
        assert!(headers.script_info.iter().any(|l| l.contains("ScriptType: v4.00+")));
        assert!(headers.styles.iter().any(|l| l.contains("Style: Default")));
        assert!(headers.styles.iter().any(|l| l.contains("Style: Alt")));
        assert!(!headers.fonts.is_empty());
        assert!(!headers.graphics.is_empty());
    }

    #[test]
    fn test_serialize_ass_preserves_headers() {
        let headers = AssHeaders {
            script_info: vec!["[Script Info]".to_string(), "Title: Test".to_string()],
            styles: vec!["[V4+ Styles]".to_string(), "Format: Name, Fontname".to_string()],
            fonts: vec!["[Fonts]".to_string(), "Name: Arial".to_string()],
            graphics: vec![],
        };
        
        let file = SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "0:00:01.00".to_string(),
                end_time: "0:00:04.00".to_string(),
                text: "Test".to_string(),
                metadata: Some(SubtitleMetadata::default()),
            }],
            headers: Some(headers),
        };

        let output = serialize(&file);
        assert!(output.contains("[Script Info]"));
        assert!(output.contains("Title: Test"));
        assert!(output.contains("[V4+ Styles]"));
        assert!(output.contains("[Fonts]"));
    }

    #[test]
    fn test_serialize_ass_newline_conversion() {
        let file = SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "0:00:01.00".to_string(),
                end_time: "0:00:04.00".to_string(),
                text: "Line1\nLine2\nLine3".to_string(),
                metadata: Some(SubtitleMetadata::default()),
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.contains("Line1\\NLine2\\NLine3"));
        assert!(!output.contains("Line1\nLine2"));
    }

    #[test]
    fn test_serialize_ass_default_metadata() {
        let file = SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "0:00:01.00".to_string(),
                end_time: "0:00:04.00".to_string(),
                text: "Hello".to_string(),
                metadata: None,
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.contains("Dialogue:"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_roundtrip_parse_serialize_parse() {
        let original = r#"[Script Info]
Title: Roundtrip Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&H00FFFFFF

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,First line
Dialogue: 0,0:00:05.00,0:00:08.00,Default,,0000,0000,0000,,Second line
"#;

        let parsed = parse(original).unwrap();
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).unwrap();

        assert_eq!(parsed.entries.len(), reparsed.entries.len());
        assert_eq!(parsed.entries[0].text, reparsed.entries[0].text);
        assert_eq!(parsed.entries[1].text, reparsed.entries[1].text);
    }

    #[test]
    fn test_parse_ass_name_actor_field() {
        let content = r#"[Script Info]
Title: Actor Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,John,0000,0000,0000,,Speech by John
Dialogue: 0,0:00:04.00,0:00:07.00,Default,Jane,0000,0000,0000,,Speech by Jane
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 2);
        
        assert_eq!(result.entries[0].metadata.as_ref().unwrap().name.as_ref().unwrap(), "John");
        assert_eq!(result.entries[1].metadata.as_ref().unwrap().name.as_ref().unwrap(), "Jane");
    }

    #[test]
    fn test_parse_ass_empty_style_field() {
        let content = r#"[Script Info]
Title: Empty Style Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,,0000,0000,0000,,No style specified
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        let meta = result.entries[0].metadata.as_ref().unwrap();
        assert!(meta.style.is_none());
    }

    #[test]
    fn test_parse_ass_no_format_line() {
        let content = r#"[Script Info]
Title: Test

[Events]
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Should this be parsed?
"#;

        let result = parse(content);
        assert!(result.is_err() || result.entries.is_empty());
    }

    #[test]
    fn test_parse_ass_missing_required_sections() {
        let content = r#"[Script Info]
Title: Incomplete File

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Entry without styles section
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_parse_ass_multiline_text_with_newlines() {
        let content = r#"[Script Info]
Title: Multiline Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Line1\NLine2\NLine3
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Line1\\NLine2\\NLine3");
    }

    #[test]
    fn test_parse_ass_various_alignment_values() {
        let content = r#"[Script Info]
Title: Alignment Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0000,0000,0000,,Align 2
Dialogue: 1,0:00:02.00,0:00:03.00,Default,,0000,0000,0000,,Layer 1
Dialogue: 2,0:00:03.00,0:00:04.00,Default,,0000,0000,0000,,Layer 2
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert_eq!(result.entries[0].metadata.as_ref().unwrap().layer, Some(0));
        assert_eq!(result.entries[1].metadata.as_ref().unwrap().layer, Some(1));
        assert_eq!(result.entries[2].metadata.as_ref().unwrap().layer, Some(2));
    }

    #[test]
    fn test_parse_ass_zero_margins() {
        let content = r#"[Script Info]
Title: Zero Margins Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Zero margins
"#;

        let result = parse(content).unwrap();
        assert_eq!(result.entries.len(), 1);
        let meta = result.entries[0].metadata.as_ref().unwrap();
        assert_eq!(meta.margin_l, Some(0));
        assert_eq!(meta.margin_r, Some(0));
        assert_eq!(meta.margin_v, Some(0));
    }

    #[test]
    fn test_serialize_ass_with_empty_headers() {
        let file = SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![SubtitleEntry {
                index: 1,
                start_time: "0:00:01.00".to_string(),
                end_time: "0:00:04.00".to_string(),
                text: "Test".to_string(),
                metadata: None,
            }],
            headers: None,
        };

        let output = serialize(&file);
        assert!(output.contains("[Events]"));
        assert!(output.contains("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"));
        assert!(output.contains("Dialogue:"));
    }

    #[test]
    fn test_roundtrip_preserves_metadata() {
        let content = r#"[Script Info]
Title: Metadata Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, Alignment
Style: Default,Arial,20,2
Style: Custom,Times,24,5

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Custom,Jane,0100,0200,0300,,Custom styled entry
"#;

        let parsed = parse(content).unwrap();
        let serialized = serialize(&parsed);
        let reparsed = parse(&serialized).unwrap();

        assert_eq!(parsed.entries.len(), reparsed.entries.len());
        
        let original_meta = parsed.entries[0].metadata.as_ref().unwrap();
        let reparsed_meta = reparsed.entries[0].metadata.as_ref().unwrap();
        
        assert_eq!(original_meta.style, reparsed_meta.style);
        assert_eq!(original_meta.name, reparsed_meta.name);
        assert_eq!(original_meta.margin_l, reparsed_meta.margin_l);
        assert_eq!(original_meta.margin_r, reparsed_meta.margin_r);
        assert_eq!(original_meta.margin_v, reparsed_meta.margin_v);
    }
}
