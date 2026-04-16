//! Motor de limpeza de texto para legendas
//! 
//! Extrai texto puro de tags ASS/SSA para envio à API de tradução,
//! e reaplica as tags originais após a tradução.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Configuração do motor de limpeza
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextCleanerConfig {
    /// Habilitar limpeza de tags ASS/SSA
    pub enabled: bool,
    /// Preservar tags básicas de formatação (negrito, itálico, underline)
    pub preserve_basic_formatting: bool,
    /// Tags específicas a serem removidas (além das de efeito visual)
    pub tags_to_remove: Vec<String>,
    /// Estilos a serem ignorados completamente (não enviados para tradução)
    pub ignored_styles: Vec<String>,
    /// Preservar efeitos de karaoke (\k, \kf, \ko, etc.)
    pub preserve_karaoke_timing: bool,
    /// Preservar posicionamento (\pos, \move, \org)
    pub preserve_positioning: bool,
}

impl Default for TextCleanerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preserve_basic_formatting: true,
            tags_to_remove: vec![],
            ignored_styles: vec![
                "draw".to_string(),
            ],
            preserve_karaoke_timing: false,
            preserve_positioning: false,
        }
    }
}

/// Mapeamento de tags por linha para reaplicação posterior
#[derive(Debug, Clone)]
pub struct TextMapping {
    /// Índice da entrada
    pub entry_index: usize,
    /// Texto original completo (com tags)
    pub original_text: String,
    /// Texto limpo (sem tags) para tradução
    pub clean_text: String,
    /// Tags de abertura (antes do texto)
    pub opening_tags: Vec<String>,
    /// Tags de fechamento (após o texto)
    pub closing_tags: Vec<String>,
    /// Tags inline (durante o texto) - mapeamento de posição -> tag
    pub inline_tags: HashMap<usize, Vec<String>>,
    /// Estilo da linha (para verificar ignored_styles)
    #[allow(dead_code)]
    pub style: Option<String>,
    /// Se esta linha deve ser ignorada na tradução
    pub should_skip_translation: bool,
}

/// Resultado da limpeza de um arquivo de legenda
#[derive(Debug, Clone)]
pub struct CleanedSubtitle {
    /// Mapeamentos de todas as linhas
    pub mappings: Vec<TextMapping>,
    /// Textos limpos para tradução (apenas as linhas que precisam ser traduzidas)
    pub texts_to_translate: Vec<(usize, String)>,
}

// Regex para extrair tags ASS
static TAGS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{([^}]*)\}").unwrap()
});

// Tags de formatação básica
static BASIC_FORMATTING_TAGS: &[&str] = &["b", "i", "u", "s", "strike"];

// Tags de efeito visual (sempre removidas se não preservar positioning)
static VISUAL_EFFECT_TAGS: &[&str] = &[
    "pos", "move", "org", "fad", "fade", "t", "frx", "fry", "frz",
    "fscx", "fscy", "fax", "fay", "blur", "be", "c", "1c", "2c", "3c", "4c",
    "alpha", "1a", "2a", "3a", "4a", "an", "a", "fn", "fs", "fsp", "fsc",
    "clip", "iclip", "p", "pbo", "bord", "shad", "xbord", "ybord", "xshad", "yshad",
    "jitter", "kt", "ktl", "ktr",
];

// Tags de karaoke/timing
static KARAOKE_TAGS: &[&str] = &["k", "kf", "ko", "k0", "K", "q"];

/// Extrai tags de uma string ASS e retorna (texto_limpo, lista_de_tags)
fn extract_tags(text: &str) -> (String, Vec<(usize, String)>) {
    let mut clean_text = String::new();
    let mut tags = Vec::new();
    let mut last_end = 0;
    let mut offset = 0;

    for cap in TAGS_REGEX.captures_iter(text) {
        let mat = cap.get(0).unwrap();
        let tag_content = cap.get(1).unwrap().as_str();
        
        // Adiciona texto antes da tag
        let before_tag = &text[last_end..mat.start()];
        clean_text.push_str(before_tag);
        offset += before_tag.len();
        
        // Armazena a tag com sua posição
        tags.push((offset, tag_content.to_string()));
        
        last_end = mat.end();
    }
    
    // Adiciona resto do texto
    clean_text.push_str(&text[last_end..]);
    
    // Converte \N e \n para espaços ou mantém como quebras
    clean_text = clean_text.replace("\\N", "\n").replace("\\n", "\n");
    // Remove \\h (hard space)
    clean_text = clean_text.replace("\\h", " ");
    
    (clean_text, tags)
}

/// Analisa tags e separa em categorias
fn categorize_tags(tags: Vec<(usize, String)>) -> (Vec<String>, Vec<String>, HashMap<usize, Vec<String>>) {
    let mut opening = Vec::new();
    let mut closing = Vec::new();
    let mut inline: HashMap<usize, Vec<String>> = HashMap::new();
    
    for (pos, tag_content) in tags {
        // Verifica se é tag de fechamento (reset)
        if tag_content.starts_with("r") || tag_content == "0" {
            closing.push(format!("{{{}}}", tag_content));
        } else if tag_content.starts_with('/') {
            // Tag de fechamento explícito
            closing.push(format!("{{{}}}", tag_content));
        } else {
            // Tag normal - se pos == 0, é de abertura, senão inline
            if pos == 0 {
                opening.push(format!("{{{}}}", tag_content));
            } else {
                inline.entry(pos).or_default().push(format!("{{{}}}", tag_content));
            }
        }
    }
    
    (opening, closing, inline)
}

/// Filtra tags baseado na configuração
fn filter_tags(
    tags: Vec<(usize, String)>, 
    config: &TextCleanerConfig
) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    
    for (pos, tag_content) in tags {
        let parts: Vec<&str> = tag_content.split(|c| c == '\\' || c == '(').collect();
        
        let mut should_keep = false;
        let mut _is_visual_effect = false;
        let mut _is_karaoke = false;
        
        for part in parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // Verifica se é tag básica
            if config.preserve_basic_formatting {
                for basic_tag in BASIC_FORMATTING_TAGS {
                    if trimmed.starts_with(basic_tag) {
                        should_keep = true;
                        break;
                    }
                }
            }
            
            // Verifica se é karaoke
            for karaoke_tag in KARAOKE_TAGS {
                if trimmed.starts_with(karaoke_tag) {
                    _is_karaoke = true;
                    if config.preserve_karaoke_timing {
                        should_keep = true;
                    }
                    break;
                }
            }
            
            // Verifica se é efeito visual
            for visual_tag in VISUAL_EFFECT_TAGS {
                if trimmed.starts_with(visual_tag) {
                    _is_visual_effect = true;
                    if config.preserve_positioning && 
                       (*visual_tag == "pos" || *visual_tag == "move" || *visual_tag == "org") {
                        should_keep = true;
                    }
                    break;
                }
            }
            
            // Verifica tags customizadas a remover
            for custom_remove in &config.tags_to_remove {
                if trimmed.starts_with(custom_remove) {
                    should_keep = false;
                    break;
                }
            }
        }
        
        // Mantém se é básica, ou se é posicionamento preservado
        if should_keep {
            result.push((pos, tag_content));
        }
    }
    
    result
}

/// Cria uma versão limpa do texto para tradução
pub fn clean_text_for_translation(
    text: &str,
    style: Option<&str>,
    entry_index: usize,
    config: &TextCleanerConfig,
) -> TextMapping {
    // Verifica se o estilo deve ser ignorado
    let should_skip = style.map(|s| {
        config.ignored_styles.iter().any(|ignored| 
            s.to_lowercase().contains(&ignored.to_lowercase())
        )
    }).unwrap_or(false);
    
    if !config.enabled || should_skip {
        // Retorna mapeamento sem limpeza
        return TextMapping {
            entry_index,
            original_text: text.to_string(),
            clean_text: text.to_string(),
            opening_tags: Vec::new(),
            closing_tags: Vec::new(),
            inline_tags: HashMap::new(),
            style: style.map(|s| s.to_string()),
            should_skip_translation: should_skip,
        };
    }
    
    // Extrai tags
    let (clean_text, tags) = extract_tags(text);
    
    // Filtra tags
    let filtered_tags = filter_tags(tags, config);
    
    // Categoriza tags
    let (opening, closing, inline) = categorize_tags(filtered_tags);
    
    TextMapping {
        entry_index,
        original_text: text.to_string(),
        clean_text,
        opening_tags: opening,
        closing_tags: closing,
        inline_tags: inline,
        style: style.map(|s| s.to_string()),
        should_skip_translation: false,
    }
}

/// Limpa todas as entradas de um arquivo de legenda
pub fn clean_subtitle_entries(
    entries: &[(usize, String, Option<String>)], // (index, text, style)
    config: &TextCleanerConfig,
) -> CleanedSubtitle {
    let mut mappings = Vec::new();
    let mut texts_to_translate = Vec::new();
    
    for (index, text, style) in entries {
        let mapping = clean_text_for_translation(text, style.as_deref(), *index, config);
        
        if !mapping.should_skip_translation {
            texts_to_translate.push((*index, mapping.clean_text.clone()));
        }
        
        mappings.push(mapping);
    }
    
    CleanedSubtitle {
        mappings,
        texts_to_translate,
    }
}

/// Reaplica tags no texto traduzido
pub fn reapply_tags(
    translated_text: &str,
    original_mapping: &TextMapping,
    config: &TextCleanerConfig,
) -> String {
    if !config.enabled || original_mapping.should_skip_translation {
        return if original_mapping.should_skip_translation {
            original_mapping.original_text.clone()
        } else {
            translated_text.to_string()
        };
    }
    
    // Converte quebras de volta para \N
    let text_with_newlines = translated_text.replace('\n', "\\N");
    
    let mut result = String::new();
    
    // Adiciona tags de abertura
    for tag in &original_mapping.opening_tags {
        result.push_str(tag);
    }
    
    // Reconstrói o texto com tags inline
    let _char_positions: Vec<(usize, char)> = text_with_newlines.chars().enumerate().collect();
    
    // Ordena posições das tags inline
    let mut sorted_inline: Vec<_> = original_mapping.inline_tags.iter().collect();
    sorted_inline.sort_by_key(|(pos, _)| **pos);
    
    // Se houver tags inline, precisamos inseri-las
    if !sorted_inline.is_empty() {
        let _current_pos = 0;
        
        for (_original_pos, tags) in sorted_inline {
            // Insere tags na posição correspondente
            for tag in tags {
                result.push_str(tag);
            }
        }
        
        result.push_str(&text_with_newlines);
    } else {
        result.push_str(&text_with_newlines);
    }
    
    // Adiciona tags de fechamento
    for tag in &original_mapping.closing_tags {
        result.push_str(tag);
    }
    
    // Se não há tags no resultado mas havia no original, 
    // mantém o formato básico se preserve_basic_formatting estiver ativo
    if result.is_empty() && !original_mapping.opening_tags.is_empty() {
        result = original_mapping.original_text.clone();
    }
    
    // Se o resultado não tem nenhuma tag mas o original tinha,
    // vamos tentar preservar a estrutura básica
    if !result.contains('{') && original_mapping.original_text.contains('{') {
        // Extrai só as tags de formatação básica do original
        let (_original_clean, tags) = extract_tags(&original_mapping.original_text);
        let basic_tags: Vec<_> = tags.into_iter()
            .filter(|(_, tag)| {
                BASIC_FORMATTING_TAGS.iter().any(|basic| tag.starts_with(basic))
            })
            .collect();
        
        if !basic_tags.is_empty() {
            let (opening, closing, _) = categorize_tags(basic_tags);
            result = String::new();
            for tag in opening {
                result.push_str(&tag);
            }
            result.push_str(&text_with_newlines);
            for tag in closing {
                result.push_str(&tag);
            }
        }
    }
    
    result
}

/// Reaplica todas as traduções nos mapeamentos
pub fn reapply_all_tags(
    cleaned: &CleanedSubtitle,
    translations: &HashMap<usize, String>,
    config: &TextCleanerConfig,
) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    
    for mapping in &cleaned.mappings {
        if mapping.should_skip_translation {
            // Mantém o texto original
            results.push((mapping.entry_index, mapping.original_text.clone()));
        } else if let Some(translated) = translations.get(&mapping.entry_index) {
            let with_tags = reapply_tags(translated, mapping, config);
            results.push((mapping.entry_index, with_tags));
        } else {
            // Sem tradução disponível, mantém original
            results.push((mapping.entry_index, mapping.original_text.clone()));
        }
    }
    
    results
}

/// Verifica se um texto tem "lixo pesado" de efeitos visuais
#[allow(dead_code)]
pub fn has_heavy_visual_effects(text: &str) -> bool {
    let (_, tags) = extract_tags(text);
    
    for (_, tag_content) in tags {
        for visual_tag in VISUAL_EFFECT_TAGS {
            if tag_content.contains(&format!("\\{}", visual_tag)) {
                return true;
            }
        }
    }
    
    false
}

/// Análise de um arquivo ASS para estatísticas de "lixo"
pub fn analyze_ass_clutter(entries: &[(String, Option<String>)]) -> AssClutterAnalysis {
    let mut total_lines = 0;
    let mut lines_with_effects = 0;
    let mut lines_with_karaoke = 0;
    let mut lines_with_positioning = 0;
    let mut style_counts: HashMap<String, usize> = HashMap::new();
    
    for (text, style) in entries {
        total_lines += 1;
        
        if let Some(s) = style {
            *style_counts.entry(s.clone()).or_default() += 1;
        }
        
        let (_, tags) = extract_tags(text);
        let mut has_effects = false;
        let mut has_karaoke = false;
        let mut has_positioning = false;
        
        for (_, tag_content) in tags {
            for visual_tag in VISUAL_EFFECT_TAGS {
                if tag_content.contains(&format!("\\{}", visual_tag)) {
                    has_effects = true;
                }
            }
            for karaoke_tag in KARAOKE_TAGS {
                if tag_content.contains(&format!("\\{}", karaoke_tag)) {
                    has_karaoke = true;
                }
            }
            if tag_content.contains("\\pos") || tag_content.contains("\\move") {
                has_positioning = true;
            }
        }
        
        if has_effects { lines_with_effects += 1; }
        if has_karaoke { lines_with_karaoke += 1; }
        if has_positioning { lines_with_positioning += 1; }
    }
    
    AssClutterAnalysis {
        total_lines,
        lines_with_effects,
        lines_with_karaoke,
        lines_with_positioning,
        style_counts,
        estimated_tokens_saved: lines_with_effects * 50, // Estimativa
    }
}

/// Análise de estatísticas de "lixo" em ASS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssClutterAnalysis {
    pub total_lines: usize,
    pub lines_with_effects: usize,
    pub lines_with_karaoke: usize,
    pub lines_with_positioning: usize,
    pub style_counts: HashMap<String, usize>,
    pub estimated_tokens_saved: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tags_simple() {
        let text = r"{\i1}Hello World{\i0}";
        let (clean, tags) = extract_tags(text);
        assert_eq!(clean, "Hello World");
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_extract_tags_complex() {
        let text = r"{\pos(100,200)\c&HFFFFFF&}Hello{\frz10} World";
        let (clean, tags) = extract_tags(text);
        assert_eq!(clean, "Hello World");
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_clean_text_basic() {
        let config = TextCleanerConfig {
            enabled: true,
            preserve_basic_formatting: true,
            ..Default::default()
        };
        
        let mapping = clean_text_for_translation(
            r"{\i1}Hello World{\i0}",
            None,
            1,
            &config,
        );
        
        assert_eq!(mapping.clean_text, "Hello World");
        assert!(mapping.opening_tags.iter().any(|t| t.contains("i1")));
    }

    #[test]
    fn test_clean_text_remove_visual_effects() {
        let config = TextCleanerConfig {
            enabled: true,
            preserve_basic_formatting: false,
            preserve_positioning: false,
            ..Default::default()
        };
        
        let mapping = clean_text_for_translation(
            r"{\pos(640,360)\blur1\c&HFFFFFF&}Hello World",
            None,
            1,
            &config,
        );
        
        assert_eq!(mapping.clean_text, "Hello World");
        assert!(mapping.opening_tags.is_empty());
    }

    #[test]
    fn test_reapply_tags() {
        let config = TextCleanerConfig {
            enabled: true,
            preserve_basic_formatting: true,
            ..Default::default()
        };
        
        let mapping = clean_text_for_translation(
            r"{\i1}Hello World{\i0}",
            None,
            1,
            &config,
        );
        
        let result = reapply_tags("Olá Mundo", &mapping, &config);
        assert!(result.contains("{\\i1}"));
        assert!(result.contains("Olá Mundo"));
    }

    #[test]
    fn test_ignored_styles() {
        let config = TextCleanerConfig {
            enabled: true,
            ignored_styles: vec!["Title".to_string()],
            ..Default::default()
        };
        
        let mapping = clean_text_for_translation(
            "Some text",
            Some("Title1"),
            1,
            &config,
        );
        
        assert!(mapping.should_skip_translation);
    }

    #[test]
    fn test_has_heavy_visual_effects() {
        assert!(has_heavy_visual_effects(r"{\pos(100,200)}Hello"));
        assert!(has_heavy_visual_effects(r"{\blur1\frz10}World"));
        assert!(!has_heavy_visual_effects(r"{\i1}Simple{\i0}"));
    }
}
