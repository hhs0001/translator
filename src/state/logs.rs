use gpui::{Context, EventEmitter};

use crate::i18n::{self, Language};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp_label: String,
    pub level: LogLevel,
    pub message: String,
    pub file: Option<String>,
}

pub struct LogsState {
    pub entries: Vec<LogEntry>,
    pub drawer_open: bool,
    pub filter: Option<LogLevel>,
    next_id: u64,
}

pub struct LogsChanged;

impl EventEmitter<LogsChanged> for LogsState {}

impl LogsState {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            entries: Vec::new(),
            drawer_open: false,
            filter: None,
            next_id: 1,
        }
    }

    pub fn add(
        &mut self,
        level: LogLevel,
        message: impl Into<String>,
        file: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let now = chrono_like_time();
        self.entries.push(LogEntry {
            id: self.next_id,
            timestamp_label: now,
            level,
            message: message.into(),
            file,
        });
        self.next_id += 1;
        if self.entries.len() > 500 {
            let extra = self.entries.len() - 500;
            self.entries.drain(0..extra);
        }
        cx.emit(LogsChanged);
        cx.notify();
    }

    pub fn info(
        &mut self,
        message: impl Into<String>,
        file: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.add(LogLevel::Info, message, file, cx);
    }

    pub fn warning(
        &mut self,
        message: impl Into<String>,
        file: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.add(LogLevel::Warning, message, file, cx);
    }

    pub fn error(
        &mut self,
        message: impl Into<String>,
        file: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.add(LogLevel::Error, message, file, cx);
    }

    pub fn success(
        &mut self,
        message: impl Into<String>,
        file: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.add(LogLevel::Success, message, file, cx);
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.entries.clear();
        cx.emit(LogsChanged);
        cx.notify();
    }

    pub fn toggle_drawer(&mut self, cx: &mut Context<Self>) {
        self.drawer_open = !self.drawer_open;
        cx.notify();
    }

    pub fn set_drawer_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.drawer_open = open;
        cx.notify();
    }

    pub fn set_filter(&mut self, filter: Option<LogLevel>, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }

    pub fn count_of(&self, level: LogLevel) -> usize {
        self.entries.iter().filter(|e| e.level == level).count()
    }

    /// Every entry as plain text, for the clipboard.
    pub fn as_text(&self) -> String {
        self.entries
            .iter()
            .map(|entry| {
                let level = match entry.level {
                    LogLevel::Info => "INFO",
                    LogLevel::Warning => "WARN",
                    LogLevel::Error => "ERROR",
                    LogLevel::Success => "OK",
                };
                match &entry.file {
                    Some(file) => {
                        format!(
                            "[{}] {level} ({file}) {}",
                            entry.timestamp_label, entry.message
                        )
                    }
                    None => format!("[{}] {level} {}", entry.timestamp_label, entry.message),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.level == LogLevel::Error)
            .count()
    }

    pub fn visible(&self) -> impl Iterator<Item = &LogEntry> {
        let filter = self.filter;
        self.entries.iter().filter(move |e| match filter {
            Some(level) => e.level == level,
            None => true,
        })
    }

    pub fn level_label(lang: Language, level: LogLevel) -> String {
        match level {
            LogLevel::Info => i18n::t(lang, "logs.filter.info"),
            LogLevel::Warning => i18n::t(lang, "logs.filter.warning"),
            LogLevel::Error => i18n::t(lang, "logs.filter.error"),
            LogLevel::Success => i18n::t(lang, "logs.filter.success"),
        }
    }
}

/// Local wall-clock time, `HH:MM:SS`.
fn chrono_like_time() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}
