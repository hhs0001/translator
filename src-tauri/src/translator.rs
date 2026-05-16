use futures::future::join_all;
use futures::StreamExt;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub const TRANSLATION_CANCELLED_ERROR: &str = "Translation cancelled";

/// LLM API format
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiFormat {
    #[default]
    #[serde(alias = "openai")]
    OpenAI,
    #[serde(alias = "anthropic")]
    Anthropic,
    #[serde(alias = "auto")]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    #[default]
    Default,
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

impl ReasoningEffort {
    fn as_api_value(&self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::None => Some("none"),
            Self::Minimal => Some("minimal"),
            Self::Low => Some("low"),
            Self::Medium => Some("medium"),
            Self::High => Some("high"),
            Self::Xhigh => Some("xhigh"),
        }
    }
}

/// LLM client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub api_format: ApiFormat,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub reasoning_effort: ReasoningEffort,
    #[serde(default)]
    pub anthropic_thinking_enabled: bool,
    #[serde(default = "default_anthropic_thinking_budget_tokens")]
    pub anthropic_thinking_budget_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8317/v1/chat/completions".to_string(),
            api_key: "dummy".to_string(),
            model: "gemini-2.5-pro".to_string(),
            api_format: ApiFormat::default(),
            headers: Vec::new(),
            reasoning_effort: ReasoningEffort::default(),
            anthropic_thinking_enabled: false,
            anthropic_thinking_budget_tokens: default_anthropic_thinking_budget_tokens(),
        }
    }
}

fn default_anthropic_thinking_budget_tokens() -> u32 {
    1024
}

fn detect_api_format(endpoint: &str, configured_format: &ApiFormat) -> ApiFormat {
    if *configured_format != ApiFormat::Auto {
        return configured_format.clone();
    }

    let lower = endpoint.to_lowercase();
    if lower.contains("anthropic") || lower.ends_with("/messages") || lower.contains("/v1/messages")
    {
        ApiFormat::Anthropic
    } else {
        ApiFormat::OpenAI
    }
}

fn normalize_endpoint_for_format(endpoint: &str, format: &ApiFormat) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    match format {
        ApiFormat::Anthropic => {
            if trimmed.ends_with("/messages") {
                trimmed.to_string()
            } else {
                // Add /messages suffix (works for both /v1 and other endpoints)
                format!("{}/messages", trimmed)
            }
        }
        ApiFormat::OpenAI | ApiFormat::Auto => {
            if trimmed.ends_with("/chat/completions") {
                trimmed.to_string()
            } else {
                // Add /chat/completions suffix (works for both /v1 and other endpoints)
                format!("{}/chat/completions", trimmed)
            }
        }
    }
}

/// Translation settings for batch processing

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationSettings {
    pub batch_size: usize,
    #[serde(default = "default_parallel_requests")]
    pub parallel_requests: usize,
    pub auto_continue: bool,
    #[serde(default)]
    pub continue_on_error: bool,
    pub max_retries: usize,
    #[serde(default)]
    pub streaming: bool,
}

fn default_parallel_requests() -> usize {
    1
}

/// Removes <think>...</think> blocks from LLM responses
fn strip_think_blocks(input: &str) -> String {
    let mut output = input.to_string();
    loop {
        let Some(start) = output.find("<think>") else {
            break;
        };
        let Some(end) = output[start + 7..].find("</think>") else {
            break;
        };
        let end = start + 7 + end + 8;
        output.replace_range(start..end, "");
    }
    output
}

impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            batch_size: 50,
            parallel_requests: 1,
            auto_continue: true,
            continue_on_error: false,
            max_retries: 3,
            streaming: false,
        }
    }
}

/// Progress tracking for translation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct TranslationProgress {
    pub total_entries: usize,
    pub translated_entries: usize,
    pub last_translated_index: usize,
    pub is_partial: bool,
    pub can_continue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationRetryInfo {
    pub attempt: usize,
    pub max_retries: usize,
    pub error_message: String,
    pub progress: TranslationProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationErrorInfo {
    pub error_message: String,
    pub progress: TranslationProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationBatchReport {
    pub translations: Vec<(usize, String)>,
    pub progress: TranslationProgress,
    pub error_message: Option<String>,
}

/// Result of a batch translation operation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTranslationResult {
    pub translations: Vec<(usize, String)>,
    pub progress: TranslationProgress,
}

/// Modelo disponível na API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    pub id: String,
    /// OpenAI usa "object", Anthropic usa "type"
    #[serde(default, alias = "type")]
    pub object: String,
    #[serde(default)]
    pub owned_by: Option<String>,
    /// OpenRouter usa "name", Anthropic usa "display_name"
    #[serde(default, alias = "display_name")]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub context_length: Option<u64>,
}

/// Resposta da API /models (OpenAI e Anthropic)
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<LlmModel>,
}

/// Request para chat completion (OpenAI-compatible format)
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Resposta do chat completion
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

// Anthropic API structs
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<AnthropicThinkingRequest>,
}

#[derive(Debug, Serialize)]
struct AnthropicThinkingRequest {
    #[serde(rename = "type")]
    thinking_type: String,
    budget_tokens: u32,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
}

// Streaming response structs (OpenAI SSE format)
#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

/// Event emitted when a single entry is translated (for streaming mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslatedEntryEvent {
    pub index: usize,
    pub text: String,
}

/// Placeholder for newlines in subtitle text during translation
const NEWLINE_PLACEHOLDER: &str = "<<NEWLINE>>";

fn is_cancelled(cancel_flag: &Option<Arc<AtomicBool>>) -> bool {
    cancel_flag
        .as_ref()
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

fn check_cancelled(cancel_flag: &Option<Arc<AtomicBool>>) -> Result<(), String> {
    if is_cancelled(cancel_flag) {
        return Err(TRANSLATION_CANCELLED_ERROR.to_string());
    }
    Ok(())
}

/// Parses a translation line in the format "INDEX|TEXT" and returns (index, translated_text)
fn parse_translation_line(line: &str, placeholder: &str) -> Option<(usize, String)> {
    if line.is_empty() || line.starts_with("```") {
        return None;
    }

    let sep_pos = line.find('|')?;
    let idx_str = &line[..sep_pos];
    let idx = idx_str.trim().parse::<usize>().ok()?;
    let text = line[sep_pos + 1..]
        .replace(placeholder, "\n")
        .replace("\\N", "\n")
        .replace("\\n", "\n");

    Some((idx, text))
}

/// LLM API client
pub struct LlmClient {
    client: Client,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(mut config: LlmConfig) -> Self {
        let detected_format = detect_api_format(&config.endpoint, &config.api_format);
        config.endpoint = normalize_endpoint_for_format(&config.endpoint, &detected_format);
        config.api_format = detected_format;
        Self {
            client: Client::new(),
            config,
        }
    }

    fn extract_ass_tags(text: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let mut rest = text;
        while let Some(start) = rest.find("{\\") {
            let after_start = &rest[start + 1..];
            if let Some(end) = after_start.find('}') {
                let tag_block = &after_start[..end];
                tags.push(format!("{{{}}}", tag_block));
                rest = &after_start[end + 1..];
            } else {
                break;
            }
        }
        tags
    }

    fn normalize_ass_tag(tag: &str) -> String {
        let mut normalized = String::new();
        let mut chars = tag.chars().peekable();
        while let Some(ch) = chars.next() {
            normalized.push(ch.to_ascii_lowercase());
            if ch == '\\' {
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' || next_ch == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        }
        normalized
    }

    fn tags_compatible(original: &str, translated: &str) -> bool {
        let original_tags = Self::extract_ass_tags(original);
        let translated_tags = Self::extract_ass_tags(translated);

        if original_tags.is_empty() && translated_tags.is_empty() {
            return true;
        }

        if original_tags.is_empty() || translated_tags.is_empty() {
            return false;
        }

        let mut counts = HashMap::new();
        for tag in original_tags {
            let normalized = Self::normalize_ass_tag(&tag);
            *counts.entry(normalized).or_insert(0usize) += 1;
        }

        for tag in translated_tags {
            let normalized = Self::normalize_ass_tag(&tag);
            match counts.get_mut(&normalized) {
                Some(count) if *count > 0 => {
                    *count -= 1;
                }
                _ => return false,
            }
        }

        counts.values().all(|count| *count == 0)
    }

    fn apply_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut builder = builder;

        match self.config.api_format {
            ApiFormat::Anthropic => {
                if !self.config.api_key.trim().is_empty() {
                    builder = builder.header("X-Api-Key", &self.config.api_key);
                }
                builder = builder.header("anthropic-version", "2023-06-01");
            }
            ApiFormat::OpenAI | ApiFormat::Auto => {
                if !self.config.api_key.trim().is_empty() {
                    builder =
                        builder.header("Authorization", format!("Bearer {}", self.config.api_key));
                }
            }
        }

        for (key, value) in &self.config.headers {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            builder = builder.header(key, value);
        }

        builder
    }

    /// Lista modelos disponíveis na API
    pub async fn list_models(&self) -> Result<Vec<LlmModel>, String> {
        // Constrói URL base removendo sufixos de endpoint
        let base_url = self
            .config
            .endpoint
            .trim_end_matches("/chat/completions")
            .trim_end_matches("/messages")
            .trim_end_matches('/');
        let models_url = format!("{}/models", base_url);

        let response = self
            .apply_headers(self.client.get(&models_url))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, body));
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse models response: {}", e))?;

        Ok(models_response.data)
    }

    /// Envia mensagem para tradução
    pub async fn translate(
        &self,
        system_prompt: &str,
        subtitle_content: &str,
    ) -> Result<String, String> {
        match self.config.api_format {
            ApiFormat::Anthropic => {
                self.translate_anthropic(system_prompt, subtitle_content)
                    .await
            }
            ApiFormat::OpenAI | ApiFormat::Auto => {
                self.translate_openai(system_prompt, subtitle_content).await
            }
        }
    }

    /// Tradução usando formato OpenAI
    async fn translate_openai(
        &self,
        system_prompt: &str,
        subtitle_content: &str,
    ) -> Result<String, String> {
        // Junta prompt e conteúdo em uma única mensagem (formato do projeto original)
        let full_content = if subtitle_content.is_empty() {
            system_prompt.to_string()
        } else {
            format!("{}\n\n{}", system_prompt, subtitle_content)
        };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: full_content,
        }];

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: Some(false),
            reasoning_effort: self
                .config
                .reasoning_effort
                .as_api_value()
                .map(str::to_string),
        };

        let response = self
            .apply_headers(
                self.client
                    .post(&self.config.endpoint)
                    .header("Content-Type", "application/json"),
            )
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Translation request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Translation API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse translation response: {}", e))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from model".to_string())?;

        Ok(content)
    }

    /// Tradução usando formato Anthropic
    async fn translate_anthropic(
        &self,
        system_prompt: &str,
        subtitle_content: &str,
    ) -> Result<String, String> {
        let user_content = if subtitle_content.is_empty() {
            system_prompt.to_string()
        } else {
            subtitle_content.to_string()
        };

        let system = if subtitle_content.is_empty() {
            None
        } else {
            Some(system_prompt.to_string())
        };

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: if self.config.anthropic_thinking_enabled {
                self.config
                    .anthropic_thinking_budget_tokens
                    .max(1024)
                    .saturating_add(4096)
                    .max(8192)
            } else {
                8192
            },
            system,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_content,
            }],
            thinking: if self.config.anthropic_thinking_enabled {
                Some(AnthropicThinkingRequest {
                    thinking_type: "enabled".to_string(),
                    budget_tokens: self.config.anthropic_thinking_budget_tokens.max(1024),
                })
            } else {
                None
            },
        };

        let response = self
            .apply_headers(
                self.client
                    .post(&self.config.endpoint)
                    .header("Content-Type", "application/json"),
            )
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Translation request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Translation API error {}: {}", status, body));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse translation response: {}", e))?;

        anthropic_response
            .content
            .iter()
            .find_map(|c| {
                if c.content_type == "text" {
                    c.text.clone()
                } else {
                    None
                }
            })
            .ok_or_else(|| "No response from model".to_string())
    }

    /// Translates a single batch with streaming (OpenAI format only)
    /// Emits events as each entry is translated
    async fn translate_streaming_batch(
        &self,
        system_prompt: &str,
        batch: &[(usize, String)],
        batch_index: usize,
        max_retries: usize,
        cancel_flag: Option<Arc<AtomicBool>>,
        original_map: &HashMap<usize, &str>,
        mut on_entry: impl FnMut(TranslatedEntryEvent),
    ) -> Result<Vec<(usize, String)>, String> {
        let mut all_results = Vec::new();
        let mut retries = 0;

        loop {
            check_cancelled(&cancel_flag)?;

            let formatted: String = batch
                .iter()
                .map(|(idx, text)| {
                    let normalized = text
                        .replace("\\N", NEWLINE_PLACEHOLDER)
                        .replace("\\n", NEWLINE_PLACEHOLDER)
                        .replace('\n', NEWLINE_PLACEHOLDER);
                    format!("{}|{}", idx, normalized)
                })
                .collect::<Vec<_>>()
                .join("\n");

            let instruction = format!(
                r#"{}

---
CRITICAL FORMAT INSTRUCTIONS:
1. Return translations in EXACTLY this format: INDEX|TRANSLATED_TEXT
2. Each subtitle must be on its own line: number|translated text
3. The marker {} represents a LINE BREAK within a subtitle. You MUST preserve it exactly as-is in your translation.
   Example input:  5|It's a special event{}that everyone attends
   Example output: 5|É um evento especial{}que todos participam
4. Do NOT remove, split, or modify {} markers - they indicate where line breaks occur in the subtitle display."#,
                system_prompt,
                NEWLINE_PLACEHOLDER,
                NEWLINE_PLACEHOLDER,
                NEWLINE_PLACEHOLDER,
                NEWLINE_PLACEHOLDER
            );

            let full_content = format!("{}\n\n{}", instruction, formatted);

            let messages = vec![ChatMessage {
                role: "user".to_string(),
                content: full_content,
            }];

            let request = ChatRequest {
                model: self.config.model.clone(),
                messages,
                stream: Some(true),
                reasoning_effort: self
                    .config
                    .reasoning_effort
                    .as_api_value()
                    .map(str::to_string),
            };

            let response = match self
                .apply_headers(
                    self.client
                        .post(&self.config.endpoint)
                        .header("Content-Type", "application/json"),
                )
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    retries += 1;
                    if retries > max_retries {
                        return Err(format!("Batch {}: Translation request failed after {} retries: {}", batch_index, max_retries, e));
                    }
                    check_cancelled(&cancel_flag)?;
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                retries += 1;
                if retries > max_retries {
                    return Err(format!("Batch {}: Translation API error {} after {} retries: {}", batch_index, status, max_retries, body));
                }
                check_cancelled(&cancel_flag)?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }

            let mut current_text = String::new();
            let mut buffer = String::new();
            let mut batch_results = Vec::new();

            let mut stream = response.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                if is_cancelled(&cancel_flag) {
                    return Err(TRANSLATION_CANCELLED_ERROR.to_string());
                }
                let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                // Process complete SSE lines
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    if let Some(json_str) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<StreamChunk>(json_str) {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    if let Some(content) = choice.delta.content {
                                        // Process received content character by character
                                        for ch in content.chars() {
                                            if ch == '\n' {
                                                // End of a line - try to parse
                                                let line_content =
                                                    current_text.trim().to_string();
                                                if let Some((idx, text)) =
                                                    parse_translation_line(
                                                        &line_content,
                                                        NEWLINE_PLACEHOLDER,
                                                    )
                                                {
                                                    // Validate ASS tag compatibility
                                                    let should_emit = original_map
                                                        .get(&idx)
                                                        .map(|orig| {
                                                            Self::tags_compatible(orig, &text)
                                                        })
                                                        .unwrap_or(true);

                                                    if should_emit {
                                                        on_entry(TranslatedEntryEvent {
                                                            index: idx,
                                                            text: text.clone(),
                                                        });
                                                        batch_results.push((idx, text));
                                                    }
                                                }
                                                current_text.clear();
                                            } else {
                                                current_text.push(ch);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "Failed to parse SSE chunk: {} - JSON: {}",
                                    e, json_str
                                );
                                let _ = e;
                            }
                        }
                    }
                }
            }

            check_cancelled(&cancel_flag)?;

            // Process last line of the batch if any
            let line_content = current_text.trim().to_string();
            if let Some((idx, text)) =
                parse_translation_line(&line_content, NEWLINE_PLACEHOLDER)
            {
                let should_emit = original_map
                    .get(&idx)
                    .map(|orig| Self::tags_compatible(orig, &text))
                    .unwrap_or(true);

                if should_emit {
                    on_entry(TranslatedEntryEvent {
                        index: idx,
                        text: text.clone(),
                    });
                    batch_results.push((idx, text));
                }
            }

            all_results.extend(batch_results);
            break;
        }

        Ok(all_results)
    }

    /// Translates subtitles with streaming (OpenAI format only)
    /// Emits events as each entry is translated
    /// Uses batching to process in smaller groups with parallel execution
    #[allow(clippy::too_many_arguments)]
    pub async fn translate_subtitles_streaming(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
        batch_size: usize,
        parallel_requests: usize,
        max_retries: usize,
        cancel_flag: Option<Arc<AtomicBool>>,
        on_entry: impl FnMut(TranslatedEntryEvent) + Send + Clone,
    ) -> Result<Vec<(usize, String)>, String> {
        // Create map of original indices for ASS tag validation
        let original_map: HashMap<usize, &str> =
            entries.iter().map(|(i, s)| (*i, s.as_str())).collect();
        let parallel_requests = parallel_requests.max(1);

        // Divide entries em batches
        let batches: Vec<Vec<(usize, String)>> = entries
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len();
        let mut batch_results: Vec<Option<Vec<(usize, String)>>> = vec![None; total_batches];
        let mut current_batch_group = 0;

        // Processa batches em grupos de parallel_requests
        check_cancelled(&cancel_flag)?;
        while current_batch_group * parallel_requests < total_batches {
            check_cancelled(&cancel_flag)?;
            let start_idx = current_batch_group * parallel_requests;
            let end_idx = (start_idx + parallel_requests).min(total_batches);

            // Prepara futures para este grupo de batches
            let mut futures = Vec::new();
            for batch_idx in start_idx..end_idx {
                if batch_results[batch_idx].is_none() {
                    let batch = batches[batch_idx].clone();
                    let cancel_flag = cancel_flag.clone();
                    let original_map = &original_map;
                    let mut on_entry_clone = on_entry.clone();

                    futures.push(async move {
                        let result = self.translate_streaming_batch(
                            system_prompt,
                            &batch,
                            batch_idx,
                            max_retries,
                            cancel_flag,
                            original_map,
                            &mut on_entry_clone,
                        ).await;
                        (batch_idx, result)
                    });
                }
            }

            if futures.is_empty() {
                current_batch_group += 1;
                continue;
            }

            // Executa batches em paralelo
            let results = join_all(futures).await;
            check_cancelled(&cancel_flag)?;

            // Processa resultados
            for (batch_idx, result) in results {
                match result {
                    Ok(translations) => {
                        batch_results[batch_idx] = Some(translations);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            current_batch_group += 1;
        }

        // Coleta e ordena todos os resultados
        let mut all_results: Vec<(usize, String)> =
            batch_results.into_iter().flatten().flatten().collect();
        all_results.sort_by_key(|(idx, _)| *idx);

        if all_results.is_empty() {
            return Err("Failed to parse streaming translation response".to_string());
        }

        Ok(all_results)
    }

    /// Traduz legendas em batch, preservando a estrutura
    pub async fn translate_subtitles(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
    ) -> Result<Vec<(usize, String)>, String> {
        // Placeholder para quebras de linha - único o suficiente para não aparecer em texto normal
        const NEWLINE_PLACEHOLDER: &str = "<<NEWLINE>>";

        // Formata as legendas para envio
        // Formato: INDEX|TEXTO (para preservar mapeamento)
        // Converte \N (ASS) e \n (real) para placeholder para evitar confusão com quebras de linha reais
        let formatted: String = entries
            .iter()
            .map(|(idx, text)| {
                let normalized = text
                    .replace("\\N", NEWLINE_PLACEHOLDER)
                    .replace("\\n", NEWLINE_PLACEHOLDER)
                    .replace('\n', NEWLINE_PLACEHOLDER);
                format!("{}|{}", idx, normalized)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let instruction = format!(
            r#"{}

---
CRITICAL FORMAT INSTRUCTIONS:
1. Return translations in EXACTLY this format: INDEX|TRANSLATED_TEXT
2. Each subtitle must be on its own line: number|translated text
3. The marker {} represents a LINE BREAK within a subtitle. You MUST preserve it exactly as-is in your translation.
   Example input:  5|It's a special event{}that everyone attends
   Example output: 5|É um evento especial{}que todos participam
4. Do NOT remove, split, or modify {} markers - they indicate where line breaks occur in the subtitle display."#,
            system_prompt,
            NEWLINE_PLACEHOLDER,
            NEWLINE_PLACEHOLDER,
            NEWLINE_PLACEHOLDER,
            NEWLINE_PLACEHOLDER
        );

        let response = self.translate(&instruction, &formatted).await?;
        let cleaned_response = strip_think_blocks(&response);

        // Parse da resposta (suporta quebras de linha reais no texto traduzido)
        let mut results = Vec::new();
        let mut current_idx: Option<usize> = None;
        let mut current_text = String::new();

        for raw_line in cleaned_response.lines() {
            let line = raw_line.trim_end();
            if line.is_empty() || line.starts_with("```") {
                continue;
            }

            if let Some(sep_pos) = line.find('|') {
                let idx_str = &line[..sep_pos];
                if let Ok(idx) = idx_str.trim().parse::<usize>() {
                    if let Some(prev_idx) = current_idx.take() {
                        // Converte placeholder de volta para \n (newline real)
                        // Também suporta caso o LLM tenha usado \N ou \n diretamente
                        let text = current_text
                            .replace(NEWLINE_PLACEHOLDER, "\n")
                            .replace("\\N", "\n")
                            .replace("\\n", "\n");
                        results.push((prev_idx, text));
                    }
                    current_idx = Some(idx);
                    current_text = line[sep_pos + 1..].to_string();
                    continue;
                }
            }

            if current_idx.is_some() {
                if !current_text.is_empty() {
                    current_text.push('\n');
                }
                current_text.push_str(line);
            }
        }

        if let Some(prev_idx) = current_idx.take() {
            // Converte placeholder de volta para \n (newline real)
            // Também suporta caso o LLM tenha usado \N ou \n diretamente
            let text = current_text
                .replace(NEWLINE_PLACEHOLDER, "\n")
                .replace("\\N", "\n")
                .replace("\\n", "\n");
            results.push((prev_idx, text));
        }

        if results.is_empty() {
            return Err("Failed to parse translation response".to_string());
        }

        let mut tag_warns = Vec::new();
        for (idx, text) in &results {
            let original_text = entries
                .iter()
                .find(|(entry_idx, _)| entry_idx == idx)
                .map(|(_, entry_text)| entry_text.as_str())
                .unwrap_or("");
            if !Self::tags_compatible(original_text, text) {
                tag_warns.push((*idx, original_text.to_string(), text.clone()));
            }
        }

        if !tag_warns.is_empty() {
            let sample = tag_warns
                .into_iter()
                .take(3)
                .map(|(idx, original, translated)| {
                    format!(
                        "#{}\nORIGINAL: {}\nTRADUZIDO: {}",
                        idx, original, translated
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            return Err(format!(
                "Translated lines contain incompatible ASS tags. Sample:\n{}",
                sample
            ));
        }

        Ok(results)
    }

    /// Traduz um lote específico de legendas (para batch processing)
    /// Permite continuar de um índice específico
    pub async fn translate_batch(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
        start_index: usize,
        batch_size: usize,
    ) -> Result<BatchTranslationResult, String> {
        let total_entries = entries.len();

        // Filtra apenas as entradas a partir do start_index
        let batch: Vec<_> = entries
            .iter()
            .filter(|(idx, _)| *idx >= start_index)
            .take(batch_size)
            .cloned()
            .collect();

        if batch.is_empty() {
            return Ok(BatchTranslationResult {
                translations: Vec::new(),
                progress: TranslationProgress {
                    total_entries,
                    translated_entries: entries
                        .iter()
                        .filter(|(idx, _)| *idx < start_index)
                        .count(),
                    last_translated_index: start_index.saturating_sub(1),
                    is_partial: false,
                    can_continue: false,
                },
            });
        }

        // Traduz o lote
        let translations = self.translate_subtitles(system_prompt, &batch).await?;

        // Calcula progresso
        let last_translated = translations.last().map(|(idx, _)| *idx).unwrap_or(0);
        let translated_count =
            entries.iter().filter(|(idx, _)| *idx < start_index).count() + translations.len();
        let is_partial = translated_count < total_entries;
        let can_continue = is_partial && !translations.is_empty();

        Ok(BatchTranslationResult {
            translations,
            progress: TranslationProgress {
                total_entries,
                translated_entries: translated_count,
                last_translated_index: last_translated,
                is_partial,
                can_continue,
            },
        })
    }

    /// Traduz um único batch (para uso em paralelo)
    async fn translate_single_batch(
        &self,
        system_prompt: &str,
        batch: Vec<(usize, String)>,
        batch_index: usize,
    ) -> (usize, Result<Vec<(usize, String)>, String>) {
        let result = self.translate_subtitles(system_prompt, &batch).await;
        (batch_index, result)
    }

    /// Traduz todas as legendas em batches, com suporte a paralelismo e auto-continue
    #[allow(clippy::too_many_arguments)]
    pub async fn translate_all_batched(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
        settings: &TranslationSettings,
        cancel_flag: Option<Arc<AtomicBool>>,
        mut on_progress: impl FnMut(TranslationProgress),
        mut on_retry: impl FnMut(TranslationRetryInfo),
        mut on_error: impl FnMut(TranslationErrorInfo),
    ) -> Result<TranslationBatchReport, String> {
        let total = entries.len();
        let parallel_requests = settings.parallel_requests.max(1);

        // Divide entries em batches
        let batches: Vec<Vec<(usize, String)>> = entries
            .chunks(settings.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len();
        let mut batch_results: Vec<Option<Vec<(usize, String)>>> = vec![None; total_batches];
        let mut current_batch_group = 0;

        let build_progress = |translations: &Vec<(usize, String)>| -> TranslationProgress {
            let translated_entries = translations.len();
            let last_translated_index = translations.iter().map(|(idx, _)| *idx).max().unwrap_or(0);
            let is_partial = translated_entries < total;
            TranslationProgress {
                total_entries: total,
                translated_entries,
                last_translated_index,
                is_partial,
                can_continue: is_partial,
            }
        };

        // Processa batches em grupos de parallel_requests
        check_cancelled(&cancel_flag)?;
        while current_batch_group * parallel_requests < total_batches {
            check_cancelled(&cancel_flag)?;
            let start_idx = current_batch_group * parallel_requests;
            let end_idx = (start_idx + parallel_requests).min(total_batches);

            // Prepara futures para este grupo de batches
            let mut futures = Vec::new();
            for batch_idx in start_idx..end_idx {
                if batch_results[batch_idx].is_none() {
                    let batch = batches[batch_idx].clone();
                    futures.push(self.translate_single_batch(system_prompt, batch, batch_idx));
                }
            }

            if futures.is_empty() {
                current_batch_group += 1;
                continue;
            }

            // Executa batches em paralelo
            let results = join_all(futures).await;
            check_cancelled(&cancel_flag)?;

            // Processa resultados
            let mut last_error: Option<String> = None;
            let mut failed_batches: Vec<usize> = Vec::new();

            for (batch_idx, result) in results {
                match result {
                    Ok(translations) => {
                        batch_results[batch_idx] = Some(translations);
                    }
                    Err(e) => {
                        last_error = Some(e.clone());
                        failed_batches.push(batch_idx);
                    }
                }
            }

            // Retry para batches que falharam
            for failed_idx in failed_batches {
                let mut retries = 0;
                loop {
                    check_cancelled(&cancel_flag)?;
                    retries += 1;

                    // Calcula progresso atual para callback
                    let current_translations: Vec<_> = batch_results
                        .iter()
                        .filter_map(|r| r.clone())
                        .flatten()
                        .collect();
                    let progress = build_progress(&current_translations);

                    if retries > settings.max_retries {
                        let error_message = format!(
                            "Translation failed after {} retries: {}",
                            settings.max_retries,
                            last_error.clone().unwrap_or_default()
                        );
                        let mut error_progress = progress.clone();
                        error_progress.can_continue =
                            settings.continue_on_error && error_progress.is_partial;

                        on_error(TranslationErrorInfo {
                            error_message: error_message.clone(),
                            progress: error_progress.clone(),
                        });

                        if !settings.continue_on_error {
                            // Coleta traduções bem-sucedidas
                            let mut translations: Vec<(usize, String)> = batch_results
                                .iter()
                                .filter_map(|r| r.clone())
                                .flatten()
                                .collect();
                            translations.sort_by_key(|(idx, _)| *idx);

                            return Ok(TranslationBatchReport {
                                translations,
                                progress: error_progress,
                                error_message: Some(error_message),
                            });
                        }

                        // Se continue_on_error, deixa o batch como None e continua
                        break;
                    }

                    on_retry(TranslationRetryInfo {
                        attempt: retries,
                        max_retries: settings.max_retries,
                        error_message: last_error.clone().unwrap_or_default(),
                        progress,
                    });

                    // Delay antes de retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    check_cancelled(&cancel_flag)?;

                    // Tenta novamente
                    let batch = batches[failed_idx].clone();
                    match self.translate_subtitles(system_prompt, &batch).await {
                        Ok(translations) => {
                            batch_results[failed_idx] = Some(translations);
                            break;
                        }
                        Err(e) => {
                            last_error = Some(e);
                        }
                    }
                }
            }

            // Atualiza progresso após cada grupo
            let current_translations: Vec<_> = batch_results
                .iter()
                .filter_map(|r| r.clone())
                .flatten()
                .collect();
            let progress = build_progress(&current_translations);
            on_progress(progress.clone());

            if !settings.auto_continue && progress.is_partial {
                break;
            }

            current_batch_group += 1;
        }

        // Coleta e ordena todas as traduções
        let mut all_translations: Vec<(usize, String)> =
            batch_results.into_iter().flatten().flatten().collect();
        all_translations.sort_by_key(|(idx, _)| *idx);

        let progress = build_progress(&all_translations);
        Ok(TranslationBatchReport {
            translations: all_translations,
            progress,
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert!(config.endpoint.contains("localhost"));
        assert_eq!(config.api_key, "dummy");
    }
}

#[cfg(test)]
mod translation_integration_tests {
    use super::*;
    use crate::subtitle::{SubtitleEntry, SubtitleFile, SubtitleFormat, SubtitleMetadata};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    fn create_test_config(endpoint: &str) -> LlmConfig {
        LlmConfig {
            endpoint: endpoint.to_string(),
            api_key: "test-key".to_string(),
            model: "test-model".to_string(),
            api_format: ApiFormat::OpenAI,
            headers: vec![],
            reasoning_effort: ReasoningEffort::default(),
            anthropic_thinking_enabled: false,
            anthropic_thinking_budget_tokens: 1024,
        }
    }

    fn create_test_subtitle_file() -> SubtitleFile {
        SubtitleFile {
            format: SubtitleFormat::Srt,
            entries: vec![
                SubtitleEntry {
                    index: 1,
                    start_time: "00:00:01,000".to_string(),
                    end_time: "00:00:04,000".to_string(),
                    text: "Hello world".to_string(),
                    metadata: None,
                },
                SubtitleEntry {
                    index: 2,
                    start_time: "00:00:05,000".to_string(),
                    end_time: "00:00:08,000".to_string(),
                    text: "This is a test".to_string(),
                    metadata: None,
                },
                SubtitleEntry {
                    index: 3,
                    start_time: "00:00:09,000".to_string(),
                    end_time: "00:00:12,000".to_string(),
                    text: "Goodbye".to_string(),
                    metadata: None,
                },
            ],
            headers: None,
        }
    }

    fn create_ass_subtitle_file() -> SubtitleFile {
        SubtitleFile {
            format: SubtitleFormat::Ass,
            entries: vec![
                SubtitleEntry {
                    index: 1,
                    start_time: "0:00:01.00".to_string(),
                    end_time: "0:00:04.00".to_string(),
                    text: r"{\i1}Hello{\i0} World".to_string(),
                    metadata: Some(SubtitleMetadata {
                        style: Some("Default".to_string()),
                        ..Default::default()
                    }),
                },
                SubtitleEntry {
                    index: 2,
                    start_time: "0:00:05.00".to_string(),
                    end_time: "0:00:08.00".to_string(),
                    text: "Normal text".to_string(),
                    metadata: Some(SubtitleMetadata {
                        style: Some("Default".to_string()),
                        ..Default::default()
                    }),
                },
                SubtitleEntry {
                    index: 3,
                    start_time: "0:00:09.00".to_string(),
                    end_time: "0:00:12.00".to_string(),
                    text: r"{\pos(100,200)}Positioned text".to_string(),
                    metadata: Some(SubtitleMetadata {
                        style: Some("Title".to_string()),
                        ..Default::default()
                    }),
                },
            ],
            headers: None,
        }
    }

    fn create_mock_openai_response(translations: &[(&str, &str)]) -> String {
        let lines: Vec<String> = translations
            .iter()
            .map(|(idx, text)| format!("{}|{}", idx, text))
            .collect();
        format!(r#"{{"choices": [{{"message": {{"role": "assistant", "content": "{}"}}}}]}}"#,
            lines.join("\n").replace('\n', "\\n")
        )
    }

    #[test]
    fn test_parse_translation_line_basic() {
        let result = parse_translation_line("1|Hello World", NEWLINE_PLACEHOLDER);
        assert!(result.is_some());
        let (idx, text) = result.unwrap();
        assert_eq!(idx, 1);
        assert_eq!(text, "Hello World");
    }

    #[test]
    fn test_parse_translation_line_with_newlines() {
        let result = parse_translation_line(&format!("2|Line1{}Line2", NEWLINE_PLACEHOLDER), NEWLINE_PLACEHOLDER);
        assert!(result.is_some());
        let (idx, text) = result.unwrap();
        assert_eq!(idx, 2);
        assert_eq!(text, "Line1\nLine2");
    }

    #[test]
    fn test_parse_translation_line_ignores_empty() {
        assert!(parse_translation_line("", NEWLINE_PLACEHOLDER).is_none());
        assert!(parse_translation_line("```json", NEWLINE_PLACEHOLDER).is_none());
    }

    #[test]
    fn test_parse_translation_line_with_ass_newlines() {
        let result = parse_translation_line(r"3|Line1\NLine2", NEWLINE_PLACEHOLDER);
        assert!(result.is_some());
        let (idx, text) = result.unwrap();
        assert_eq!(idx, 3);
        assert_eq!(text, "Line1\nLine2");
    }

    #[test]
    fn test_parse_translation_line_with_real_newlines() {
        let result = parse_translation_line("4|Line1\nLine2", NEWLINE_PLACEHOLDER);
        assert!(result.is_some());
        let (idx, text) = result.unwrap();
        assert_eq!(idx, 4);
        assert_eq!(text, "Line1\nLine2");
    }

    #[test]
    fn test_strip_think_blocks_single() {
        let input = "<think> some thought</think> output text";
        let result = strip_think_blocks(input);
        assert_eq!(result, " output text");
    }

    #[test]
    fn test_strip_think_blocks_multiple() {
        let input = "<think> first</think>text1<think> second</think>text2<think> third</think>";
        let result = strip_think_blocks(input);
        assert_eq!(result, "text1text2");
    }

    #[test]
    fn test_strip_think_blocks_none() {
        let input = "plain text without think blocks";
        let result = strip_think_blocks(input);
        assert_eq!(result, "plain text without think blocks");
    }

    #[test]
    fn test_strip_think_blocks_nested() {
        let input = "<think> outer<think> inner</think> outer continued</think> real content";
        let result = strip_think_blocks(input);
        assert_eq!(result, " outer continued real content");
    }

    #[test]
    fn test_tags_compatible_exact_match() {
        let original = r"{\i1}Hello{\i0}";
        let translated = r"{\i1}Olá{\i0}";
        assert!(LlmClient::tags_compatible(original, translated));
    }

    #[test]
    fn test_tags_compatible_different_tags() {
        let original = r"{\i1}Hello{\i0}";
        let translated = r"{\b1}Olá{\b0}";
        assert!(!LlmClient::tags_compatible(original, translated));
    }

    #[test]
    fn test_tags_compatible_empty_both() {
        assert!(LlmClient::tags_compatible("Hello", "Olá"));
    }

    #[test]
    fn test_tags_compatible_empty_original() {
        assert!(!LlmClient::tags_compatible("Hello", r"{\i1}Olá{\i0}"));
    }

    #[test]
    fn test_tags_compatible_empty_translated() {
        assert!(!LlmClient::tags_compatible(r"{\i1}Hello{\i0}", "Olá"));
    }

    #[test]
    fn test_tags_compatible_case_insensitive() {
        let original = r"{\I1}Hello{\I0}";
        let translated = r"{\i1}Olá{\i0}";
        assert!(LlmClient::tags_compatible(original, translated));
    }

    #[test]
    fn test_tags_compatible_multiple_same_tags() {
        let original = r"{\i1}{\i1}Hello{\i0}{\i0}";
        let translated = r"{\i1}{\i1}Olá{\i0}{\i0}";
        assert!(LlmClient::tags_compatible(original, translated));
    }

    #[test]
    fn test_tags_compatible_mismatched_count() {
        let original = r"{\i1}Hello{\i0}";
        let translated = r"{\i1}{\i1}Olá{\i0}{\i0}";
        assert!(!LlmClient::tags_compatible(original, translated));
    }

    #[test]
    fn test_batch_translation_result_progress() {
        let result = BatchTranslationResult {
            translations: vec![(1, "Translated 1".to_string()), (2, "Translated 2".to_string())],
            progress: TranslationProgress {
                total_entries: 10,
                translated_entries: 2,
                last_translated_index: 2,
                is_partial: true,
                can_continue: true,
            },
        };

        assert_eq!(result.progress.total_entries, 10);
        assert_eq!(result.progress.translated_entries, 2);
        assert!(result.progress.is_partial);
        assert!(result.progress.can_continue);
    }

    #[test]
    fn test_translation_progress_default() {
        let progress = TranslationProgress::default();
        assert_eq!(progress.total_entries, 0);
        assert_eq!(progress.translated_entries, 0);
        assert!(!progress.is_partial);
        assert!(!progress.can_continue);
    }

    #[test]
    fn test_translation_settings_default() {
        let settings = TranslationSettings::default();
        assert_eq!(settings.batch_size, 50);
        assert_eq!(settings.parallel_requests, 1);
        assert!(settings.auto_continue);
        assert!(!settings.continue_on_error);
        assert_eq!(settings.max_retries, 3);
    }

    #[test]
    fn test_translation_batch_report_structure() {
        let report = TranslationBatchReport {
            translations: vec![(1, "Test 1".to_string()), (2, "Test 2".to_string())],
            progress: TranslationProgress {
                total_entries: 5,
                translated_entries: 2,
                last_translated_index: 2,
                is_partial: true,
                can_continue: true,
            },
            error_message: None,
        };

        assert_eq!(report.translations.len(), 2);
        assert!(report.error_message.is_none());
        assert!(report.progress.is_partial);
    }

    #[test]
    fn test_translation_batch_report_with_error() {
        let report = TranslationBatchReport {
            translations: vec![(1, "Test 1".to_string())],
            progress: TranslationProgress {
                total_entries: 5,
                translated_entries: 1,
                last_translated_index: 1,
                is_partial: true,
                can_continue: true,
            },
            error_message: Some("Batch 2 failed".to_string()),
        };

        assert!(report.error_message.is_some());
        assert_eq!(report.error_message.unwrap(), "Batch 2 failed");
    }

    #[test]
    fn test_translation_retry_info() {
        let retry_info = TranslationRetryInfo {
            attempt: 2,
            max_retries: 3,
            error_message: "Connection timeout".to_string(),
            progress: TranslationProgress {
                total_entries: 10,
                translated_entries: 4,
                last_translated_index: 4,
                is_partial: true,
                can_continue: true,
            },
        };

        assert_eq!(retry_info.attempt, 2);
        assert_eq!(retry_info.max_retries, 3);
        assert!(retry_info.progress.can_continue);
    }

    #[test]
    fn test_translation_error_info() {
        let error_info = TranslationErrorInfo {
            error_message: "Max retries exceeded".to_string(),
            progress: TranslationProgress {
                total_entries: 10,
                translated_entries: 5,
                last_translated_index: 5,
                is_partial: true,
                can_continue: false,
            },
        };

        assert!(error_info.progress.is_partial);
        assert!(!error_info.progress.can_continue);
    }

    #[tokio::test]
    async fn test_translate_batch_resume_from_index() {
        let config = create_test_config("http://localhost:8045/v1/chat/completions");
        let client = LlmClient::new(config);

        let entries = vec![
            (1, "First".to_string()),
            (2, "Second".to_string()),
            (3, "Third".to_string()),
            (4, "Fourth".to_string()),
            (5, "Fifth".to_string()),
        ];

        let result = client.translate_batch("Translate", &entries, 3, 10).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.translations.first().map(|(idx, _)| *idx), Some(3));
        assert!(batch_result.translations.iter().all(|(idx, _)| *idx >= 3));
    }

    #[tokio::test]
    async fn test_translate_batch_empty_when_start_exceeds_entries() {
        let config = create_test_config("http://localhost:8045/v1/chat/completions");
        let client = LlmClient::new(config);

        let entries = vec![
            (1, "First".to_string()),
            (2, "Second".to_string()),
        ];

        let result = client.translate_batch("Translate", &entries, 100, 10).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert!(batch_result.translations.is_empty());
        assert_eq!(batch_result.progress.total_entries, 2);
        assert_eq!(batch_result.progress.translated_entries, 2);
    }

    #[tokio::test]
    async fn test_translate_batch_respects_batch_size() {
        let config = create_test_config("http://localhost:8045/v1/chat/completions");
        let client = LlmClient::new(config);

        let entries: Vec<(usize, String)> = (1..=20).map(|i| (i, format!("Text {}", i))).collect();

        let result = client.translate_batch("Translate", &entries, 1, 5).await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert!(batch_result.translations.len() <= 5);
    }

    #[test]
    fn test_cancel_flag_check() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!is_cancelled(&Some(flag.clone())));

        flag.store(true, Ordering::Relaxed);
        assert!(is_cancelled(&Some(flag)));
    }

    #[test]
    fn test_cancel_flag_none() {
        assert!(!is_cancelled(&None));
    }

    #[test]
    fn test_check_cancelled_ok() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(check_cancelled(&Some(flag)).is_ok());
    }

    #[test]
    fn test_check_cancelled_err() {
        let flag = Arc::new(AtomicBool::new(true));
        let result = check_cancelled(&Some(flag));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TRANSLATION_CANCELLED_ERROR);
    }

    #[test]
    fn test_translated_entry_event_serde() {
        let event = TranslatedEntryEvent {
            index: 5,
            text: "Translated text".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: TranslatedEntryEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.index, 5);
        assert_eq!(parsed.text, "Translated text");
    }

    #[test]
    fn test_api_format_detection_openai() {
        let endpoint = "http://localhost:8045/v1/chat/completions";
        let format = detect_api_format(endpoint, &ApiFormat::Auto);
        assert_eq!(format, ApiFormat::OpenAI);
    }

    #[test]
    fn test_api_format_detection_anthropic_from_endpoint() {
        let endpoint = "http://api.anthropic.com/v1/messages";
        let format = detect_api_format(endpoint, &ApiFormat::Auto);
        assert_eq!(format, ApiFormat::Anthropic);
    }

    #[test]
    fn test_api_format_detection_explicit_override() {
        let endpoint = "http://localhost:8045/v1/chat/completions";
        let format = detect_api_format(endpoint, &ApiFormat::Anthropic);
        assert_eq!(format, ApiFormat::Anthropic);
    }

    #[test]
    fn test_normalize_endpoint_openai() {
        let endpoint = "http://localhost:8045/v1";
        let normalized = normalize_endpoint_for_format(endpoint, &ApiFormat::OpenAI);
        assert_eq!(normalized, "http://localhost:8045/v1/chat/completions");
    }

    #[test]
    fn test_normalize_endpoint_anthropic() {
        let endpoint = "http://api.anthropic.com/v1";
        let normalized = normalize_endpoint_for_format(endpoint, &ApiFormat::Anthropic);
        assert_eq!(normalized, "http://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_normalize_endpoint_preserves_existing() {
        let endpoint = "http://localhost:8045/v1/chat/completions";
        let normalized = normalize_endpoint_for_format(endpoint, &ApiFormat::OpenAI);
        assert_eq!(normalized, endpoint);
    }

    #[test]
    fn test_normalize_endpoint_anthropic_preserves_messages() {
        let endpoint = "http://api.anthropic.com/v1/messages";
        let normalized = normalize_endpoint_for_format(endpoint, &ApiFormat::Anthropic);
        assert_eq!(normalized, endpoint);
    }

    #[test]
    fn test_reasoning_effort_as_api_value() {
        assert_eq!(ReasoningEffort::None.as_api_value(), Some("none"));
        assert_eq!(ReasoningEffort::Minimal.as_api_value(), Some("minimal"));
        assert_eq!(ReasoningEffort::Low.as_api_value(), Some("low"));
        assert_eq!(ReasoningEffort::Medium.as_api_value(), Some("medium"));
        assert_eq!(ReasoningEffort::High.as_api_value(), Some("high"));
        assert_eq!(ReasoningEffort::Xhigh.as_api_value(), Some("xhigh"));
        assert_eq!(ReasoningEffort::Default.as_api_value(), None);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(config.endpoint.contains("localhost"));
        assert_eq!(config.api_key, "dummy");
        assert_eq!(config.model, "gemini-2.5-pro");
    }

    #[test]
    fn test_llm_model_serde() {
        let model = LlmModel {
            id: "gpt-4".to_string(),
            object: "model".to_string(),
            owned_by: Some("openai".to_string()),
            name: Some("GPT-4".to_string()),
            description: Some("Powerful model".to_string()),
            context_length: Some(8192),
        };

        let json = serde_json::to_string(&model).unwrap();
        let parsed: LlmModel = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "gpt-4");
        assert_eq!(parsed.context_length, Some(8192));
    }

    #[test]
    fn test_chat_message_serde() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ChatMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.role, "user");
        assert_eq!(parsed.content, "Hello");
    }
}
