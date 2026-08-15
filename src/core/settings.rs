use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::paths;
use super::text_cleaner::TextCleanerConfig;
use super::translator::{ApiFormat, LlmConfig, ReasoningEffort, TranslationSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    // API
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_format: ApiFormat,
    #[serde(default)]
    pub headers: Vec<HeaderItem>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub custom_model: String,
    #[serde(default)]
    pub language_detection_model: String,

    // Prompt
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub selected_template_id: Option<String>,

    // Tradução
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_parallel_requests")]
    pub parallel_requests: usize,
    #[serde(default = "default_auto_continue")]
    pub auto_continue: bool,
    #[serde(default = "default_continue_on_error")]
    pub continue_on_error: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default)]
    pub streaming: bool,
    #[serde(default)]
    pub reasoning_effort: ReasoningEffort,
    #[serde(default)]
    pub anthropic_thinking_enabled: bool,
    #[serde(default = "default_anthropic_thinking_budget_tokens")]
    pub anthropic_thinking_budget_tokens: u32,

    // Saída
    #[serde(default = "default_output_mode")]
    pub output_mode: String,
    #[serde(default = "default_mux_language")]
    pub mux_language: String,
    #[serde(default = "default_mux_title")]
    pub mux_title: String,
    #[serde(default)]
    pub separate_output_dir: String,
    #[serde(default)]
    pub cleanup_extracted_subtitles: bool,
    #[serde(default)]
    pub cleanup_mux_artifacts: bool,

    // Interface language
    #[serde(default = "default_language")]
    pub language: String,

    // Text Cleaner (remoção de "lixo" de legendas ASS)
    #[serde(default)]
    pub text_cleaner_enabled: bool,
    #[serde(default = "default_text_cleaner_preserve_basic")]
    pub text_cleaner_preserve_basic_formatting: bool,
    #[serde(default)]
    pub text_cleaner_tags_to_remove: Vec<String>,
    #[serde(default = "default_text_cleaner_ignored_styles")]
    pub text_cleaner_ignored_styles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderItem {
    pub id: String,
    pub key: String,
    pub value: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            api_key: String::new(),
            api_format: ApiFormat::default(),
            headers: Vec::new(),
            model: String::new(),
            custom_model: String::new(),
            language_detection_model: String::new(),
            prompt: String::new(),
            selected_template_id: None,
            batch_size: default_batch_size(),
            parallel_requests: default_parallel_requests(),
            auto_continue: default_auto_continue(),
            continue_on_error: default_continue_on_error(),
            max_retries: default_max_retries(),
            concurrency: default_concurrency(),
            streaming: false,
            reasoning_effort: ReasoningEffort::default(),
            anthropic_thinking_enabled: false,
            anthropic_thinking_budget_tokens: default_anthropic_thinking_budget_tokens(),
            output_mode: default_output_mode(),
            mux_language: default_mux_language(),
            mux_title: default_mux_title(),
            separate_output_dir: String::new(),
            cleanup_extracted_subtitles: false,
            cleanup_mux_artifacts: false,
            language: default_language(),
            text_cleaner_enabled: false,
            text_cleaner_preserve_basic_formatting: true,
            text_cleaner_tags_to_remove: Vec::new(),
            text_cleaner_ignored_styles: vec!["draw".to_string()],
        }
    }
}

impl AppSettings {
    pub fn to_llm_config(&self) -> crate::core::translator::LlmConfig {
        let model = if !self.custom_model.trim().is_empty() {
            self.custom_model.clone()
        } else {
            self.model.clone()
        };

        LlmConfig {
            endpoint: self.base_url.clone(),
            api_key: self.api_key.clone(),
            model,
            api_format: self.api_format.clone(),
            headers: self
                .headers
                .iter()
                .filter(|h| !h.key.is_empty())
                .map(|h| (h.key.clone(), h.value.clone()))
                .collect(),
            reasoning_effort: self.reasoning_effort.clone(),
            anthropic_thinking_enabled: self.anthropic_thinking_enabled,
            anthropic_thinking_budget_tokens: self.anthropic_thinking_budget_tokens,
        }
    }

    /// LLM config for language detection (uses `language_detection_model` when set).
    pub fn to_language_detection_llm_config(&self) -> crate::core::translator::LlmConfig {
        let mut config = self.to_llm_config();
        if !self.language_detection_model.trim().is_empty() {
            config.model = self.language_detection_model.clone();
        }
        config
    }

    pub fn to_translation_settings(&self) -> TranslationSettings {
        TranslationSettings {
            batch_size: self.batch_size,
            parallel_requests: self.parallel_requests,
            auto_continue: self.auto_continue,
            continue_on_error: self.continue_on_error,
            max_retries: self.max_retries,
            streaming: self.streaming,
        }
    }

    pub fn text_cleaner_config(&self) -> crate::core::text_cleaner::TextCleanerConfig {
        TextCleanerConfig {
            enabled: self.text_cleaner_enabled,
            preserve_basic_formatting: self.text_cleaner_preserve_basic_formatting,
            tags_to_remove: self.text_cleaner_tags_to_remove.clone(),
            ignored_styles: self.text_cleaner_ignored_styles.clone(),
            preserve_karaoke_timing: false,
            preserve_positioning: false,
        }
    }
}

fn default_base_url() -> String {
    "http://localhost:8045/v1".to_string()
}

fn default_batch_size() -> usize {
    50
}

fn default_parallel_requests() -> usize {
    1
}

fn default_auto_continue() -> bool {
    true
}

fn default_continue_on_error() -> bool {
    true
}

fn default_max_retries() -> usize {
    3
}

fn default_concurrency() -> usize {
    1
}

fn default_anthropic_thinking_budget_tokens() -> u32 {
    1024
}

fn default_output_mode() -> String {
    "separate".to_string()
}

fn default_mux_language() -> String {
    "por".to_string()
}

fn default_mux_title() -> String {
    "Portuguese".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_text_cleaner_preserve_basic() -> bool {
    true
}

fn default_text_cleaner_ignored_styles() -> Vec<String> {
    vec!["draw".to_string()]
}

pub fn load() -> Result<AppSettings, String> {
    paths::migrate_old_settings();
    let path = paths::settings_path()?;
    load_from(&path)
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = paths::settings_path()?;
    save_to(&path, settings)
}

pub fn load_from(path: &Path) -> Result<AppSettings, String> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read settings: {}", e))?;
    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

pub fn save_to(path: &Path, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create app data dir: {}", e))?;
    }

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write settings: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("translator-settings-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn missing_file_returns_defaults() {
        let path = unique_temp_file("missing.json");
        let settings = load_from(&path).unwrap();
        assert_eq!(settings.base_url, "http://localhost:8045/v1");
        assert_eq!(settings.batch_size, 50);
        assert!(settings.auto_continue);
        assert_eq!(settings.mux_language, "por");
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn load_save_roundtrip_temp_dir() {
        let path = unique_temp_file("settings.json");
        let mut settings = AppSettings::default();
        settings.base_url = "http://example.local/v1".to_string();
        settings.model = "test-model".to_string();
        settings.batch_size = 12;

        save_to(&path, &settings).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.base_url, "http://example.local/v1");
        assert_eq!(loaded.model, "test-model");
        assert_eq!(loaded.batch_size, 12);

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn camel_case_json_compatibility() {
        let json = r#"{
            "baseUrl": "http://x",
            "apiKey": "secret",
            "batchSize": 10,
            "selectedTemplateId": "abc",
            "textCleanerEnabled": true,
            "textCleanerIgnoredStyles": ["draw", "sign"]
        }"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.base_url, "http://x");
        assert_eq!(settings.api_key, "secret");
        assert_eq!(settings.batch_size, 10);
        assert_eq!(settings.selected_template_id.as_deref(), Some("abc"));
        assert!(settings.text_cleaner_enabled);
        assert_eq!(
            settings.text_cleaner_ignored_styles,
            vec!["draw".to_string(), "sign".to_string()]
        );
        // unspecified fields keep defaults
        assert_eq!(settings.max_retries, 3);
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn to_llm_config_prefers_custom_model() {
        let mut settings = AppSettings::default();
        settings.model = "listed".to_string();
        settings.custom_model = "custom".to_string();
        settings.base_url = "http://localhost:8045/v1".to_string();
        settings.headers = vec![
            HeaderItem {
                id: "1".into(),
                key: "X-Test".into(),
                value: "yes".into(),
            },
            HeaderItem {
                id: "2".into(),
                key: String::new(),
                value: "ignored".into(),
            },
        ];

        let config = settings.to_llm_config();
        assert_eq!(config.model, "custom");
        assert_eq!(config.endpoint, "http://localhost:8045/v1");
        assert_eq!(config.headers, vec![("X-Test".into(), "yes".into())]);
    }

    #[test]
    fn text_cleaner_config_maps_fields() {
        let mut settings = AppSettings::default();
        settings.text_cleaner_enabled = true;
        settings.text_cleaner_preserve_basic_formatting = false;
        settings.text_cleaner_tags_to_remove = vec!["blur".into()];
        let cfg = settings.text_cleaner_config();
        assert!(cfg.enabled);
        assert!(!cfg.preserve_basic_formatting);
        assert_eq!(cfg.tags_to_remove, vec!["blur".to_string()]);
        assert!(!cfg.preserve_karaoke_timing);
        assert!(!cfg.preserve_positioning);
    }
}
