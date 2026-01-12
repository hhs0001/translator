use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};


/// Configuração do cliente LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
}


impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8317/v1/chat/completions".to_string(),
            api_key: "dummy".to_string(),
            model: "gemini-2.5-pro".to_string(),
            headers: Vec::new(),
        }
    }
}

fn normalize_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{}/chat/completions", trimmed)
    } else {
        format!("{}/chat/completions", trimmed)
    }
}

/// Translation settings for batch processing

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationSettings {
    pub batch_size: usize,
    pub auto_continue: bool,
    #[serde(default)]
    pub continue_on_error: bool,
    pub max_retries: usize,
}


impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            batch_size: 50,
            auto_continue: true,
            continue_on_error: false,
            max_retries: 3,
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
    pub object: String,
    #[serde(default)]
    pub owned_by: Option<String>,
}

/// Resposta da API /models
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

/// Cliente para comunicação com a API LLM
pub struct LlmClient {
    client: Client,
    config: LlmConfig,
}

impl LlmClient {
    pub fn new(mut config: LlmConfig) -> Self {
        config.endpoint = normalize_endpoint(&config.endpoint);
        Self {
            client: Client::new(),
            config,
        }
    }

    fn apply_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        let mut builder = builder;

        if !self.config.api_key.trim().is_empty() {
            builder = builder.header(
                "Authorization",
                format!("Bearer {}", self.config.api_key),
            );
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
        // Constrói URL para /models
        let base_url = self
            .config
            .endpoint
            .trim_end_matches("/chat/completions")
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

    /// Traduz legendas em batch, preservando a estrutura
    pub async fn translate_subtitles(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
    ) -> Result<Vec<(usize, String)>, String> {
        // Formata as legendas para envio
        // Formato: INDEX|TEXTO (para preservar mapeamento)
        let formatted: String = entries
            .iter()
            .map(|(idx, text)| format!("{}|{}", idx, text.replace('\n', "\\N")))
            .collect::<Vec<_>>()
            .join("\n");

        let instruction = format!(
            "{}\n\n---\nIMPORTANT: Return the translations in EXACTLY the same format: INDEX|TRANSLATED_TEXT\nEach line should be: number|translated text\nPreserve \\N as line breaks within subtitles.",
            system_prompt
        );

        let response = self.translate(&instruction, &formatted).await?;

        // Parse da resposta
        let mut results = Vec::new();
        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Tenta encontrar o separador |
            if let Some(sep_pos) = line.find('|') {
                let idx_str = &line[..sep_pos];
                let text = &line[sep_pos + 1..];

                if let Ok(idx) = idx_str.trim().parse::<usize>() {
                    // Converte \N de volta para quebras de linha
                    let text = text.replace("\\N", "\n").replace("\\n", "\n");
                    results.push((idx, text));
                }
            }
        }

        if results.is_empty() {
            return Err("Failed to parse translation response".to_string());
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

    /// Traduz todas as legendas em batches, com suporte a auto-continue
    pub async fn translate_all_batched(
        &self,
        system_prompt: &str,
        entries: &[(usize, String)],
        settings: &TranslationSettings,
        mut on_progress: impl FnMut(TranslationProgress),
        mut on_retry: impl FnMut(TranslationRetryInfo),
        mut on_error: impl FnMut(TranslationErrorInfo),
    ) -> Result<TranslationBatchReport, String> {
        let mut all_translations: Vec<(usize, String)> = Vec::new();
        let mut current_index = 1; // Legendas geralmente começam em 1
        let total = entries.len();
        let mut retries = 0;

        let mut build_progress = |translations: &Vec<(usize, String)>| -> TranslationProgress {
            let translated_entries = translations.len();
            let last_translated_index = translations
                .last()
                .map(|(idx, _)| *idx)
                .unwrap_or(0);
            let is_partial = translated_entries < total;
            TranslationProgress {
                total_entries: total,
                translated_entries,
                last_translated_index,
                is_partial,
                can_continue: is_partial,
            }
        };

        loop {
            match self
                .translate_batch(system_prompt, entries, current_index, settings.batch_size)
                .await
            {
                Ok(result) => {
                    retries = 0; // Reset retries on success

                    if result.translations.is_empty() {
                        break;
                    }

                    // Adiciona traduções
                    all_translations.extend(result.translations);

                    // Atualiza progresso
                    let progress = build_progress(&all_translations);
                    on_progress(progress.clone());

                    // Verifica se completou
                    if !progress.is_partial {
                        break;
                    }

                    // Continua do próximo índice
                    if settings.auto_continue {
                        current_index = progress.last_translated_index + 1;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    retries += 1;
                    let progress = build_progress(&all_translations);

                    if retries >= settings.max_retries {
                        let error_message = format!(
                            "Translation failed after {} retries: {}",
                            settings.max_retries, e
                        );
                        let mut error_progress = progress.clone();
                        error_progress.can_continue =
                            settings.continue_on_error && error_progress.is_partial;

                        on_error(TranslationErrorInfo {
                            error_message: error_message.clone(),
                            progress: error_progress.clone(),
                        });

                        return Ok(TranslationBatchReport {
                            translations: all_translations,
                            progress: error_progress,
                            error_message: Some(error_message),
                        });
                    }

                    on_retry(TranslationRetryInfo {
                        attempt: retries,
                        max_retries: settings.max_retries,
                        error_message: e,
                        progress,
                    });

                    // Pequeno delay antes de retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }

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
