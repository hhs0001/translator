mod ffmpeg;
mod subtitle;
mod translator;

use std::fs;
use std::path::Path;

use ffmpeg::SubtitleTrack;
use serde::{Deserialize, Serialize};
use subtitle::{SubtitleFile, SubtitleFormat};
use tauri::{Emitter, Manager};

use translator::{
    ApiFormat, BatchTranslationResult, LlmClient, LlmConfig, LlmModel, TranslationBatchReport,
    TranslationProgress, TranslationSettings,
};


// ============================================================================
// Comandos de Legendas
// ============================================================================

/// Carrega e faz parse de um arquivo de legenda
#[tauri::command]
async fn load_subtitle(path: String) -> Result<SubtitleFile, String> {
    let content = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Tenta detectar encoding e converter para UTF-8
    let (content, _, _) = encoding_rs::UTF_8.decode(&content);
    let content = content.to_string();

    let format =
        SubtitleFile::detect_format(&path).ok_or_else(|| "Unknown subtitle format".to_string())?;

    SubtitleFile::parse(&content, format)
}

/// Salva um arquivo de legenda
#[tauri::command]
async fn save_subtitle(path: String, file: SubtitleFile) -> Result<(), String> {
    let content = file.serialize();
    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

/// Detecta o formato de um arquivo de legenda
#[tauri::command]
fn detect_subtitle_format(filename: String) -> Option<String> {
    SubtitleFile::detect_format(&filename).map(|f| match f {
        SubtitleFormat::Srt => "srt".to_string(),
        SubtitleFormat::Ass => "ass".to_string(),
        SubtitleFormat::Ssa => "ssa".to_string(),
        SubtitleFormat::Vtt => "vtt".to_string(),
    })
}

// ============================================================================
// Comandos de FFmpeg
// ============================================================================

/// Verifica se FFmpeg está instalado
#[tauri::command]
fn check_ffmpeg_installed() -> Result<String, String> {
    ffmpeg::check_ffmpeg()
}

/// Lista faixas de legenda em um vídeo
#[tauri::command]
async fn list_video_subtitle_tracks(video_path: String) -> Result<Vec<SubtitleTrack>, String> {
    ffmpeg::list_subtitle_tracks(&video_path)
}

/// Extrai uma faixa de legenda do vídeo
#[tauri::command]
async fn extract_subtitle_track(
    video_path: String,
    track_index: usize,
    output_path: String,
) -> Result<(), String> {
    ffmpeg::extract_subtitle_track(&video_path, track_index, &output_path)
}

/// Adiciona legenda ao vídeo (mux)
#[tauri::command]
async fn mux_subtitle_to_video(
    video_path: String,
    subtitle_path: String,
    output_path: String,
    language: Option<String>,
    title: Option<String>,
) -> Result<(), String> {
    ffmpeg::mux_subtitle_track(
        &video_path,
        &subtitle_path,
        &output_path,
        language.as_deref(),
        title.as_deref(),
    )
}

// ============================================================================
// Comandos de Tradução (LLM)
// ============================================================================

/// Lista modelos disponíveis na API
#[tauri::command]
async fn list_llm_models(config: LlmConfig) -> Result<Vec<LlmModel>, String> {
    let client = LlmClient::new(config);
    client.list_models().await
}

/// Traduz um arquivo de legenda
#[tauri::command]
async fn translate_subtitle(
    config: LlmConfig,
    system_prompt: String,
    mut file: SubtitleFile,
) -> Result<SubtitleFile, String> {
    let client = LlmClient::new(config);

    // Extrai textos para tradução
    let texts = file.extract_texts();

    // Traduz
    let translations = client.translate_subtitles(&system_prompt, &texts).await?;

    // Apply translations back
    file.apply_translations(translations);

    Ok(file)
}

/// Traduz texto livre (para testes)
#[tauri::command]
async fn translate_text(
    config: LlmConfig,
    system_prompt: String,
    text: String,
) -> Result<String, String> {
    let client = LlmClient::new(config);
    client.translate(&system_prompt, &text).await
}

/// Resultado de tradução em batch com progresso
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleTranslationResult {
    file: SubtitleFile,
    progress: TranslationProgress,
    error_message: Option<String>,
}


/// Traduz um lote específico de legendas (para continue functionality)
#[tauri::command]
async fn translate_subtitle_batch(
    config: LlmConfig,
    system_prompt: String,
    file: SubtitleFile,
    start_index: usize,
    batch_size: usize,
) -> Result<BatchTranslationResult, String> {
    let client = LlmClient::new(config);

    // Extrai textos para tradução
    let texts = file.extract_texts();

    // Traduz apenas o batch
    client
        .translate_batch(&system_prompt, &texts, start_index, batch_size)
        .await
}

/// Evento de progresso com identificador de arquivo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    file_id: String,
    progress: f64,
    translated: usize,
    total: usize,
}

/// Evento de erro com identificador de arquivo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent {
    file_id: String,
    error: String,
    retry_count: usize,
}

/// Evento de streaming de entrada traduzida
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingEntryEvent {
    file_id: String,
    index: usize,
    text: String,
}

/// Traduz arquivo completo com batching e auto-continue
#[tauri::command]
async fn translate_subtitle_full(
    app: tauri::AppHandle,
    config: LlmConfig,
    system_prompt: String,
    mut file: SubtitleFile,
    settings: TranslationSettings,
    file_id: String,
) -> Result<SubtitleTranslationResult, String> {
    let client = LlmClient::new(config);

    // Extract texts for translation
    let texts = file.extract_texts();
    let total = texts.len();

    // If streaming is enabled, use streaming mode
    if settings.streaming {
        let file_id_stream = file_id.clone();
        let app_stream = app.clone();

        let translations = client
            .translate_subtitles_streaming(
                &system_prompt,
                &texts,
                settings.batch_size,
                settings.max_retries,
                move |entry| {
                    // Emit event for each translated entry
                    let _ = app_stream.emit("translation:entry", StreamingEntryEvent {
                        file_id: file_id_stream.clone(),
                        index: entry.index,
                        text: entry.text,
                    });
                },
            )
            .await?;

        let translated_count = translations.len();

        // Apply translations back
        file.apply_translations(translations);

        let progress = TranslationProgress {
            total_entries: total,
            translated_entries: translated_count,
            last_translated_index: if translated_count > 0 { translated_count - 1 } else { 0 },
            is_partial: translated_count < total,
            can_continue: translated_count < total,
        };

        // Emit final progress
        let _ = app.emit("translation:progress", ProgressEvent {
            file_id: file_id.clone(),
            progress: 100.0,
            translated: translated_count,
            total,
        });

        return Ok(SubtitleTranslationResult {
            file,
            progress,
            error_message: None,
        });
    }

    // Default batch mode
    let file_id_progress = file_id.clone();
    let file_id_retry = file_id.clone();
    let file_id_error = file_id.clone();
    let app_progress = app.clone();
    let app_retry = app.clone();

    // Translate with batching
    let TranslationBatchReport {
        translations,
        progress,
        error_message,
    } = client
        .translate_all_batched(
            &system_prompt,
            &texts,
            &settings,
            move |prog| {
                let percent = if prog.total_entries > 0 {
                    (prog.translated_entries as f64 / prog.total_entries as f64) * 100.0
                } else {
                    0.0
                };
                let _ = app_progress.emit("translation:progress", ProgressEvent {
                    file_id: file_id_progress.clone(),
                    progress: percent,
                    translated: prog.translated_entries,
                    total: prog.total_entries,
                });
            },
            move |retry| {
                let _ = app_retry.emit("translation:error", ErrorEvent {
                    file_id: file_id_retry.clone(),
                    error: retry.error_message.clone(),
                    retry_count: retry.attempt,
                });
            },
            move |error| {
                let _ = app.emit("translation:error", ErrorEvent {
                    file_id: file_id_error.clone(),
                    error: error.error_message.clone(),
                    retry_count: 0,
                });
            },
        )
        .await?;

    // Apply translations back
    file.apply_translations(translations);

    Ok(SubtitleTranslationResult {
        file,
        progress,
        error_message,
    })
}


// ============================================================================
// Detecção de Idioma
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetectedLanguage {
    code: String,         // ISO 639-2 (por, eng, spa, etc)
    name: String,         // Nome completo (Portuguese, English, Spanish)
    display_name: String, // Nome para exibição (Portuguese (pt-BR))
}

/// Detecta o idioma alvo de tradução baseado no prompt
#[tauri::command]
async fn detect_language(
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

    // Tenta extrair JSON da resposta
    let response = response.trim();
    let cleaned = strip_think_blocks(response);

    // Remove possíveis marcadores de código markdown
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

    let lang: LangResponse = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse language detection response: {}. Response was: {}", e, response))?;

    Ok(DetectedLanguage {
        code: lang.code,
        name: lang.name,
        display_name: lang.display_name,
    })
}

/// Continua tradução de um arquivo parcialmente traduzido
#[tauri::command]
async fn continue_translation(
    config: LlmConfig,
    system_prompt: String,
    original_file: SubtitleFile,
    mut translated_file: SubtitleFile,
    start_from_index: usize,
    batch_size: usize,
) -> Result<SubtitleTranslationResult, String> {
    let client = LlmClient::new(config);

    // Extrai textos do original
    let texts = original_file.extract_texts();
    let total = texts.len();

    // Traduz apenas a partir do índice especificado
    let result = client
        .translate_batch(&system_prompt, &texts, start_from_index, batch_size)
        .await?;

    // Aplica novas traduções ao arquivo já traduzido
    translated_file.apply_translations(result.translations);

    // Conta quantas entradas foram traduzidas no total
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


// ============================================================================
// Settings da UI
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    // API
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    api_format: ApiFormat,
    #[serde(default)]
    headers: Vec<HeaderItem>,
    #[serde(default)]
    model: String,
    #[serde(default)]
    custom_model: String,
    #[serde(default)]
    language_detection_model: String,

    // Prompt
    #[serde(default)]
    prompt: String,
    #[serde(default)]
    selected_template_id: Option<String>,

    // Tradução
    #[serde(default = "default_batch_size")]
    batch_size: usize,
    #[serde(default = "default_parallel_requests")]
    parallel_requests: usize,
    #[serde(default = "default_auto_continue")]
    auto_continue: bool,
    #[serde(default = "default_continue_on_error")]
    continue_on_error: bool,
    #[serde(default = "default_max_retries")]
    max_retries: usize,
    #[serde(default = "default_concurrency")]
    concurrency: usize,
    #[serde(default)]
    streaming: bool,

    // Saída
    #[serde(default = "default_output_mode")]
    output_mode: String,
    #[serde(default = "default_mux_language")]
    mux_language: String,
    #[serde(default = "default_mux_title")]
    mux_title: String,
    #[serde(default)]
    separate_output_dir: String,

    // Interface language
    #[serde(default = "default_language")]
    language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeaderItem {
    id: String,
    key: String,
    value: String,
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
            output_mode: default_output_mode(),
            mux_language: default_mux_language(),
            mux_title: default_mux_title(),
            separate_output_dir: String::new(),
            language: default_language(),
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

fn get_settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("settings.json"))
}

/// Migra configurações do diretório antigo (com.translator.app) para o novo (com.translator)
fn migrate_old_settings(app: &tauri::AppHandle) {
    let current_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return,
    };

    // Tenta encontrar o diretório antigo
    if let Some(parent) = current_dir.parent() {
        let old_dir = parent.join("com.translator.app");
        if old_dir.exists() && old_dir != current_dir {
            // Migra settings.json
            let old_settings = old_dir.join("settings.json");
            let new_settings = current_dir.join("settings.json");
            if old_settings.exists() && !new_settings.exists() {
                let _ = fs::copy(&old_settings, &new_settings);
            }

            // Migra templates.json
            let old_templates = old_dir.join("templates.json");
            let new_templates = current_dir.join("templates.json");
            if old_templates.exists() && !new_templates.exists() {
                let _ = fs::copy(&old_templates, &new_templates);
            }
        }
    }
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    // Tenta migrar configurações antigas na primeira execução
    migrate_old_settings(&app);

    let path = get_settings_path(&app)?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))
}

// ============================================================================
// Prompt Templates
// ============================================================================


/// Prompt template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptTemplate {
    id: String,
    name: String,
    content: String,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    updated_at: u64,
}

/// Templates storage structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TemplatesData {
    templates: Vec<PromptTemplate>,
}

/// Get templates file path
fn get_templates_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Create directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("templates.json"))
}

/// Load templates from file
#[tauri::command]
async fn load_templates(app: tauri::AppHandle) -> Result<Vec<PromptTemplate>, String> {
    let path = get_templates_path(&app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read templates: {}", e))?;

    let data: TemplatesData =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse templates: {}", e))?;

    Ok(data.templates)
}

/// Save templates to file
#[tauri::command]
async fn save_templates(
    app: tauri::AppHandle,
    templates: Vec<PromptTemplate>,
) -> Result<(), String> {
    let path = get_templates_path(&app)?;

    let data = TemplatesData { templates };
    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize templates: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write templates: {}", e))
}

/// Add a new template
#[tauri::command]
async fn add_template(
    app: tauri::AppHandle,
    name: String,
    content: String,
) -> Result<PromptTemplate, String> {
    let mut templates = load_templates(app.clone()).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis() as u64;

    let template = PromptTemplate {
        id: format!("{}-{}", now, rand_id()),
        name,
        content,
        created_at: now,
        updated_at: now,
    };

    templates.push(template.clone());
    save_templates(app, templates).await?;

    Ok(template)
}

/// Delete a template
#[tauri::command]
async fn delete_template(app: tauri::AppHandle, template_id: String) -> Result<(), String> {
    let mut templates = load_templates(app.clone()).await?;
    templates.retain(|t| t.id != template_id);
    save_templates(app, templates).await
}

/// Update a template
#[tauri::command]
async fn update_template(
    app: tauri::AppHandle,
    template_id: String,
    name: Option<String>,
    content: Option<String>,
) -> Result<PromptTemplate, String> {
    let mut templates = load_templates(app.clone()).await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {}", e))?
        .as_millis() as u64;

    let template = templates
        .iter_mut()
        .find(|t| t.id == template_id)
        .ok_or_else(|| "Template not found".to_string())?;

    if let Some(n) = name {
        template.name = n;
    }
    if let Some(c) = content {
        template.content = c;
    }
    template.updated_at = now;

    let updated = template.clone();
    save_templates(app, templates).await?;

    Ok(updated)
}

/// Generate a simple random ID
fn rand_id() -> String {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:x}", nanos)
}

// ============================================================================
// Utilitários
// ============================================================================

/// Retorna informações sobre um arquivo
#[tauri::command]
fn get_file_info(path: String) -> Result<FileInfo, String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_video = matches!(
        extension.as_str(),
        "mkv" | "mp4" | "avi" | "mov" | "webm" | "m4v" | "ts"
    );

    let is_subtitle = matches!(extension.as_str(), "srt" | "ass" | "ssa" | "vtt");

    Ok(FileInfo {
        path: path.to_string_lossy().to_string(),
        filename,
        extension,
        size: metadata.len(),
        is_video,
        is_subtitle,
    })
}

/// Delete files (for cleanup)
#[tauri::command]
async fn delete_files(paths: Vec<String>) -> Result<Vec<String>, String> {
    let mut deleted = Vec::new();
    let mut errors = Vec::new();

    for path in paths {
        match fs::remove_file(&path) {
            Ok(_) => deleted.push(path),
            Err(e) => errors.push(format!("{}: {}", path, e)),
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Some files failed to delete: {}",
            errors.join(", ")
        ));
    }

    Ok(deleted)
}

/// Backup a file by copying it with .bak extension
#[tauri::command]
async fn backup_file(path: String) -> Result<String, String> {
    let backup_path = format!("{}.bak", path);
    fs::copy(&path, &backup_path).map_err(|e| format!("Failed to backup file: {}", e))?;
    Ok(backup_path)
}

/// Replace a file with another (for replace_original mux mode)
#[tauri::command]
async fn replace_file(source_path: String, target_path: String) -> Result<(), String> {
    // First backup the original
    let backup_path = format!("{}.bak", target_path);
    fs::copy(&target_path, &backup_path)
        .map_err(|e| format!("Failed to backup original: {}", e))?;

    // Remove the original
    fs::remove_file(&target_path).map_err(|e| format!("Failed to remove original: {}", e))?;

    // Rename source to target
    fs::rename(&source_path, &target_path).map_err(|e| format!("Failed to rename file: {}", e))?;

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct FileInfo {
    path: String,
    filename: String,
    extension: String,
    size: u64,
    is_video: bool,
    is_subtitle: bool,
}

// ============================================================================
// App Entry Point
// ============================================================================

/// Retorna o caminho da pasta de dados do app
#[tauri::command]
fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

/// Abre uma pasta no explorador de arquivos
#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Legendas
            load_subtitle,
            save_subtitle,
            detect_subtitle_format,
            // FFmpeg
            check_ffmpeg_installed,
            list_video_subtitle_tracks,
            extract_subtitle_track,
            mux_subtitle_to_video,
// Tradução
            list_llm_models,
            translate_subtitle,
            translate_text,
            translate_subtitle_batch,
            translate_subtitle_full,
            continue_translation,
            detect_language,
            // Settings
            load_settings,
            save_settings,
            // Templates
            load_templates,
            save_templates,

            add_template,
            delete_template,
            update_template,
            // Utilitários
            get_file_info,
            delete_files,
            backup_file,
            replace_file,
            get_app_data_dir,
            open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
