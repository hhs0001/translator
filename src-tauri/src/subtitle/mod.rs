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
