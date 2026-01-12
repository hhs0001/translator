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
    BatchTranslationResult, LlmClient, LlmConfig, LlmModel, TranslationBatchReport,
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

    // Aplica traduções de volta
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

/// Traduz arquivo completo com batching e auto-continue
#[tauri::command]
async fn translate_subtitle_full(
    app: tauri::AppHandle,
    config: LlmConfig,
    system_prompt: String,
    mut file: SubtitleFile,
    settings: TranslationSettings,
) -> Result<SubtitleTranslationResult, String> {
    let client = LlmClient::new(config);

    // Extrai textos para tradução
    let texts = file.extract_texts();

    // Traduz com batching
    let TranslationBatchReport {
        translations,
        progress,
        error_message,
    } = client
        .translate_all_batched(
            &system_prompt,
            &texts,
            &settings,
            |progress| {
                let _ = app.emit("translation:progress", progress.clone());
            },
            |retry| {
                let _ = app.emit("translation:retry", retry.clone());
            },
            |error| {
                let _ = app.emit("translation:error", error.clone());
            },
        )
        .await?;

    // Aplica traduções de volta
    file.apply_translations(translations);

    Ok(SubtitleTranslationResult {
        file,
        progress,
        error_message,
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
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default = "default_api_key")]
    api_key: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default)]
    custom_model: String,
    #[serde(default)]
    selected_template_id: Option<String>,
    #[serde(default)]
    prompt: String,
    #[serde(default = "default_batch_size")]
    batch_size: usize,
    #[serde(default = "default_auto_continue")]
    auto_continue: bool,
    #[serde(default = "default_continue_on_error")]
    continue_on_error: bool,
    #[serde(default = "default_max_retries")]
    max_retries: usize,
    #[serde(default = "default_concurrency")]
    concurrency: usize,
    #[serde(default = "default_output_mode")]
    output_mode: String,
    #[serde(default)]
    mux_language: Option<String>,
    #[serde(default)]
    mux_title: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            api_key: default_api_key(),
            headers: Vec::new(),
            model: default_model(),
            custom_model: String::new(),
            selected_template_id: None,
            prompt: String::new(),
            batch_size: default_batch_size(),
            auto_continue: default_auto_continue(),
            continue_on_error: default_continue_on_error(),
            max_retries: default_max_retries(),
            concurrency: default_concurrency(),
            output_mode: default_output_mode(),
            mux_language: None,
            mux_title: None,
        }
    }
}

fn default_base_url() -> String {
    "http://localhost:8317/v1".to_string()
}

fn default_api_key() -> String {
    "dummy".to_string()
}

fn default_model() -> String {
    "gemini-2.5-pro".to_string()
}

fn default_batch_size() -> usize {
    50
}

fn default_auto_continue() -> bool {
    true
}

fn default_continue_on_error() -> bool {
    false
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

fn get_settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    Ok(app_data_dir.join("settings.json"))
}

#[tauri::command]
async fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
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
struct PromptTemplate {
    id: String,
    name: String,
    content: String,
    created_at: u64,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
