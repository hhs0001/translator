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
}
