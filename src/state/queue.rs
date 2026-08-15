use std::path::{Path, PathBuf};

use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{AppContext, Context, Entity, EventEmitter, WeakEntity};
use uuid::Uuid;

use crate::core::cancel::TranslationCancelState;
use crate::core::ffmpeg::{self, SubtitleTrack};
use crate::core::files::{self, FileInfo};
use crate::core::subtitle::{SubtitleEntry, SubtitleFile};
use crate::core::translate::{self, DetectedLanguage, TranslationCallbacks};
use crate::core::translator::TRANSLATION_CANCELLED_ERROR;
use crate::i18n::{self, Language};
use crate::state::logs::{LogLevel, LogsState};
use crate::state::settings::SettingsState;

const PROCESSING: &[FileStatus] = &[
    FileStatus::Extracting,
    FileStatus::Translating,
    FileStatus::DetectingLanguage,
    FileStatus::Saving,
    FileStatus::Muxing,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileKind {
    Subtitle,
    Video,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileStatus {
    Pending,
    Extracting,
    Translating,
    DetectingLanguage,
    Saving,
    Muxing,
    Paused,
    Cancelled,
    Completed,
    Error,
}

impl FileStatus {
    pub fn is_processing(self) -> bool {
        PROCESSING.contains(&self)
    }

    pub fn label(self, lang: Language) -> String {
        let key = match self {
            Self::Pending => "translation.status.pending",
            Self::Extracting => "translation.status.extracting",
            Self::Translating => "translation.status.translating",
            Self::DetectingLanguage => "translation.status.detecting_language",
            Self::Saving => "translation.status.saving",
            Self::Muxing => "translation.status.muxing",
            Self::Paused => "translation.status.paused",
            Self::Cancelled => "translation.status.cancelled",
            Self::Completed => "translation.status.completed",
            Self::Error => "translation.status.error",
        };
        i18n::t(lang, key)
    }
}

#[derive(Clone)]
pub struct QueueFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: FileKind,
    pub status: FileStatus,
    pub progress: f32,
    pub total_lines: usize,
    pub translated_lines: usize,
    pub error: Option<String>,
    pub original: Option<SubtitleFile>,
    pub translated_entries: Option<Vec<SubtitleEntry>>,
    pub selected_track_index: Option<usize>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub is_loading_tracks: bool,
    pub extracted_subtitle_path: Option<String>,
    pub detected_language: Option<DetectedLanguage>,
    pub output_subtitle_path: Option<String>,
    pub output_video_path: Option<String>,
}

pub struct QueueState {
    pub files: Vec<QueueFile>,
    pub current_file_id: Option<String>,
    pub is_translating: bool,
    pub is_paused: bool,
    cancel: TranslationCancelState,
    settings: WeakEntity<SettingsState>,
    logs: WeakEntity<LogsState>,
}

pub struct QueueChanged;

impl EventEmitter<QueueChanged> for QueueState {}

enum WorkerEvent {
    Status {
        status: FileStatus,
        error: Option<String>,
    },
    Progress {
        percent: f32,
        done: usize,
        total: usize,
    },
    Entry {
        index: usize,
        text: String,
    },
    Original {
        file: SubtitleFile,
    },
    Translated {
        file: SubtitleFile,
    },
    ExtractedPath(String),
    Language(DetectedLanguage),
    OutputSubtitle(String),
    OutputVideo(String),
    Log {
        level: LogLevel,
        key: &'static str,
        vars: Vec<(String, String)>,
        file_name: String,
    },
    Done,
}

impl QueueState {
    pub fn new(
        settings: Entity<SettingsState>,
        logs: Entity<LogsState>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self {
            files: Vec::new(),
            current_file_id: None,
            is_translating: false,
            is_paused: false,
            cancel: TranslationCancelState::default(),
            settings: settings.downgrade(),
            logs: logs.downgrade(),
        }
    }

    pub fn pending_count(&self) -> usize {
        self.files
            .iter()
            .filter(|f| f.status == FileStatus::Pending)
            .count()
    }

    pub fn processing_count(&self) -> usize {
        self.files
            .iter()
            .filter(|f| f.status.is_processing())
            .count()
    }

    pub fn current_file(&self) -> Option<&QueueFile> {
        let id = self.current_file_id.as_deref()?;
        self.files.iter().find(|f| f.id == id)
    }

    pub fn file(&self, id: &str) -> Option<&QueueFile> {
        self.files.iter().find(|f| f.id == id)
    }

    pub fn file_mut(&mut self, id: &str) -> Option<&mut QueueFile> {
        self.files.iter_mut().find(|f| f.id == id)
    }

    pub fn add_paths(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let mut added = 0;
        for path in paths {
            match files::get_file_info(&path) {
                Ok(info) if info.is_subtitle || info.is_video => {
                    self.push_file(info, cx);
                    added += 1;
                }
                _ => {}
            }
        }
        if added > 0 {
            let lang = self.language(cx);
            self.log(
                cx,
                LogLevel::Info,
                i18n::tf(
                    lang,
                    "logMessages.filesAdded",
                    &[("count", &added.to_string())],
                ),
                None,
            );
        }
        cx.emit(QueueChanged);
        cx.notify();
    }

    fn push_file(&mut self, info: FileInfo, cx: &mut Context<Self>) {
        let kind = if info.is_video {
            FileKind::Video
        } else {
            FileKind::Subtitle
        };
        let id = Uuid::new_v4().to_string();
        let file = QueueFile {
            id: id.clone(),
            name: info.filename,
            path: info.path,
            kind,
            status: FileStatus::Pending,
            progress: 0.0,
            total_lines: 0,
            translated_lines: 0,
            error: None,
            original: None,
            translated_entries: None,
            selected_track_index: None,
            subtitle_tracks: Vec::new(),
            is_loading_tracks: kind == FileKind::Video,
            extracted_subtitle_path: None,
            detected_language: None,
            output_subtitle_path: None,
            output_video_path: None,
        };
        self.files.push(file);
        if self.current_file_id.is_none() {
            self.current_file_id = Some(id.clone());
        }
        if kind == FileKind::Video {
            self.load_video_tracks(id, cx);
        }
    }

    pub fn select_file(&mut self, id: String, cx: &mut Context<Self>) {
        self.current_file_id = Some(id);
        cx.notify();
    }

    pub fn remove_file(&mut self, id: &str, cx: &mut Context<Self>) {
        self.cancel.cancel(id);
        self.files.retain(|f| f.id != id);
        if self.current_file_id.as_deref() == Some(id) {
            self.current_file_id = self.files.first().map(|f| f.id.clone());
        }
        cx.emit(QueueChanged);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.cancel.cancel_all();
        self.files.clear();
        self.current_file_id = None;
        self.is_translating = false;
        self.is_paused = false;
        cx.emit(QueueChanged);
        cx.notify();
    }

    pub fn set_selected_track(&mut self, id: &str, index: usize, cx: &mut Context<Self>) {
        if let Some(file) = self.file_mut(id) {
            file.selected_track_index = Some(index);
        }
        cx.notify();
    }

    pub fn set_all_video_tracks(&mut self, index: usize, cx: &mut Context<Self>) {
        for file in &mut self.files {
            if file.kind == FileKind::Video {
                file.selected_track_index = Some(index);
            }
        }
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Info,
            i18n::tf(
                lang,
                "logMessages.trackSelectedForAll",
                &[("index", &(index + 1).to_string())],
            ),
            None,
        );
        cx.notify();
    }

    pub fn update_translated_line(
        &mut self,
        file_id: &str,
        entry_index: usize,
        text: String,
        cx: &mut Context<Self>,
    ) {
        if let Some(file) = self.file_mut(file_id) {
            if let Some(entries) = file.translated_entries.as_mut() {
                if let Some(entry) = entries.iter_mut().find(|e| e.index == entry_index) {
                    entry.text = text;
                }
            }
        }
        cx.notify();
    }

    pub fn start(&mut self, cx: &mut Context<Self>) {
        if self.files.is_empty() {
            return;
        }
        self.is_translating = true;
        self.is_paused = false;
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Info,
            i18n::t(lang, "logMessages.startingTranslation"),
            None,
        );
        self.pump(cx);
        cx.notify();
    }

    pub fn pause(&mut self, cx: &mut Context<Self>) {
        self.is_paused = true;
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Warning,
            i18n::t(lang, "logMessages.translationPaused"),
            None,
        );
        cx.notify();
    }

    pub fn resume(&mut self, cx: &mut Context<Self>) {
        self.is_paused = false;
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Info,
            i18n::t(lang, "logMessages.translationResumed"),
            None,
        );
        self.pump(cx);
        cx.notify();
    }

    pub fn cancel_file(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(file) = self.file(id) else {
            return;
        };
        if matches!(
            file.status,
            FileStatus::Completed | FileStatus::Error | FileStatus::Cancelled
        ) {
            return;
        }
        let name = file.name.clone();
        if file.status.is_processing() {
            self.cancel.cancel(id);
        }
        if let Some(file) = self.file_mut(id) {
            file.status = FileStatus::Cancelled;
        }
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Warning,
            i18n::tf(
                lang,
                "logMessages.translationCancelledFor",
                &[("fileName", &name)],
            ),
            Some(name),
        );
        cx.notify();
    }

    pub fn cancel_all(&mut self, cx: &mut Context<Self>) {
        self.cancel.cancel_all();
        for file in &mut self.files {
            if file.status == FileStatus::Pending || file.status.is_processing() {
                file.status = FileStatus::Cancelled;
            }
        }
        self.is_translating = false;
        self.is_paused = false;
        self.current_file_id = None;
        let lang = self.language(cx);
        self.log(
            cx,
            LogLevel::Warning,
            i18n::t(lang, "logMessages.translationCancelledForAll"),
            None,
        );
        cx.notify();
    }

    fn pump(&mut self, cx: &mut Context<Self>) {
        if !self.is_translating || self.is_paused {
            return;
        }
        let concurrency = self
            .settings
            .upgrade()
            .map(|s| s.read(cx).settings.concurrency.max(1))
            .unwrap_or(1);
        let active = self.processing_count();
        let pending: Vec<String> = self
            .files
            .iter()
            .filter(|f| f.status == FileStatus::Pending)
            .map(|f| f.id.clone())
            .collect();

        if active == 0 && pending.is_empty() {
            self.is_translating = false;
            self.current_file_id = self.files.first().map(|f| f.id.clone());
            let lang = self.language(cx);
            self.log(
                cx,
                LogLevel::Success,
                i18n::t(lang, "logMessages.allFilesProcessed"),
                None,
            );
            cx.notify();
            return;
        }

        let slots = concurrency.saturating_sub(active);
        for id in pending.into_iter().take(slots) {
            self.start_file(id, cx);
        }
    }

    fn start_file(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(file) = self.file(&id).cloned() else {
            return;
        };
        self.current_file_id = Some(id.clone());
        if let Some(file) = self.file_mut(&id) {
            file.status = if file.kind == FileKind::Video {
                FileStatus::Extracting
            } else {
                FileStatus::Translating
            };
            file.error = None;
        }
        cx.notify();

        let Some(settings_entity) = self.settings.upgrade() else {
            return;
        };
        let settings = settings_entity.read(cx).settings.clone();
        let cancel = self.cancel.clone();
        let file_id = id.clone();

        cx.spawn(async move |this, cx| {
            let (tx, mut rx) = mpsc::unbounded();
            crate::core::runtime::spawn(async move {
                process_file(file, settings, cancel, tx).await;
            });

            while let Some(event) = rx.next().await {
                let file_id = file_id.clone();
                let _ = this.update(cx, |this, cx| {
                    this.apply_event(&file_id, event, cx);
                });
            }

            let _ = this.update(cx, |this, cx| {
                this.pump(cx);
            });
        })
        .detach();
    }

    fn apply_event(&mut self, file_id: &str, event: WorkerEvent, cx: &mut Context<Self>) {
        match event {
            WorkerEvent::Status { status, error } => {
                if let Some(file) = self.file_mut(file_id) {
                    if file.status != FileStatus::Cancelled {
                        file.status = status;
                        file.error = error;
                    }
                }
            }
            WorkerEvent::Progress {
                percent,
                done,
                total,
            } => {
                if let Some(file) = self.file_mut(file_id) {
                    file.progress = percent;
                    file.translated_lines = done;
                    file.total_lines = total;
                }
            }
            WorkerEvent::Entry { index, text } => {
                if let Some(file) = self.file_mut(file_id) {
                    let entries = file.translated_entries.get_or_insert_with(|| {
                        file.original
                            .as_ref()
                            .map(|s| s.entries.clone())
                            .unwrap_or_default()
                    });
                    if let Some(entry) = entries.iter_mut().find(|e| e.index == index) {
                        entry.text = text;
                    }
                }
            }
            WorkerEvent::Original { file: subtitle } => {
                if let Some(file) = self.file_mut(file_id) {
                    file.total_lines = subtitle.entries.len();
                    file.original = Some(subtitle);
                }
            }
            WorkerEvent::Translated { file: subtitle } => {
                if let Some(file) = self.file_mut(file_id) {
                    file.translated_entries = Some(subtitle.entries);
                    file.progress = 100.0;
                }
            }
            WorkerEvent::ExtractedPath(path) => {
                if let Some(file) = self.file_mut(file_id) {
                    file.extracted_subtitle_path = Some(path);
                }
            }
            WorkerEvent::Language(lang) => {
                if let Some(file) = self.file_mut(file_id) {
                    file.detected_language = Some(lang);
                }
            }
            WorkerEvent::OutputSubtitle(path) => {
                if let Some(file) = self.file_mut(file_id) {
                    file.output_subtitle_path = Some(path);
                }
            }
            WorkerEvent::OutputVideo(path) => {
                if let Some(file) = self.file_mut(file_id) {
                    file.output_video_path = Some(path);
                }
            }
            WorkerEvent::Log {
                level,
                key,
                vars,
                file_name,
            } => {
                let lang = self.language(cx);
                let pairs: Vec<(&str, &str)> =
                    vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                let message = i18n::tf(lang, key, &pairs);
                self.log(cx, level, message, Some(file_name));
            }
            WorkerEvent::Done => {}
        }
        cx.emit(QueueChanged);
        cx.notify();
    }

    fn load_video_tracks(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(path) = self.file(&id).map(|f| f.path.clone()) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { ffmpeg::list_subtitle_tracks(&path) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if let Some(file) = this.file_mut(&id) {
                    file.is_loading_tracks = false;
                    match result {
                        Ok(tracks) => {
                            if file.selected_track_index.is_none() && !tracks.is_empty() {
                                file.selected_track_index = Some(0);
                            }
                            file.subtitle_tracks = tracks;
                        }
                        Err(_) => file.subtitle_tracks = Vec::new(),
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn language(&self, cx: &Context<Self>) -> Language {
        self.settings
            .upgrade()
            .map(|s| s.read(cx).language())
            .unwrap_or_default()
    }

    fn log(&self, cx: &mut Context<Self>, level: LogLevel, message: String, file: Option<String>) {
        if let Some(logs) = self.logs.upgrade() {
            logs.update(cx, |logs, cx| {
                logs.add(level, message, file, cx);
            });
        }
    }
}

async fn process_file(
    file: QueueFile,
    settings: crate::core::settings::AppSettings,
    cancel: TranslationCancelState,
    tx: mpsc::UnboundedSender<WorkerEvent>,
) {
    let send = |event: WorkerEvent| {
        let _ = tx.unbounded_send(event);
    };
    let log = |level, key, vars: Vec<(&str, String)>, name: &str| {
        let _ = tx.unbounded_send(WorkerEvent::Log {
            level,
            key,
            vars: vars.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            file_name: name.to_string(),
        });
    };

    if is_cancelled(&file.id, &cancel, &tx) {
        return;
    }

    let mut subtitle_path = file.path.clone();
    let mut extracted: Option<String> = None;

    if file.kind == FileKind::Video {
        send(WorkerEvent::Status {
            status: FileStatus::Extracting,
            error: None,
        });
        log(
            LogLevel::Info,
            "logMessages.extractingSubtitle",
            vec![("fileName", file.name.clone())],
            &file.name,
        );
        let track = file.selected_track_index.unwrap_or(0);
        let output = replace_extension(&file.path, "extracted.ass");
        if let Err(err) = ffmpeg::extract_subtitle_track(&file.path, track, &output) {
            fail(&tx, err);
            return;
        }
        subtitle_path = output.clone();
        extracted = Some(output.clone());
        send(WorkerEvent::ExtractedPath(output));
        if is_cancelled(&file.id, &cancel, &tx) {
            return;
        }
    }

    let subtitle = match files::load_subtitle(&subtitle_path) {
        Ok(s) => s,
        Err(err) => {
            fail(&tx, err);
            return;
        }
    };
    send(WorkerEvent::Original {
        file: subtitle.clone(),
    });

    let mut mux_language = settings.mux_language.clone();
    let mut mux_title = settings.mux_title.clone();

    if !settings.language_detection_model.trim().is_empty() {
        send(WorkerEvent::Status {
            status: FileStatus::DetectingLanguage,
            error: None,
        });
        log(
            LogLevel::Info,
            "logMessages.detectingTargetLanguage",
            vec![],
            &file.name,
        );
        match translate::detect_language(
            settings.to_language_detection_llm_config(),
            settings.prompt.clone(),
        )
        .await
        {
            Ok(detected) => {
                mux_language = detected.code.clone();
                mux_title = detected.display_name.clone();
                log(
                    LogLevel::Info,
                    "logMessages.targetLanguageDetected",
                    vec![("language", detected.display_name.clone())],
                    &file.name,
                );
                send(WorkerEvent::Language(detected));
            }
            Err(err) => {
                log(
                    LogLevel::Warning,
                    "logMessages.couldNotDetectLanguage",
                    vec![("error", err)],
                    &file.name,
                );
            }
        }
        if is_cancelled(&file.id, &cancel, &tx) {
            return;
        }
    }

    send(WorkerEvent::Status {
        status: FileStatus::Translating,
        error: None,
    });
    log(
        LogLevel::Info,
        "logMessages.translatingFile",
        vec![
            ("fileName", file.name.clone()),
            ("lines", subtitle.entries.len().to_string()),
        ],
        &file.name,
    );

    let tx_progress = tx.clone();
    let tx_entry = tx.clone();
    let tx_error = tx.clone();
    let file_name = file.name.clone();
    let callbacks = TranslationCallbacks {
        on_progress: Box::new(move |percent, done, total| {
            let _ = tx_progress.unbounded_send(WorkerEvent::Progress {
                percent: percent as f32,
                done,
                total,
            });
        }),
        on_entry: Box::new(move |index, text| {
            let _ = tx_entry.unbounded_send(WorkerEvent::Entry { index, text });
        }),
        on_error: Box::new(move |message, attempt| {
            let _ = tx_error.unbounded_send(WorkerEvent::Log {
                level: LogLevel::Warning,
                key: "logMessages.errorInFile",
                vars: vec![
                    ("fileName".into(), file_name.clone()),
                    ("attempt".into(), attempt.to_string()),
                    ("error".into(), message),
                ],
                file_name: file_name.clone(),
            });
        }),
    };

    let cleaner = if settings.text_cleaner_enabled {
        Some(settings.text_cleaner_config())
    } else {
        None
    };

    let result = translate::translate_subtitle_full(
        settings.to_llm_config(),
        settings.prompt.clone(),
        subtitle,
        settings.to_translation_settings(),
        file.id.clone(),
        cleaner,
        &cancel,
        callbacks,
    )
    .await;

    let translated = match result {
        Ok(result) => {
            if result.progress.is_partial && settings.auto_continue {
                log(
                    LogLevel::Warning,
                    "logMessages.partialTranslation",
                    vec![],
                    &file.name,
                );
            }
            send(WorkerEvent::Translated {
                file: result.file.clone(),
            });
            result.file
        }
        Err(err) if err.contains(TRANSLATION_CANCELLED_ERROR) => {
            send(WorkerEvent::Status {
                status: FileStatus::Cancelled,
                error: None,
            });
            return;
        }
        Err(err) => {
            log(
                LogLevel::Error,
                "logMessages.errorProcessing",
                vec![("fileName", file.name.clone()), ("error", err.clone())],
                &file.name,
            );
            fail(&tx, err);
            return;
        }
    };

    if is_cancelled(&file.id, &cancel, &tx) {
        return;
    }

    send(WorkerEvent::Status {
        status: FileStatus::Saving,
        error: None,
    });

    let output_subtitle = output_subtitle_path(&file, &subtitle_path, &settings);
    if let Err(err) = files::save_subtitle(&output_subtitle, &translated) {
        fail(&tx, err);
        return;
    }
    send(WorkerEvent::OutputSubtitle(output_subtitle.clone()));
    log(
        LogLevel::Info,
        "logMessages.subtitleSaved",
        vec![("path", output_subtitle.clone())],
        &file.name,
    );

    if settings.output_mode == "mux" && file.kind == FileKind::Video {
        send(WorkerEvent::Status {
            status: FileStatus::Muxing,
            error: None,
        });
        log(
            LogLevel::Info,
            "logMessages.doingMux",
            vec![("fileName", file.name.clone())],
            &file.name,
        );
        let output_video = replace_extension(&file.path, "muxed.mkv");
        if let Err(err) = ffmpeg::mux_subtitle_track(
            &file.path,
            &output_subtitle,
            &output_video,
            Some(&mux_language),
            Some(&mux_title),
        ) {
            fail(&tx, err);
            return;
        }
        send(WorkerEvent::OutputVideo(output_video.clone()));
        log(
            LogLevel::Success,
            "logMessages.muxedVideoSaved",
            vec![("path", output_video)],
            &file.name,
        );
    }

    let mut cleanup = Vec::new();
    if settings.cleanup_extracted_subtitles {
        if let Some(path) = extracted.clone() {
            cleanup.push(path);
        }
    }
    if settings.cleanup_mux_artifacts
        && settings.output_mode == "mux"
        && file.kind == FileKind::Video
    {
        cleanup.push(output_subtitle);
        if let Some(path) = extracted {
            if !cleanup.contains(&path) {
                cleanup.push(path);
            }
        }
    }
    if !cleanup.is_empty() {
        match files::delete_files(&cleanup) {
            Ok(_) => log(
                LogLevel::Info,
                "logMessages.tempFilesRemoved",
                vec![("count", cleanup.len().to_string())],
                &file.name,
            ),
            Err(err) => log(
                LogLevel::Warning,
                "logMessages.failedToRemoveTempFiles",
                vec![("error", err)],
                &file.name,
            ),
        }
    }

    if is_cancelled(&file.id, &cancel, &tx) {
        return;
    }

    send(WorkerEvent::Status {
        status: FileStatus::Completed,
        error: None,
    });
    send(WorkerEvent::Progress {
        percent: 100.0,
        done: translated.entries.len(),
        total: translated.entries.len(),
    });
    log(
        LogLevel::Success,
        "logMessages.fileProcessed",
        vec![("fileName", file.name.clone())],
        &file.name,
    );
    send(WorkerEvent::Done);
}

fn fail(tx: &mpsc::UnboundedSender<WorkerEvent>, err: String) {
    let _ = tx.unbounded_send(WorkerEvent::Status {
        status: FileStatus::Error,
        error: Some(err),
    });
    let _ = tx.unbounded_send(WorkerEvent::Done);
}

fn is_cancelled(
    id: &str,
    cancel: &TranslationCancelState,
    tx: &mpsc::UnboundedSender<WorkerEvent>,
) -> bool {
    if cancel.is_cancelled(id) {
        let _ = tx.unbounded_send(WorkerEvent::Status {
            status: FileStatus::Cancelled,
            error: None,
        });
        true
    } else {
        false
    }
}

fn replace_extension(path: &str, suffix_and_ext: &str) -> String {
    let path = Path::new(path);
    match path.file_stem().and_then(|s| s.to_str()) {
        Some(stem) => path
            .with_file_name(format!("{stem}.{suffix_and_ext}"))
            .to_string_lossy()
            .into_owned(),
        None => format!("{}.{suffix_and_ext}", path.display()),
    }
}

fn output_subtitle_path(
    file: &QueueFile,
    subtitle_path: &str,
    settings: &crate::core::settings::AppSettings,
) -> String {
    if !settings.separate_output_dir.trim().is_empty() {
        let name = Path::new(&file.name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("subtitle");
        Path::new(&settings.separate_output_dir)
            .join(format!("{name}.translated.ass"))
            .to_string_lossy()
            .into_owned()
    } else {
        let base = if file.kind == FileKind::Video {
            &file.path
        } else {
            subtitle_path
        };
        replace_extension(base, "translated.ass")
    }
}
