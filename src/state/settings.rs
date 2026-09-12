use futures::channel::oneshot;
use gpui::{AppContext, Context, Entity, EventEmitter};

use crate::core::ffmpeg;
use crate::core::settings::{self, AppSettings, HeaderItem};
use crate::core::templates::{self, PromptTemplate};
use crate::core::translator::LlmModel;
use crate::i18n::Language;

pub struct SettingsState {
    pub settings: AppSettings,
    pub templates: Vec<PromptTemplate>,
    pub ffmpeg: Option<Result<String, String>>,
    pub models: Vec<LlmModel>,
    pub models_loading: bool,
    pub models_error: Option<String>,
    pub save_error: Option<String>,
}

pub struct SettingsChanged;

impl EventEmitter<SettingsChanged> for SettingsState {}

impl SettingsState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let settings = settings::load().unwrap_or_default();
        let templates = templates::load_all().unwrap_or_default();
        let this = Self {
            settings,
            templates,
            ffmpeg: None,
            models: Vec::new(),
            models_loading: false,
            models_error: None,
            save_error: None,
        };
        this.refresh_ffmpeg(cx);
        this
    }

    pub fn language(&self) -> Language {
        Language::from_code(&self.settings.language)
    }

    pub fn persist(&mut self, cx: &mut Context<Self>) {
        match settings::save(&self.settings) {
            Ok(()) => self.save_error = None,
            Err(err) => self.save_error = Some(err),
        }
        cx.emit(SettingsChanged);
        cx.notify();
    }

    pub fn update<F>(&mut self, f: F, cx: &mut Context<Self>)
    where
        F: FnOnce(&mut AppSettings),
    {
        f(&mut self.settings);
        self.persist(cx);
    }

    pub fn is_dark(&self) -> bool {
        self.settings.theme != "light"
    }

    pub fn set_dark(&mut self, dark: bool, window: &mut gpui::Window, cx: &mut Context<Self>) {
        self.settings.theme = if dark { "dark" } else { "light" }.to_string();
        crate::theme::apply(dark, Some(window), cx);
        self.persist(cx);
    }

    pub fn set_header_field(&mut self, id: &str, key: bool, value: String, cx: &mut Context<Self>) {
        if let Some(header) = self.settings.headers.iter_mut().find(|h| h.id == id) {
            if key {
                header.key = value;
            } else {
                header.value = value;
            }
            self.persist(cx);
        }
    }

    pub fn set_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.settings.language = language.code().to_string();
        gpui_component::set_locale(language.code());
        self.persist(cx);
    }

    pub fn add_header(&mut self, cx: &mut Context<Self>) {
        self.settings.headers.push(HeaderItem {
            id: uuid::Uuid::new_v4().to_string(),
            key: String::new(),
            value: String::new(),
        });
        self.persist(cx);
    }

    pub fn remove_header(&mut self, id: &str, cx: &mut Context<Self>) {
        self.settings.headers.retain(|h| h.id != id);
        self.persist(cx);
    }

    pub fn refresh_ffmpeg(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(async { ffmpeg::check_ffmpeg() }).await;
            let _ = this.update(cx, |this, cx| {
                this.ffmpeg = Some(result);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ffmpeg_ok(&self) -> bool {
        matches!(self.ffmpeg, Some(Ok(_)))
    }

    pub fn refresh_models(&mut self, cx: &mut Context<Self>) {
        self.models_loading = true;
        self.models_error = None;
        cx.notify();

        let config = self.settings.to_llm_config();
        cx.spawn(async move |this, cx| {
            let (tx, rx) = oneshot::channel();
            crate::core::runtime::spawn(async move {
                let result = crate::core::translate::list_llm_models(config).await;
                let _ = tx.send(result);
            });
            let result = rx.await.unwrap_or_else(|_| Err("cancelled".to_string()));

            let _ = this.update(cx, |this, cx| {
                this.models_loading = false;
                match result {
                    Ok(models) => {
                        this.models = models;
                        this.models_error = None;
                    }
                    Err(err) => this.models_error = Some(err),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn add_template(
        &mut self,
        name: String,
        content: String,
        cx: &mut Context<Self>,
    ) -> Result<PromptTemplate, String> {
        let template = templates::add(name, content)?;
        self.templates = templates::load_all()?;
        cx.emit(SettingsChanged);
        cx.notify();
        Ok(template)
    }

    pub fn update_template(
        &mut self,
        id: &str,
        name: Option<String>,
        content: Option<String>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        templates::update(id, name, content)?;
        self.templates = templates::load_all()?;
        cx.emit(SettingsChanged);
        cx.notify();
        Ok(())
    }

    pub fn delete_template(&mut self, id: &str, cx: &mut Context<Self>) -> Result<(), String> {
        templates::delete(id)?;
        if self.settings.selected_template_id.as_deref() == Some(id) {
            self.settings.selected_template_id = None;
            let _ = settings::save(&self.settings);
        }
        self.templates = templates::load_all()?;
        cx.emit(SettingsChanged);
        cx.notify();
        Ok(())
    }

    pub fn apply_template(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(template) = self.templates.iter().find(|t| t.id == id).cloned() {
            self.settings.prompt = template.content;
            self.settings.selected_template_id = Some(template.id);
            self.persist(cx);
        }
    }

    pub fn open_config_folder(&self) {
        if let Ok(dir) = crate::core::paths::app_data_dir() {
            let _ = crate::core::files::open_folder(dir);
        }
    }
}

pub fn read_language(settings: &Entity<SettingsState>, cx: &gpui::App) -> Language {
    settings.read(cx).language()
}
