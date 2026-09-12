use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::cancel::TranslationCancelState;
use super::subtitle::SubtitleFile;
use super::text_cleaner::{clean_subtitle_entries, reapply_all_tags, TextCleanerConfig};
use super::translator::{
    noop_batch_observer, BatchObserver, BatchTranslationResult, LlmClient, LlmConfig, LlmModel,
    TranslationBatchReport, TranslationProgress, TranslationSettings, TRANSLATION_CANCELLED_ERROR,
};

pub struct TranslationCallbacks {
    pub on_progress: Box<dyn Fn(f64, usize, usize) + Send + Sync>,
    pub on_entry: Box<dyn Fn(usize, String) + Send + Sync>,
    pub on_error: Box<dyn Fn(String, usize) + Send + Sync>,
    /// Per-batch telemetry, so the UI can show one segment per parallel request.
    pub on_batch: BatchObserver,
}

impl Default for TranslationCallbacks {
    fn default() -> Self {
        Self {
            on_progress: Box::new(|_, _, _| {}),
            on_entry: Box::new(|_, _| {}),
            on_error: Box::new(|_, _| {}),
            on_batch: noop_batch_observer(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTranslationResult {
    pub file: SubtitleFile,
    pub progress: TranslationProgress,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedLanguage {
    pub code: String,
    pub name: String,
    pub display_name: String,
}

pub async fn list_llm_models(config: LlmConfig) -> Result<Vec<LlmModel>, String> {
    let client = LlmClient::new(config);
    client.list_models().await
}

pub async fn translate_text(
    config: LlmConfig,
    system_prompt: String,
    text: String,
) -> Result<String, String> {
    let client = LlmClient::new(config);
    client.translate(&system_prompt, &text).await
}

pub async fn translate_subtitle(
    config: LlmConfig,
    system_prompt: String,
    mut file: SubtitleFile,
) -> Result<SubtitleFile, String> {
    let client = LlmClient::new(config);
    let texts = file.extract_texts();
    let translations = client.translate_subtitles(&system_prompt, &texts).await?;
    file.apply_translations(translations);
    Ok(file)
}

pub async fn translate_subtitle_batch(
    config: LlmConfig,
    system_prompt: String,
    file: SubtitleFile,
    start_index: usize,
    batch_size: usize,
) -> Result<BatchTranslationResult, String> {
    let client = LlmClient::new(config);
    let texts = file.extract_texts();
    client
        .translate_batch(&system_prompt, &texts, start_index, batch_size)
        .await
}

/// Traduz arquivo completo com batching, streaming opcional e auto-continue.
#[allow(clippy::too_many_arguments)]
pub async fn translate_subtitle_full(
    config: LlmConfig,
    system_prompt: String,
    mut file: SubtitleFile,
    settings: TranslationSettings,
    file_id: String,
    text_cleaner_config: Option<TextCleanerConfig>,
    cancel_state: &TranslationCancelState,
    callbacks: TranslationCallbacks,
) -> Result<SubtitleTranslationResult, String> {
    let cancel_handle = cancel_state.register(&file_id);
    if cancel_handle.is_cancelled() {
        return Err(TRANSLATION_CANCELLED_ERROR.to_string());
    }

    let client = LlmClient::new(config);

    // `None` means the user turned the cleaner off. `TextCleanerConfig::default()`
    // has `enabled: true`, so it must not be used as the fallback here.
    let use_cleaner = text_cleaner_config
        .as_ref()
        .is_some_and(|config| config.enabled);
    let cleaner_config = text_cleaner_config.unwrap_or_default();

    let (texts_to_translate, cleaned_data, total) = if use_cleaner {
        let entries_with_style: Vec<(usize, String, Option<String>)> = file
            .entries
            .iter()
            .map(|e| {
                (
                    e.index,
                    e.text.clone(),
                    e.metadata.as_ref().and_then(|m| m.style.clone()),
                )
            })
            .collect();

        let cleaned = clean_subtitle_entries(&entries_with_style, &cleaner_config);
        let total = cleaned.mappings.len();
        let texts: Vec<(usize, String)> = cleaned.texts_to_translate.clone();
        (texts, Some(cleaned), total)
    } else {
        let texts = file.extract_texts();
        let total = texts.len();
        (texts, None, total)
    };

    if settings.streaming {
        let on_entry = Arc::new(callbacks.on_entry);
        let on_progress = Arc::new(callbacks.on_progress);
        // Streaming emits one entry at a time; keep the aggregate progress in
        // sync so the queue bar advances instead of jumping to 100% at the end.
        let streamed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let translations = client
            .translate_subtitles_streaming(
                &system_prompt,
                &texts_to_translate,
                settings.batch_size,
                settings.parallel_requests,
                settings.max_retries,
                Some(cancel_handle.flag()),
                Arc::clone(&callbacks.on_batch),
                {
                    let on_entry = Arc::clone(&on_entry);
                    let on_progress = Arc::clone(&on_progress);
                    let streamed = Arc::clone(&streamed);
                    move |entry| {
                        (on_entry)(entry.index, entry.text);
                        let done = streamed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        let percent = if total > 0 {
                            (done as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };
                        (on_progress)(percent.min(100.0), done.min(total), total);
                    }
                },
            )
            .await?;

        if cancel_handle.is_cancelled() {
            return Err(TRANSLATION_CANCELLED_ERROR.to_string());
        }

        let translated_count = translations.len();

        let final_translations = if let Some(ref cleaned) = cleaned_data {
            let translations_map: std::collections::HashMap<usize, String> =
                translations.into_iter().collect();
            reapply_all_tags(cleaned, &translations_map, &cleaner_config)
        } else {
            translations
        };

        file.apply_translations(final_translations);
        normalize_line_breaks(&mut file);

        let progress = TranslationProgress {
            total_entries: total,
            translated_entries: translated_count,
            last_translated_index: if translated_count > 0 {
                translated_count - 1
            } else {
                0
            },
            is_partial: translated_count < total,
            can_continue: translated_count < total,
        };

        (on_progress)(100.0, translated_count, total);

        return Ok(SubtitleTranslationResult {
            file,
            progress,
            error_message: None,
        });
    }

    let on_progress = Arc::new(callbacks.on_progress);
    let on_error = Arc::new(callbacks.on_error);
    let on_entry = Arc::new(callbacks.on_entry);

    let TranslationBatchReport {
        translations,
        progress,
        error_message,
    } = client
        .translate_all_batched(
            &system_prompt,
            &texts_to_translate,
            &settings,
            Some(cancel_handle.flag()),
            {
                let on_progress = Arc::clone(&on_progress);
                move |prog| {
                    let percent = if prog.total_entries > 0 {
                        (prog.translated_entries as f64 / prog.total_entries as f64) * 100.0
                    } else {
                        0.0
                    };
                    (on_progress)(percent, prog.translated_entries, prog.total_entries);
                }
            },
            {
                let on_error = Arc::clone(&on_error);
                move |retry| {
                    (on_error)(retry.error_message.clone(), retry.attempt);
                }
            },
            {
                let on_error = Arc::clone(&on_error);
                move |error| {
                    (on_error)(error.error_message.clone(), 0);
                }
            },
            {
                let on_entry = Arc::clone(&on_entry);
                move |entries: &[(usize, String)]| {
                    for (index, text) in entries {
                        (on_entry)(*index, text.clone());
                    }
                }
            },
            Arc::clone(&callbacks.on_batch),
        )
        .await?;

    if cancel_handle.is_cancelled() {
        return Err(TRANSLATION_CANCELLED_ERROR.to_string());
    }

    let final_translations = if let Some(ref cleaned) = cleaned_data {
        let translations_map: std::collections::HashMap<usize, String> =
            translations.into_iter().collect();
        reapply_all_tags(cleaned, &translations_map, &cleaner_config)
    } else {
        translations
    };

    file.apply_translations(final_translations);
    normalize_line_breaks(&mut file);

    Ok(SubtitleTranslationResult {
        file,
        progress,
        error_message,
    })
}

/// `\N` is an ASS escape, so plain-text formats must carry real line breaks
/// instead. (The ASS serializer turns newlines back into `\N` on its own.)
fn normalize_line_breaks(file: &mut SubtitleFile) {
    use super::subtitle::SubtitleFormat;

    if matches!(file.format, SubtitleFormat::Ass | SubtitleFormat::Ssa) {
        return;
    }
    for entry in &mut file.entries {
        if entry.text.contains(r"\N") || entry.text.contains(r"\n") {
            entry.text = entry.text.replace(r"\N", "\n").replace(r"\n", "\n");
        }
    }
}

pub async fn continue_translation(
    config: LlmConfig,
    system_prompt: String,
    original_file: SubtitleFile,
    mut translated_file: SubtitleFile,
    start_from_index: usize,
    batch_size: usize,
) -> Result<SubtitleTranslationResult, String> {
    let client = LlmClient::new(config);

    let texts = original_file.extract_texts();
    let total = texts.len();

    let result = client
        .translate_batch(&system_prompt, &texts, start_from_index, batch_size)
        .await?;

    translated_file.apply_translations(result.translations);

    let translated_count = translated_file
        .entries
        .iter()
        .zip(original_file.entries.iter())
        .filter(|(t, o)| t.text != o.text)
        .count();

    Ok(SubtitleTranslationResult {
        file: translated_file,
        progress: TranslationProgress {
            total_entries: total,
            translated_entries: translated_count,
            last_translated_index: result.progress.last_translated_index,
            is_partial: translated_count < total,
            can_continue: translated_count < total,
        },
        error_message: None,
    })
}

pub async fn detect_language(
    config: LlmConfig,
    translation_prompt: String,
) -> Result<DetectedLanguage, String> {
    let client = LlmClient::new(config);

    let prompt = format!(
        r#"Given this translation prompt, identify the target language or code-mixed vernacular (e.g., Tenglish, Hinglish, Spanglish).

Translation prompt: [{}]

Respond with ONLY a JSON object in this exact format (no markdown, no extra text):
{{"code": "por", "name": "Portuguese", "displayName": "Portuguese (pt-BR)"}}

Where:
- "code" is the ISO 639-2 three-letter code (e.g., "por" for Portuguese, "eng" for English, "spa" for Spanish). If an exact code doesn't exist for the vernacular, use the code for the dominant/root language.
- "name" is the language name in English
- "displayName" is the language name with regional variant if specified (e.g., "Portuguese (pt-BR)", "English (en-US)", "Spanish (es-MX)")

Examples:
- "Translate to Brazilian Portuguese" -> {{"code": "por", "name": "Portuguese", "displayName": "Portuguese (pt-BR)"}}
- "Translate to English" -> {{"code": "eng", "name": "English", "displayName": "English"}}
- "Traduzir para português do Brasil" -> {{"code": "por", "name": "Portuguese", "displayName": "Portuguese (pt-BR)"}}
- "Translate to Spanglish" -> {{"code": "spa", "name": "Spanish", "displayName": "Spanglish"}}"#,
        translation_prompt
    );

    let response = client.translate(&prompt, "").await?;

    let response = response.trim();
    let cleaned = strip_think_blocks(response);

    let json_str = if cleaned.trim_start().starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    };

    let json_str = json_str.trim();
    let json_str = if let (Some(start), Some(end)) = (json_str.find('{'), json_str.rfind('}')) {
        json_str[start..=end].to_string()
    } else {
        json_str.to_string()
    };

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct LangResponse {
        code: String,
        name: String,
        display_name: String,
    }

    let lang: LangResponse = serde_json::from_str(&json_str).map_err(|e| {
        format!(
            "Failed to parse language detection response: {}. Response was: {}",
            e, response
        )
    })?;

    Ok(DetectedLanguage {
        code: lang.code,
        name: lang.name,
        display_name: lang.display_name,
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_think_blocks_removes_wrapped_content() {
        let raw = "<think>secret</think>{\"code\":\"por\"}";
        assert_eq!(strip_think_blocks(raw), "{\"code\":\"por\"}");
    }

    #[test]
    fn cancel_before_translate_returns_cancelled() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let state = TranslationCancelState::default();
            state.cancel("file-x");

            let err = translate_subtitle_full(
                LlmConfig::default(),
                "prompt".into(),
                SubtitleFile {
                    format: super::super::subtitle::SubtitleFormat::Srt,
                    entries: vec![],
                    headers: None,
                },
                TranslationSettings::default(),
                "file-x".into(),
                None,
                &state,
                TranslationCallbacks::default(),
            )
            .await
            .unwrap_err();

            assert_eq!(err, TRANSLATION_CANCELLED_ERROR);
        });
    }
}
