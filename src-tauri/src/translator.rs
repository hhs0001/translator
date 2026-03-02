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
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8317/v1/chat/completions".to_string(),
            api_key: "dummy".to_string(),
            model: "gemini-2.5-pro".to_string(),
            api_format: ApiFormat::default(),
            headers: Vec::new(),
        }
    }
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
            } else if trimmed.ends_with("/v1") {
                format!("{}/messages", trimmed)
            } else {
                format!("{}/messages", trimmed)
            }
        }
        ApiFormat::OpenAI | ApiFormat::Auto => {
            if trimmed.ends_with("/chat/completions") {
                trimmed.to_string()
            } else if trimmed.ends_with("/v1") {
                format!("{}/chat/completions", trimmed)
            } else {
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
pub struct TranslationProgress {
    pub total_entries: usize,
    pub translated_entries: usize,
    pub last_translated_index: usize,
    pub is_partial: bool,
    pub can_continue: bool,
}

impl Default for TranslationProgress {
    fn default() -> Self {
        Self {
            total_entries: 0,
            translated_entries: 0,
            last_translated_index: 0,
            is_partial: false,
            can_continue: false,
        }
    }
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
    text: String,
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
            max_tokens: 8192,
            system,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_content,
            }],
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
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| "No response from model".to_string())
    }

    /// Translates subtitles with streaming (OpenAI format only)
    /// Emits events as each entry is translated
    /// Uses batching to process in smaller groups
    pub async fn translate_subtitles_streaming(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
        batch_size: usize,
        max_retries: usize,
        cancel_flag: Option<Arc<AtomicBool>>,
        mut on_entry: impl FnMut(TranslatedEntryEvent),
    ) -> Result<Vec<(usize, String)>, String> {
        // Create map of original indices for ASS tag validation (using references to avoid cloning)
        let original_map: HashMap<usize, &str> =
            entries.iter().map(|(i, s)| (*i, s.as_str())).collect();
        let mut all_results = Vec::new();

        // Process in batches
        for batch in entries.chunks(batch_size) {
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
            };

            // Retry loop for streaming batch
            let mut attempt = 0;
            let batch_results = loop {
                check_cancelled(&cancel_flag)?;
                attempt += 1;

                let response = self
                    .apply_headers(
                        self.client
                            .post(&self.config.endpoint)
                            .header("Content-Type", "application/json"),
                    )
                    .json(&request)
                    .send()
                    .await;

                let response = match response {
                    Ok(r) => r,
                    Err(e) => {
                        if attempt > max_retries {
                            return Err(format!("Translation request failed: {}", e));
                        }
                        check_cancelled(&cancel_flag)?;
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                };

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    if attempt > max_retries {
                        return Err(format!("Translation API error {}: {}", status, body));
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
                                    // Log invalid JSON chunks in debug mode for troubleshooting
                                    #[cfg(debug_assertions)]
                                    eprintln!(
                                        "Failed to parse SSE chunk: {} - JSON: {}",
                                        e, json_str
                                    );
                                    let _ = e; // Suppress unused warning in release
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

                break batch_results;
            };

            all_results.extend(batch_results);
        }

        if all_results.is_empty() {
            return Err("Failed to parse streaming translation response".to_string());
        }

        all_results.sort_by_key(|(idx, _)| *idx);
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
