pub mod ass;
pub mod srt;

use serde::{Deserialize, Serialize};

/// Representa uma entrada de legenda (comum a todos os formatos)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    pub index: usize,
    pub start_time: String,
    pub end_time: String,
    pub text: String,
    /// Metadados específicos do formato (estilo, posição, etc.)
    #[serde(default)]
    pub metadata: Option<SubtitleMetadata>,
}

/// Metadados específicos do formato ASS
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubtitleMetadata {
    pub style: Option<String>,
    pub name: Option<String>,
    pub margin_l: Option<i32>,
    pub margin_r: Option<i32>,
    pub margin_v: Option<i32>,
    pub effect: Option<String>,
    /// Formato do Layer (ASS)
    pub layer: Option<i32>,
}

/// Representa um arquivo de legenda completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleFile {
    pub format: SubtitleFormat,
    pub entries: Vec<SubtitleEntry>,
    /// Headers e metadados do arquivo (para ASS: Script Info, Styles, etc.)
    #[serde(default)]
    pub headers: Option<AssHeaders>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    Ssa,
    Vtt,
}

/// Headers específicos do formato ASS/SSA
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssHeaders {
    pub script_info: Vec<String>,
    pub styles: Vec<String>,
    pub fonts: Vec<String>,
    pub graphics: Vec<String>,
}

impl SubtitleFile {
    /// Detecta o formato baseado na extensão do arquivo
    pub fn detect_format(filename: &str) -> Option<SubtitleFormat> {
        let ext = filename.rsplit('.').next()?.to_lowercase();
        match ext.as_str() {
            "srt" => Some(SubtitleFormat::Srt),
            "ass" => Some(SubtitleFormat::Ass),
            "ssa" => Some(SubtitleFormat::Ssa),
            "vtt" => Some(SubtitleFormat::Vtt),
            _ => None,
        }
    }

    /// Faz parse de um arquivo de legenda
    pub fn parse(content: &str, format: SubtitleFormat) -> Result<Self, String> {
        match format {
            SubtitleFormat::Srt => srt::parse(content),
            SubtitleFormat::Ass | SubtitleFormat::Ssa => ass::parse(content),
            SubtitleFormat::Vtt => Err("VTT parsing not yet implemented".to_string()),
        }
    }

    /// Serializa de volta para o formato original
    pub fn serialize(&self) -> String {
        match self.format {
            SubtitleFormat::Srt => srt::serialize(self),
            SubtitleFormat::Ass | SubtitleFormat::Ssa => ass::serialize(self),
            SubtitleFormat::Vtt => "".to_string(), // TODO
        }
    }

    /// Extrai apenas os textos para tradução (preservando estrutura)
    pub fn extract_texts(&self) -> Vec<(usize, String)> {
        self.entries
            .iter()
            .map(|e| (e.index, e.text.clone()))
            .collect()
    }

    /// Aplica textos traduzidos de volta
    pub fn apply_translations(&mut self, translations: Vec<(usize, String)>) {
        for (index, text) in translations {
            if let Some(entry) = self.entries.iter_mut().find(|e| e.index == index) {
                entry.text = text;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_srt() {
        assert_eq!(SubtitleFile::detect_format("movie.srt"), Some(SubtitleFormat::Srt));
        assert_eq!(SubtitleFile::detect_format("subtitles.SRT"), Some(SubtitleFormat::Srt));
        assert_eq!(SubtitleFile::detect_format("video.srt"), Some(SubtitleFormat::Srt));
    }

    #[test]
    fn test_detect_format_ass() {
        assert_eq!(SubtitleFile::detect_format("movie.ass"), Some(SubtitleFormat::Ass));
        assert_eq!(SubtitleFile::detect_format("subtitles.ASS"), Some(SubtitleFormat::Ass));
    }

    #[test]
    fn test_detect_format_ssa() {
        assert_eq!(SubtitleFile::detect_format("movie.ssa"), Some(SubtitleFormat::Ssa));
        assert_eq!(SubtitleFile::detect_format("sub.SSA"), Some(SubtitleFormat::Ssa));
    }

    #[test]
    fn test_detect_format_vtt() {
        assert_eq!(SubtitleFile::detect_format("movie.vtt"), Some(SubtitleFormat::Vtt));
        assert_eq!(SubtitleFile::detect_format("subs.VTT"), Some(SubtitleFormat::Vtt));
    }

    #[test]
    fn test_detect_format_unknown() {
        assert_eq!(SubtitleFile::detect_format("movie.txt"), None);
        assert_eq!(SubtitleFile::detect_format("movie.mp4"), None);
        assert_eq!(SubtitleFile::detect_format("movie"), None);
        assert_eq!(SubtitleFile::detect_format(".srt"), Some(SubtitleFormat::Srt));
        assert_eq!(SubtitleFile::detect_format("noextension"), None);
    }

    #[test]
    fn test_detect_format_with_path() {
        assert_eq!(SubtitleFile::detect_format("/path/to/movie.srt"), Some(SubtitleFormat::Srt));
        assert_eq!(SubtitleFile::detect_format("C:\\Users\\test\\file.ass"), Some(SubtitleFormat::Ass));
        assert_eq!(SubtitleFile::detect_format("/home/user/Desktop/subtitles.vtt"), Some(SubtitleFormat::Vtt));
    }

    #[test]
    fn test_extract_texts() {
        let file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![
                SubtitleEntry { index: 1, start_time: "00:00:01,000".to_string(), end_time: "00:00:04,000".to_string(), text: "First text".to_string(), metadata: None },
                SubtitleEntry { index: 2, start_time: "00:00:05,000".to_string(), end_time: "00:00:08,000".to_string(), text: "Second text".to_string(), metadata: None },
                SubtitleEntry { index: 3, start_time: "00:00:09,000".to_string(), end_time: "00:00:12,000".to_string(), text: "Third text".to_string(), metadata: None },
            ],
            headers: None,
        };

        let texts = file.extract_texts();
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0], (1, "First text".to_string()));
        assert_eq!(texts[1], (2, "Second text".to_string()));
        assert_eq!(texts[2], (3, "Third text".to_string()));
    }

    #[test]
    fn test_apply_translations() {
        let mut file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![
                SubtitleEntry { index: 1, start_time: "00:00:01,000".to_string(), end_time: "00:00:04,000".to_string(), text: "Original".to_string(), metadata: None },
                SubtitleEntry { index: 2, start_time: "00:00:05,000".to_string(), end_time: "00:00:08,000".to_string(), text: "Original 2".to_string(), metadata: None },
            ],
            headers: None,
        };

        let translations = vec![
            (1, "Translated 1".to_string()),
            (2, "Translated 2".to_string()),
        ];

        file.apply_translations(translations);

        assert_eq!(file.entries[0].text, "Translated 1");
        assert_eq!(file.entries[1].text, "Translated 2");
    }

    #[test]
    fn test_apply_translations_partial() {
        let mut file = SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![
                SubtitleEntry { index: 1, start_time: "00:00:01,000".to_string(), end_time: "00:00:04,000".to_string(), text: "Original".to_string(), metadata: None },
                SubtitleEntry { index: 2, start_time: "00:00:05,000".to_string(), end_time: "00:00:08,000".to_string(), text: "Original 2".to_string(), metadata: None },
                SubtitleEntry { index: 3, start_time: "00:00:09,000".to_string(), end_time: "00:00:12,000".to_string(), text: "Original 3".to_string(), metadata: None },
            ],
            headers: None,
        };

        let translations = vec![
            (2, "Translated 2".to_string()),
        ];

        file.apply_translations(translations);

        assert_eq!(file.entries[0].text, "Original");
        assert_eq!(file.entries[1].text, "Translated 2");
        assert_eq!(file.entries[2].text, "Original 3");
    }

    #[test]
    fn test_parse_and_serialize_roundtrip_srt() {
        let content = r#"1
00:00:01,000 --> 00:00:04,000
Hello World

2
00:00:05,000 --> 00:00:08,000
This is a test
"#;

        let file = SubtitleFile::parse(content, SubtitleFormat::Srt).unwrap();
        let serialized = file.serialize();
        let reparsed = SubtitleFile::parse(&serialized, SubtitleFormat::Srt).unwrap();

        assert_eq!(file.entries.len(), reparsed.entries.len());
        assert_eq!(file.entries[0].text, reparsed.entries[0].text);
        assert_eq!(file.entries[1].text, reparsed.entries[1].text);
    }

    #[test]
    fn test_parse_and_serialize_roundtrip_ass() {
        let content = r#"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:04.00,Default,,0000,0000,0000,,Hello ASS
"#;

        let file = SubtitleFile::parse(content, SubtitleFormat::Ass).unwrap();
        let serialized = file.serialize();
        let reparsed = SubtitleFile::parse(&serialized, SubtitleFormat::Ass).unwrap();

        assert_eq!(file.entries.len(), reparsed.entries.len());
        assert_eq!(file.entries[0].text, reparsed.entries[0].text);
    }

    #[test]
    fn test_parse_vtt_not_implemented() {
        let content = "WEBVTT\n\n1\n00:00:01.000 --> 00:00:04.000\nHello VTT";
        let result = SubtitleFile::parse(content, SubtitleFormat::Vtt);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "VTT parsing not yet implemented");
    }

    #[test]
    fn test_serialize_vtt_not_implemented() {
        let file = SubtitleFile {
            format: SubtitleFormat::Vtt,
            entries: vec![
                SubtitleEntry { index: 1, start_time: "00:00:01.000".to_string(), end_time: "00:00:04.000".to_string(), text: "Hello".to_string(), metadata: None },
            ],
            headers: None,
        };
        let result = file.serialize();
        assert_eq!(result, "");
    }

    #[test]
    fn test_ass_headers_default() {
        let headers = AssHeaders::default();
        assert!(headers.script_info.is_empty());
        assert!(headers.styles.is_empty());
        assert!(headers.fonts.is_empty());
        assert!(headers.graphics.is_empty());
    }

    #[test]
    fn test_subtitle_metadata_default() {
        let meta = SubtitleMetadata::default();
        assert!(meta.style.is_none());
        assert!(meta.name.is_none());
        assert!(meta.margin_l.is_none());
        assert!(meta.margin_r.is_none());
        assert!(meta.margin_v.is_none());
        assert!(meta.effect.is_none());
        assert!(meta.layer.is_none());
    }

    #[test]
    fn test_subtitle_format_equality() {
        assert_eq!(SubtitleFormat::Srt, SubtitleFormat::Srt);
        assert_eq!(SubtitleFormat::Ass, SubtitleFormat::Ass);
        assert_ne!(SubtitleFormat::Srt, SubtitleFormat::Ass);
    }
}
