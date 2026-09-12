use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{
    Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction,
};
use gpui_component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme, IndexPath, Sizable, WindowExt};

use crate::core::settings::AppSettings;
use crate::core::translator::{ApiFormat, ReasoningEffort};
use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::settings::SettingsState;
use crate::views::ui;

/// A single option in a dropdown: a stable value plus what the user sees.
#[derive(Clone)]
pub struct Choice {
    value: SharedString,
    title: SharedString,
    subtitle: Option<SharedString>,
}

impl Choice {
    fn new(value: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            title: title.into(),
            subtitle: None,
        }
    }

    fn with_subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

impl SelectItem for Choice {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.title.to_lowercase().contains(&query)
            || self.value.to_lowercase().contains(&query)
            || self
                .subtitle
                .as_ref()
                .is_some_and(|s| s.to_lowercase().contains(&query))
    }

    fn render(&self, _: &mut Window, _: &mut App) -> impl IntoElement {
        v_flex()
            .gap(px(1.))
            .child(div().child(self.title.clone()))
            .when_some(self.subtitle.clone(), |this, subtitle| {
                this.child(div().text_xs().opacity(0.6).child(subtitle))
            })
    }
}

type Dropdown = Entity<SelectState<SearchableVec<Choice>>>;

pub struct SettingsView {
    settings: Entity<SettingsState>,

    base_url: Entity<InputState>,
    api_key: Entity<InputState>,
    custom_model: Entity<InputState>,
    prompt: Entity<InputState>,
    mux_language: Entity<InputState>,
    mux_title: Entity<InputState>,
    output_dir: Entity<InputState>,
    batch_size: Entity<InputState>,
    parallel: Entity<InputState>,
    concurrency: Entity<InputState>,
    retries: Entity<InputState>,
    thinking_budget: Entity<InputState>,
    style_input: Entity<InputState>,
    tag_input: Entity<InputState>,
    template_name: Entity<InputState>,
    template_content: Entity<InputState>,

    api_format: Dropdown,
    model: Dropdown,
    detection_model: Dropdown,
    output_mode: Dropdown,
    reasoning: Dropdown,
    ui_language: Dropdown,

    header_inputs: HashMap<String, (Entity<InputState>, Entity<InputState>)>,
    header_subs: Vec<Subscription>,
    models_signature: String,
    /// Models are fetched lazily, the first time this page is rendered.
    models_requested: bool,
    editing_template: Option<String>,
    _subs: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(
        settings: Entity<SettingsState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let snap = settings.read(cx).settings.clone();
        let lang = settings.read(cx).language();

        let base_url = text_field(window, cx, &snap.base_url, "http://localhost:8045/v1");
        let api_key = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(snap.api_key.clone())
                .masked(true)
                .placeholder("sk-…")
        });
        let custom_model = text_field(window, cx, &snap.custom_model, "gpt-4o-mini");
        let prompt = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder(i18n::t(lang, "settings.prompt.placeholder"))
                .default_value(snap.prompt.clone())
        });
        let mux_language = text_field(window, cx, &snap.mux_language, "por");
        let mux_title = text_field(window, cx, &snap.mux_title, "Portuguese");
        let output_dir = text_field(
            window,
            cx,
            &snap.separate_output_dir,
            &i18n::t(lang, "settings.output.sameAsOriginal"),
        );
        let batch_size = text_field(window, cx, &snap.batch_size.to_string(), "50");
        let parallel = text_field(window, cx, &snap.parallel_requests.to_string(), "1");
        let concurrency = text_field(window, cx, &snap.concurrency.to_string(), "1");
        let retries = text_field(window, cx, &snap.max_retries.to_string(), "3");
        let thinking_budget = text_field(
            window,
            cx,
            &snap.anthropic_thinking_budget_tokens.to_string(),
            "1024",
        );
        let style_input = text_field(window, cx, "", "draw, sign, op, ed");
        let tag_input = text_field(window, cx, "", "blur, fscx");
        let template_name = text_field(
            window,
            cx,
            "",
            &i18n::t(lang, "settings.templates.nameExample"),
        );
        let template_content = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .placeholder(i18n::t(lang, "settings.templates.promptPlaceholder"))
        });

        let api_format = dropdown(
            window,
            cx,
            api_format_choices(lang),
            api_format_value(&snap.api_format),
        );
        let output_mode = dropdown(
            window,
            cx,
            output_mode_choices(lang),
            snap.output_mode.clone(),
        );
        let reasoning = dropdown(
            window,
            cx,
            reasoning_choices(lang),
            reasoning_value(&snap.reasoning_effort).to_string(),
        );
        let ui_language = dropdown(
            window,
            cx,
            vec![
                Choice::new("en", "English"),
                Choice::new("pt-BR", "Português (Brasil)"),
            ],
            snap.language.clone(),
        );
        let model = dropdown(
            window,
            cx,
            model_choices(&[], lang, false, &snap.model),
            snap.model.clone(),
        );
        let detection_model = dropdown(
            window,
            cx,
            model_choices(&[], lang, true, &snap.language_detection_model),
            if snap.language_detection_model.is_empty() {
                "none".to_string()
            } else {
                snap.language_detection_model.clone()
            },
        );

        let mut this = Self {
            settings: settings.clone(),
            base_url,
            api_key,
            custom_model,
            prompt,
            mux_language,
            mux_title,
            output_dir,
            batch_size,
            parallel,
            concurrency,
            retries,
            thinking_budget,
            style_input,
            tag_input,
            template_name,
            template_content,
            api_format,
            model,
            detection_model,
            output_mode,
            reasoning,
            ui_language,
            header_inputs: HashMap::new(),
            header_subs: Vec::new(),
            models_signature: String::new(),
            models_requested: false,
            editing_template: None,
            _subs: Vec::new(),
        };

        this.bind_inputs(window, cx);
        this.sync_headers(window, cx);

        this._subs
            .push(cx.observe_in(&settings, window, |this, _, window, cx| {
                this.sync_models(window, cx);
                this.sync_headers(window, cx);
                cx.notify();
            }));

        this
    }

    fn bind_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.bind_text(self.base_url.clone(), |s, v| s.base_url = v, cx);
        self.bind_text(self.api_key.clone(), |s, v| s.api_key = v, cx);
        self.bind_text(self.custom_model.clone(), |s, v| s.custom_model = v, cx);
        self.bind_text(self.prompt.clone(), |s, v| s.prompt = v, cx);
        self.bind_text(self.mux_language.clone(), |s, v| s.mux_language = v, cx);
        self.bind_text(self.mux_title.clone(), |s, v| s.mux_title = v, cx);
        self.bind_text(
            self.output_dir.clone(),
            |s, v| s.separate_output_dir = v,
            cx,
        );

        self.bind_number(
            self.batch_size.clone(),
            1,
            500,
            10,
            |s, v| s.batch_size = v as usize,
            window,
            cx,
        );
        self.bind_number(
            self.parallel.clone(),
            1,
            32,
            1,
            |s, v| s.parallel_requests = v as usize,
            window,
            cx,
        );
        self.bind_number(
            self.concurrency.clone(),
            1,
            16,
            1,
            |s, v| s.concurrency = v as usize,
            window,
            cx,
        );
        self.bind_number(
            self.retries.clone(),
            0,
            10,
            1,
            |s, v| s.max_retries = v as usize,
            window,
            cx,
        );
        self.bind_number(
            self.thinking_budget.clone(),
            1024,
            200_000,
            1024,
            |s, v| s.anthropic_thinking_budget_tokens = v as u32,
            window,
            cx,
        );

        self.bind_select(
            self.api_format.clone(),
            |s, v| {
                s.api_format = match v.as_ref() {
                    "openai" => ApiFormat::OpenAI,
                    "anthropic" => ApiFormat::Anthropic,
                    _ => ApiFormat::Auto,
                };
            },
            cx,
        );
        self.bind_select(
            self.output_mode.clone(),
            |s, v| s.output_mode = v.to_string(),
            cx,
        );
        self.bind_select(
            self.reasoning.clone(),
            |s, v| {
                s.reasoning_effort = match v.as_ref() {
                    "none" => ReasoningEffort::None,
                    "minimal" => ReasoningEffort::Minimal,
                    "low" => ReasoningEffort::Low,
                    "medium" => ReasoningEffort::Medium,
                    "high" => ReasoningEffort::High,
                    "xhigh" => ReasoningEffort::Xhigh,
                    _ => ReasoningEffort::Default,
                };
            },
            cx,
        );
        self.bind_select(self.model.clone(), |s, v| s.model = v.to_string(), cx);
        self.bind_select(
            self.detection_model.clone(),
            |s, v| {
                s.language_detection_model = if v.as_ref() == "none" {
                    String::new()
                } else {
                    v.to_string()
                };
            },
            cx,
        );

        let settings = self.settings.clone();
        self._subs.push(cx.subscribe(
            &self.ui_language,
            move |_, _, event: &SelectEvent<SearchableVec<Choice>>, cx| {
                let SelectEvent::Confirm(Some(value)) = event else {
                    return;
                };
                let language = Language::from_code(value);
                settings.update(cx, |state, cx| state.set_language(language, cx));
            },
        ));
    }

    fn bind_text(
        &mut self,
        input: Entity<InputState>,
        write: fn(&mut AppSettings, String),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs.push(
            cx.subscribe(&input, move |_, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Blur) {
                    let value = input.read(cx).value().to_string();
                    settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                }
            }),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn bind_number(
        &mut self,
        input: Entity<InputState>,
        min: i64,
        max: i64,
        step: i64,
        write: fn(&mut AppSettings, i64),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs.push(
            cx.subscribe(&input, move |_, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Blur | InputEvent::PressEnter { .. }) {
                    let raw = input.read(cx).value().to_string();
                    let value = raw.trim().parse::<i64>().unwrap_or(min).clamp(min, max);
                    settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                }
            }),
        );

        let settings = self.settings.clone();
        self._subs.push(cx.subscribe_in(
            &input,
            window,
            move |_, input, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                let current = input.read(cx).value().trim().parse::<i64>().unwrap_or(min);
                let next = match action {
                    StepAction::Increment => current + step,
                    StepAction::Decrement => current - step,
                }
                .clamp(min, max);
                input.update(cx, |state, cx| {
                    state.set_value(next.to_string(), window, cx);
                });
                settings.update(cx, |state, cx| state.update(|s| write(s, next), cx));
            },
        ));
    }

    fn bind_select(
        &mut self,
        select: Dropdown,
        write: fn(&mut AppSettings, SharedString),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs.push(cx.subscribe(
            &select,
            move |_, _, event: &SelectEvent<SearchableVec<Choice>>, cx| {
                let SelectEvent::Confirm(Some(value)) = event else {
                    return;
                };
                let value = value.clone();
                settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
            },
        ));
    }

    /// Rebuilds the model dropdowns when the fetched model list changes.
    fn sync_models(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.settings.read(cx);
        let lang = state.language();
        let signature = state
            .models
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>()
            .join("|");
        if signature == self.models_signature {
            return;
        }
        self.models_signature = signature;

        let models = state.models.clone();
        let selected_model = state.settings.model.clone();
        let selected_detection = state.settings.language_detection_model.clone();

        let items = model_choices(&models, lang, false, &selected_model);
        self.model.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            select.set_selected_value(&SharedString::from(selected_model), window, cx);
        });

        let items = model_choices(&models, lang, true, &selected_detection);
        let detection_value = if selected_detection.is_empty() {
            "none".to_string()
        } else {
            selected_detection
        };
        self.detection_model.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            select.set_selected_value(&SharedString::from(detection_value), window, cx);
        });
    }

    /// Creates/destroys the two inputs backing each custom header row.
    fn sync_headers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let headers = self.settings.read(cx).settings.headers.clone();
        let ids: Vec<String> = headers.iter().map(|h| h.id.clone()).collect();
        self.header_inputs.retain(|id, _| ids.contains(id));

        let mut created = false;
        for header in headers {
            if self.header_inputs.contains_key(&header.id) {
                continue;
            }
            created = true;
            let key = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(header.key.clone())
                    .placeholder("Header-Name")
            });
            let value = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(header.value.clone())
                    .placeholder("Value")
            });

            let settings = self.settings.clone();
            let id = header.id.clone();
            self.header_subs
                .push(cx.subscribe(&key, move |_, input, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Blur) {
                        let value = input.read(cx).value().to_string();
                        settings
                            .update(cx, |state, cx| state.set_header_field(&id, true, value, cx));
                    }
                }));

            let settings = self.settings.clone();
            let id = header.id.clone();
            self.header_subs.push(
                cx.subscribe(&value, move |_, input, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Blur) {
                        let value = input.read(cx).value().to_string();
                        settings.update(cx, |state, cx| {
                            state.set_header_field(&id, false, value, cx)
                        });
                    }
                }),
            );

            self.header_inputs.insert(header.id, (key, value));
        }

        if created {
            cx.notify();
        }
    }

    fn open_template_dialog(
        &mut self,
        template: Option<crate::core::templates::PromptTemplate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let lang = self.settings.read(cx).language();
        self.editing_template = template.as_ref().map(|t| t.id.clone());

        let (name, content) = match &template {
            Some(template) => (template.name.clone(), template.content.clone()),
            None => (
                String::new(),
                self.settings.read(cx).settings.prompt.clone(),
            ),
        };
        self.template_name.update(cx, |input, cx| {
            input.set_value(name, window, cx);
        });
        self.template_content.update(cx, |input, cx| {
            input.set_value(content, window, cx);
        });

        let title = if template.is_some() {
            i18n::t(lang, "settings.templates.editTemplate")
        } else {
            i18n::t(lang, "settings.templates.newTemplateTitle")
        };
        let name_input = self.template_name.clone();
        let content_input = self.template_content.clone();
        let name_label = i18n::t(lang, "common.name");
        let content_label = i18n::t(lang, "settings.templates.promptContent");
        let settings = self.settings.clone();
        let editing = self.editing_template.clone();

        window.open_dialog(cx, move |dialog, _window, cx: &mut App| {
            let name_input = name_input.clone();
            let content_input = content_input.clone();
            let settings = settings.clone();
            let editing = editing.clone();
            dialog
                .title(title.clone())
                .w(px(520.))
                .child(
                    v_flex()
                        .gap_3()
                        .py_2()
                        .child(ui::field(
                            name_label.clone(),
                            None,
                            Input::new(&name_input),
                            cx,
                        ))
                        .child(ui::field(
                            content_label.clone(),
                            None,
                            Input::new(&content_input).h(px(180.)),
                            cx,
                        )),
                )
                .on_ok(move |_, _, cx| {
                    let name = name_input.read(cx).value().trim().to_string();
                    let content = content_input.read(cx).value().to_string();
                    if name.is_empty() || content.trim().is_empty() {
                        return false;
                    }
                    settings.update(cx, |state, cx| match editing.clone() {
                        Some(id) => {
                            let _ = state.update_template(&id, Some(name), Some(content), cx);
                        }
                        None => {
                            let _ = state.add_template(name, content, cx);
                        }
                    });
                    true
                })
        });
    }
}

fn text_field(
    window: &mut Window,
    cx: &mut Context<SettingsView>,
    value: &str,
    placeholder: &str,
) -> Entity<InputState> {
    let value = value.to_string();
    let placeholder = placeholder.to_string();
    cx.new(|cx| {
        InputState::new(window, cx)
            .default_value(value)
            .placeholder(placeholder)
    })
}

fn dropdown(
    window: &mut Window,
    cx: &mut Context<SettingsView>,
    items: Vec<Choice>,
    selected: String,
) -> Dropdown {
    let index = items
        .iter()
        .position(|item| item.value.as_ref() == selected)
        .map(IndexPath::new);
    cx.new(|cx| SelectState::new(SearchableVec::new(items), index, window, cx).searchable(true))
}

fn api_format_value(format: &ApiFormat) -> String {
    match format {
        ApiFormat::OpenAI => "openai",
        ApiFormat::Anthropic => "anthropic",
        ApiFormat::Auto => "auto",
    }
    .to_string()
}

fn api_format_choices(lang: Language) -> Vec<Choice> {
    vec![
        Choice::new("auto", i18n::t(lang, "settings.api.autoDetect")),
        Choice::new("openai", "OpenAI Compatible"),
        Choice::new("anthropic", "Anthropic"),
    ]
}

fn output_mode_choices(lang: Language) -> Vec<Choice> {
    vec![
        Choice::new("separate", i18n::t(lang, "settings.output.separateFile")),
        Choice::new("mux", i18n::t(lang, "settings.output.muxVideo")),
    ]
}

fn reasoning_value(effort: &ReasoningEffort) -> &'static str {
    match effort {
        ReasoningEffort::Default => "default",
        ReasoningEffort::None => "none",
        ReasoningEffort::Minimal => "minimal",
        ReasoningEffort::Low => "low",
        ReasoningEffort::Medium => "medium",
        ReasoningEffort::High => "high",
        ReasoningEffort::Xhigh => "xhigh",
    }
}

fn reasoning_choices(lang: Language) -> Vec<Choice> {
    [
        ("default", "settings.translationSettings.thinkingDefault"),
        ("none", "settings.translationSettings.thinkingNone"),
        ("minimal", "settings.translationSettings.thinkingMinimal"),
        ("low", "settings.translationSettings.thinkingLow"),
        ("medium", "settings.translationSettings.thinkingMedium"),
        ("high", "settings.translationSettings.thinkingHigh"),
        ("xhigh", "settings.translationSettings.thinkingXHigh"),
    ]
    .into_iter()
    .map(|(value, key)| Choice::new(value, i18n::t(lang, key)))
    .collect()
}

fn model_choices(
    models: &[crate::core::translator::LlmModel],
    lang: Language,
    with_none: bool,
    selected: &str,
) -> Vec<Choice> {
    let mut choices = Vec::new();
    if with_none {
        choices.push(Choice::new("none", i18n::t(lang, "settings.api.none")));
    }
    // The saved model must stay visible even before the list is fetched.
    if !selected.is_empty() && !models.iter().any(|model| model.id == selected) {
        choices.push(
            Choice::new(selected.to_string(), selected.to_string())
                .with_subtitle(i18n::t(lang, "settings.api.savedModel")),
        );
    }
    choices.extend(models.iter().map(|model| {
        let title = model.name.clone().unwrap_or_else(|| model.id.clone());
        let mut choice = Choice::new(model.id.clone(), title);
        let context = model
            .context_length
            .map(|ctx| format!("{}k ctx", ctx / 1000))
            .unwrap_or_default();
        let subtitle = if context.is_empty() {
            model.id.clone()
        } else {
            format!("{} · {context}", model.id)
        };
        choice = choice.with_subtitle(subtitle);
        choice
    }));
    choices
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.models_requested {
            self.models_requested = true;
            let has_endpoint = !self.settings.read(cx).settings.base_url.trim().is_empty();
            if has_endpoint {
                self.settings
                    .update(cx, |state, cx| state.refresh_models(cx));
            }
        }

        let state = self.settings.read(cx);
        let lang = state.language();
        let snap = state.settings.clone();
        let ffmpeg = state.ffmpeg.clone();
        let models_loading = state.models_loading;
        let models_error = state.models_error.clone();
        let models_count = state.models.len();
        let templates = state.templates.clone();
        let save_error = state.save_error.clone();
        let dark = state.is_dark();

        div()
            .id("settings-scroll")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                v_flex()
                    .w_full()
                    .gap_5()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .items_center()
                            .child(ui::icon_badge(Ico::Settings, cx.theme().primary, cx))
                            .child(
                                v_flex()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(i18n::t(lang, "settings.title")),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(i18n::t(lang, "settings.subtitle")),
                                    ),
                            ),
                    )
                    .when_some(save_error, |this, error| {
                        this.child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .p_3()
                                .rounded(px(10.))
                                .bg(ui::tint(cx.theme().danger, 0.12))
                                .child(
                                    Ico::TriangleAlert
                                        .icon()
                                        .small()
                                        .text_color(cx.theme().danger),
                                )
                                .child(div().text_xs().text_color(cx.theme().danger).child(error)),
                        )
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .gap_5()
                            .items_start()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_5()
                                    .child(self.api_card(
                                        &snap,
                                        lang,
                                        models_loading,
                                        models_count,
                                        models_error,
                                        cx,
                                    ))
                                    .child(self.prompt_card(&snap, &templates, lang, cx))
                                    .child(self.output_card(&snap, lang, cx)),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_5()
                                    .child(self.translation_card(&snap, lang, cx))
                                    .child(self.cleaner_card(&snap, lang, cx))
                                    .child(self.ffmpeg_card(ffmpeg, lang, cx))
                                    .child(self.appearance_card(dark, lang, cx))
                                    .child(self.app_data_card(lang, cx)),
                            ),
                    ),
            )
    }
}

impl SettingsView {
    fn api_card(
        &self,
        snap: &AppSettings,
        lang: Language,
        loading: bool,
        models_count: usize,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let detected = if matches!(snap.api_format, ApiFormat::Anthropic)
            || (matches!(snap.api_format, ApiFormat::Auto)
                && snap.base_url.to_lowercase().contains("anthropic"))
        {
            "Anthropic"
        } else {
            "OpenAI"
        };

        ui::card(cx)
            .child(ui::card_header(
                Ico::Key,
                cx.theme().primary,
                i18n::t(lang, "settings.api.title"),
                Some(i18n::t(lang, "settings.api.baseUrlHint")),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.api.baseUrl"),
                None,
                Input::new(&self.base_url).small(),
                cx,
            ))
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_end()
                    .pb(px(4.))
                    .child(div().flex_1().min_w_0().child(ui::field(
                        i18n::t(lang, "settings.api.apiFormat"),
                        None,
                        Select::new(&self.api_format).small(),
                        cx,
                    )))
                    .child(ui::chip(
                        format!("{}: {detected}", i18n::t(lang, "settings.api.format")),
                        cx.theme().info,
                        Some(Ico::Zap),
                        cx,
                    )),
            )
            .child(ui::field(
                i18n::t(lang, "settings.api.apiKey"),
                None,
                Input::new(&self.api_key).small().mask_toggle(),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.api.model"),
                Some(i18n::tf(
                    lang,
                    "settings.api.modelsFound",
                    &[("count", &models_count.to_string())],
                )),
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_center()
                    .child(
                        div().flex_1().min_w_0().child(
                            Select::new(&self.model)
                                .small()
                                .placeholder(i18n::t(lang, "settings.api.selectModel"))
                                .search_placeholder(i18n::t(lang, "settings.api.searchModels"))
                                .empty(
                                    div()
                                        .p_2()
                                        .text_xs()
                                        .child(i18n::t(lang, "settings.api.noModelsFound")),
                                ),
                        ),
                    )
                    .child(
                        Button::new("refresh-models")
                            .small()
                            .outline()
                            .icon(Ico::Refresh.icon())
                            .loading(loading)
                            .tooltip(i18n::t(lang, "settings.api.searchModels"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.settings
                                    .update(cx, |state, cx| state.refresh_models(cx));
                            })),
                    ),
                cx,
            ))
            .when_some(error, |this, error| {
                this.child(
                    div()
                        .text_xs()
                        .whitespace_normal()
                        .text_color(cx.theme().danger)
                        .child(error),
                )
            })
            .child(ui::field(
                i18n::t(lang, "settings.api.customModel"),
                Some(i18n::t(lang, "settings.api.customModelHint")),
                Input::new(&self.custom_model).small().cleanable(true),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.api.languageDetectionModel"),
                Some(i18n::t(lang, "settings.api.languageDetectionModelHint")),
                Select::new(&self.detection_model)
                    .small()
                    .placeholder(i18n::t(lang, "settings.api.none")),
                cx,
            ))
            .child(ui::rule(cx))
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        h_flex()
                            .w_full()
                            .justify_between()
                            .items_center()
                            .child(ui::overline(
                                i18n::t(lang, "settings.api.advancedHeaders"),
                                cx,
                            ))
                            .child(
                                Button::new("add-header")
                                    .xsmall()
                                    .ghost()
                                    .label(i18n::t(lang, "settings.api.addHeader"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.update(cx, |state, cx| state.add_header(cx));
                                    })),
                            ),
                    )
                    .children(snap.headers.iter().map(|header| {
                        let id = header.id.clone();
                        let inputs = self.header_inputs.get(&header.id);
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .when_some(inputs, |this, (key, value)| {
                                this.child(div().flex_1().min_w_0().child(Input::new(key).small()))
                                    .child(
                                        div().flex_1().min_w_0().child(Input::new(value).small()),
                                    )
                            })
                            .child(
                                Button::new(SharedString::from(format!("rm-header-{id}")))
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::Close.icon())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let id = id.clone();
                                        this.settings
                                            .update(cx, |state, cx| state.remove_header(&id, cx));
                                    })),
                            )
                    })),
            )
    }

    fn prompt_card(
        &self,
        snap: &AppSettings,
        templates: &[crate::core::templates::PromptTemplate],
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        ui::card(cx)
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(ui::card_header(
                        Ico::Sparkles,
                        cx.theme().chart_2,
                        i18n::t(lang, "settings.prompt.title"),
                        Some(i18n::t(lang, "settings.prompt.hint")),
                        cx,
                    ))
                    .child(
                        Button::new("new-template")
                            .xsmall()
                            .outline()
                            .label(i18n::t(lang, "settings.templates.newTemplate"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_template_dialog(None, window, cx);
                            })),
                    ),
            )
            .child(Input::new(&self.prompt).h(px(150.)))
            .child(ui::overline(i18n::t(lang, "settings.templates.title"), cx))
            .child(if templates.is_empty() {
                div()
                    .w_full()
                    .p_3()
                    .rounded(px(10.))
                    .border_1()
                    .border_dashed()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(i18n::t(lang, "settings.templates.noTemplates"))
                    .into_any_element()
            } else {
                v_flex()
                    .w_full()
                    .gap_1p5()
                    .children(templates.iter().map(|template| {
                        let selected = snap.selected_template_id.as_deref() == Some(&template.id);
                        let apply_id = template.id.clone();
                        let edit_template = template.clone();
                        let delete_id = template.id.clone();
                        h_flex()
                            .w_full()
                            .gap_2()
                            .p_2p5()
                            .rounded(px(10.))
                            .border_1()
                            .border_color(if selected {
                                cx.theme().primary.opacity(0.5)
                            } else {
                                cx.theme().border.opacity(0.6)
                            })
                            .bg(if selected {
                                ui::tint(cx.theme().primary, 0.08)
                            } else {
                                cx.theme().muted.opacity(0.3)
                            })
                            .items_center()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .whitespace_nowrap()
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(ui::preview_text(&template.name, 48)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .whitespace_nowrap()
                                            .text_ellipsis()
                                            .overflow_hidden()
                                            .child(ui::preview_text(&template.content, 90)),
                                    ),
                            )
                            .child(
                                Button::new(SharedString::from(format!("apply-{}", template.id)))
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::Check.icon())
                                    .tooltip(i18n::t(lang, "settings.prompt.applyTemplate"))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let id = apply_id.clone();
                                        this.settings
                                            .update(cx, |state, cx| state.apply_template(&id, cx));
                                        let prompt = this.settings.read(cx).settings.prompt.clone();
                                        this.prompt.update(cx, |input, cx| {
                                            input.set_value(prompt, window, cx);
                                        });
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("edit-{}", template.id)))
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::Pencil.icon())
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_template_dialog(
                                            Some(edit_template.clone()),
                                            window,
                                            cx,
                                        );
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("del-{}", template.id)))
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::Trash.icon())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let id = delete_id.clone();
                                        this.settings.update(cx, |state, cx| {
                                            let _ = state.delete_template(&id, cx);
                                        });
                                    })),
                            )
                    }))
                    .into_any_element()
            })
    }

    fn translation_card(
        &self,
        snap: &AppSettings,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let presets = [25usize, 50, 100, 150];

        ui::card(cx)
            .child(ui::card_header(
                Ico::Gauge,
                cx.theme().chart_1,
                i18n::t(lang, "settings.translationSettings.title"),
                Some(i18n::t(lang, "settings.translationSettings.batchSizeHint")),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.translationSettings.batchSize"),
                None,
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(NumberInput::new(&self.batch_size).small())
                    .child(h_flex().gap_1().children(presets.into_iter().map(|preset| {
                        let selected = snap.batch_size == preset;
                        let mut btn = Button::new(SharedString::from(format!("batch-{preset}")))
                            .xsmall()
                            .label(format!(
                                "{preset} {}",
                                i18n::t(lang, "settings.translationSettings.lines")
                            ))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.batch_size.update(cx, |input, cx| {
                                    input.set_value(preset.to_string(), window, cx);
                                });
                                this.settings.update(cx, |state, cx| {
                                    state.update(|s| s.batch_size = preset, cx)
                                });
                            }));
                        btn = if selected { btn.primary() } else { btn.ghost() };
                        btn
                    }))),
                cx,
            ))
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(div().flex_1().min_w_0().child(ui::field(
                        i18n::t(lang, "settings.translationSettings.parallelRequests"),
                        Some(i18n::t(
                            lang,
                            "settings.translationSettings.parallelRequestsHint",
                        )),
                        NumberInput::new(&self.parallel).small(),
                        cx,
                    )))
                    .child(div().flex_1().min_w_0().child(ui::field(
                        i18n::t(lang, "settings.translationSettings.concurrency"),
                        Some(i18n::t(
                            lang,
                            "settings.translationSettings.concurrencyHint",
                        )),
                        NumberInput::new(&self.concurrency).small(),
                        cx,
                    ))),
            )
            .child(ui::field(
                i18n::t(lang, "settings.translationSettings.maxRetries"),
                None,
                NumberInput::new(&self.retries).small(),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.translationSettings.thinkingMode"),
                Some(i18n::t(
                    lang,
                    "settings.translationSettings.thinkingModeHint",
                )),
                Select::new(&self.reasoning).small(),
                cx,
            ))
            .child(ui::rule(cx))
            .child(toggle_row(
                "streaming",
                Ico::Zap,
                i18n::t(lang, "settings.translationSettings.streaming"),
                Some(i18n::t(lang, "settings.translationSettings.streamingHint")),
                snap.streaming,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings
                        .update(cx, |state, cx| state.update(|s| s.streaming = checked, cx));
                }),
                cx,
            ))
            .child(toggle_row(
                "auto-continue",
                Ico::Refresh,
                i18n::t(lang, "settings.translationSettings.autoContinue"),
                None,
                snap.auto_continue,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings.update(cx, |state, cx| {
                        state.update(|s| s.auto_continue = checked, cx)
                    });
                }),
                cx,
            ))
            .child(toggle_row(
                "continue-error",
                Ico::TriangleAlert,
                i18n::t(lang, "settings.translationSettings.continueOnError"),
                None,
                snap.continue_on_error,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings.update(cx, |state, cx| {
                        state.update(|s| s.continue_on_error = checked, cx)
                    });
                }),
                cx,
            ))
            .child(toggle_row(
                "anthropic-thinking",
                Ico::Sparkles,
                i18n::t(lang, "settings.translationSettings.anthropicThinking"),
                Some(i18n::t(
                    lang,
                    "settings.translationSettings.anthropicThinkingHint",
                )),
                snap.anthropic_thinking_enabled,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings.update(cx, |state, cx| {
                        state.update(|s| s.anthropic_thinking_enabled = checked, cx)
                    });
                }),
                cx,
            ))
            .when(snap.anthropic_thinking_enabled, |this| {
                this.child(ui::field(
                    i18n::t(lang, "settings.translationSettings.anthropicThinkingBudget"),
                    Some(i18n::t(
                        lang,
                        "settings.translationSettings.anthropicThinkingBudgetHint",
                    )),
                    NumberInput::new(&self.thinking_budget).small(),
                    cx,
                ))
            })
    }

    fn output_card(
        &self,
        snap: &AppSettings,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_mux = snap.output_mode == "mux";

        ui::card(cx)
            .child(ui::card_header(
                Ico::FolderOpen,
                cx.theme().chart_3,
                i18n::t(lang, "settings.output.title"),
                Some(if is_mux {
                    i18n::t(lang, "settings.output.muxOutputHint")
                } else {
                    i18n::t(lang, "settings.output.separateOutputHint")
                }),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.output.outputMode"),
                None,
                Select::new(&self.output_mode).small(),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.output.outputFolder"),
                None,
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(&self.output_dir).small().cleanable(true)),
                    )
                    .child(
                        Button::new("browse-output")
                            .small()
                            .outline()
                            .icon(Ico::Folder.icon())
                            .label(i18n::t(lang, "common.select"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.pick_output_dir(window, cx);
                            })),
                    ),
                cx,
            ))
            .when(is_mux, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_start()
                        .child(div().flex_1().min_w_0().child(ui::field(
                            i18n::t(lang, "settings.output.languageCode"),
                            Some(i18n::t(lang, "settings.output.languageCodeHint")),
                            Input::new(&self.mux_language).small(),
                            cx,
                        )))
                        .child(div().flex_1().min_w_0().child(ui::field(
                            i18n::t(lang, "settings.output.trackTitle"),
                            None,
                            Input::new(&self.mux_title).small(),
                            cx,
                        ))),
                )
            })
            .child(toggle_row(
                "cleanup-extracted",
                Ico::Trash,
                i18n::t(lang, "settings.output.cleanupExtracted"),
                Some(i18n::t(lang, "settings.output.cleanupExtractedHint")),
                snap.cleanup_extracted_subtitles,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings.update(cx, |state, cx| {
                        state.update(|s| s.cleanup_extracted_subtitles = checked, cx)
                    });
                }),
                cx,
            ))
            .when(is_mux, |this| {
                this.child(toggle_row(
                    "cleanup-mux",
                    Ico::Trash,
                    i18n::t(lang, "settings.output.cleanupMux"),
                    Some(i18n::t(lang, "settings.output.cleanupMuxHint")),
                    snap.cleanup_mux_artifacts,
                    cx.listener(|this, checked: &bool, _, cx| {
                        let checked = *checked;
                        this.settings.update(cx, |state, cx| {
                            state.update(|s| s.cleanup_mux_artifacts = checked, cx)
                        });
                    }),
                    cx,
                ))
            })
    }

    fn cleaner_card(
        &self,
        snap: &AppSettings,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        ui::card(cx)
            .child(ui::card_header(
                Ico::Wand,
                cx.theme().chart_4,
                i18n::t(lang, "settings.textCleaner.title"),
                Some(i18n::t(lang, "settings.textCleaner.whatItDoesDescription")),
                cx,
            ))
            .child(toggle_row(
                "cleaner-enabled",
                Ico::Wand,
                i18n::t(lang, "settings.textCleaner.enabled"),
                Some(i18n::t(lang, "settings.textCleaner.enabledHint")),
                snap.text_cleaner_enabled,
                cx.listener(|this, checked: &bool, _, cx| {
                    let checked = *checked;
                    this.settings.update(cx, |state, cx| {
                        state.update(|s| s.text_cleaner_enabled = checked, cx)
                    });
                }),
                cx,
            ))
            .when(snap.text_cleaner_enabled, |this| {
                this.child(toggle_row(
                    "preserve-basic",
                    Ico::FileText,
                    i18n::t(lang, "settings.textCleaner.preserveBasic"),
                    Some(i18n::t(lang, "settings.textCleaner.preserveBasicHint")),
                    snap.text_cleaner_preserve_basic_formatting,
                    cx.listener(|this, checked: &bool, _, cx| {
                        let checked = *checked;
                        this.settings.update(cx, |state, cx| {
                            state.update(|s| s.text_cleaner_preserve_basic_formatting = checked, cx)
                        });
                    }),
                    cx,
                ))
                .child(self.chip_editor(
                    i18n::t(lang, "settings.textCleaner.ignoredStyles"),
                    i18n::t(lang, "settings.textCleaner.ignoredStylesHint"),
                    snap.text_cleaner_ignored_styles.clone(),
                    true,
                    cx,
                ))
                .child(self.chip_editor(
                    i18n::t(lang, "settings.textCleaner.customTags"),
                    i18n::t(lang, "settings.textCleaner.customTagsHint"),
                    snap.text_cleaner_tags_to_remove.clone(),
                    false,
                    cx,
                ))
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .items_start()
                        .p_3()
                        .rounded(px(10.))
                        .bg(ui::tint(cx.theme().warning, 0.1))
                        .child(Ico::Info.icon().xsmall().text_color(cx.theme().warning))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .whitespace_normal()
                                .text_color(cx.theme().warning)
                                .child(i18n::t(lang, "settings.textCleaner.tokenSavings")),
                        ),
                )
            })
    }

    /// Editor for the cleaner's style/tag lists, rendered as removable chips.
    fn chip_editor(
        &self,
        label: String,
        hint: String,
        values: Vec<String>,
        is_style: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let input = if is_style {
            self.style_input.clone()
        } else {
            self.tag_input.clone()
        };

        ui::field(
            label,
            Some(hint),
            v_flex()
                .w_full()
                .gap_2()
                .when(!values.is_empty(), |this| {
                    this.child(
                        h_flex()
                            .gap_1()
                            .flex_wrap()
                            .children(values.into_iter().map(|value| {
                                let removed = value.clone();
                                h_flex()
                                    .id(SharedString::from(format!(
                                        "chip-{}-{value}",
                                        if is_style { "style" } else { "tag" }
                                    )))
                                    .gap_1()
                                    .items_center()
                                    .px_2()
                                    .py(px(3.))
                                    .rounded_full()
                                    .bg(cx.theme().muted.opacity(0.8))
                                    .text_xs()
                                    .child(value.clone())
                                    .child(
                                        Ico::Close
                                            .icon()
                                            .xsmall()
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                    .cursor_pointer()
                                    .hover(|this| this.bg(ui::tint(cx.theme().danger, 0.18)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let removed = removed.clone();
                                        this.settings.update(cx, |state, cx| {
                                            state.update(
                                                |s| {
                                                    let list = if is_style {
                                                        &mut s.text_cleaner_ignored_styles
                                                    } else {
                                                        &mut s.text_cleaner_tags_to_remove
                                                    };
                                                    list.retain(|item| item != &removed);
                                                },
                                                cx,
                                            )
                                        });
                                    }))
                            })),
                    )
                })
                .child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .items_center()
                        .child(div().flex_1().min_w_0().child(Input::new(&input).small()))
                        .child(
                            Button::new(SharedString::from(format!(
                                "add-{}",
                                if is_style { "style" } else { "tag" }
                            )))
                            .small()
                            .outline()
                            .icon(Ico::Plus.icon())
                            .on_click(cx.listener(
                                move |this, _, window, cx| {
                                    this.add_cleaner_values(is_style, window, cx);
                                },
                            )),
                        ),
                ),
            cx,
        )
    }

    fn add_cleaner_values(&mut self, is_style: bool, window: &mut Window, cx: &mut Context<Self>) {
        let input = if is_style {
            self.style_input.clone()
        } else {
            self.tag_input.clone()
        };
        let raw = input.read(cx).value().to_string();
        let values: Vec<String> = raw
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
        if values.is_empty() {
            return;
        }

        self.settings.update(cx, |state, cx| {
            state.update(
                |s| {
                    let list = if is_style {
                        &mut s.text_cleaner_ignored_styles
                    } else {
                        &mut s.text_cleaner_tags_to_remove
                    };
                    for value in &values {
                        if !list.contains(value) {
                            list.push(value.clone());
                        }
                    }
                },
                cx,
            )
        });
        input.update(cx, |state, cx| state.set_value("", window, cx));
    }

    fn pick_output_dir(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = rx.await else {
                return;
            };
            let Some(path) = paths.first().map(|p| p.to_string_lossy().to_string()) else {
                return;
            };
            let _ = this.update_in(cx, |this, window, cx| {
                this.output_dir.update(cx, |input, cx| {
                    input.set_value(path.clone(), window, cx);
                });
                this.settings.update(cx, |state, cx| {
                    state.update(|s| s.separate_output_dir = path.clone(), cx)
                });
            });
        })
        .detach();
    }

    fn ffmpeg_card(
        &self,
        ffmpeg: Option<Result<String, String>>,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (color, icon, label) = match &ffmpeg {
            Some(Ok(version)) => (
                cx.theme().success,
                Ico::CircleCheck,
                format!("{}: {version}", i18n::t(lang, "settings.ffmpeg.installed")),
            ),
            Some(Err(_)) => (
                cx.theme().danger,
                Ico::CircleX,
                i18n::t(lang, "settings.ffmpeg.notFoundDescription"),
            ),
            None => (
                cx.theme().muted_foreground,
                Ico::Loader,
                i18n::t(lang, "settings.ffmpeg.checking"),
            ),
        };

        ui::card(cx)
            .child(ui::card_header(
                Ico::Terminal,
                cx.theme().chart_5,
                i18n::t(lang, "settings.ffmpeg.title"),
                Some(i18n::t(lang, "settings.ffmpeg.description")),
                cx,
            ))
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_center()
                    .p_3()
                    .rounded(px(10.))
                    .bg(ui::tint(color, 0.1))
                    .child(icon.icon().small().text_color(color))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .whitespace_normal()
                            .text_color(color)
                            .child(label),
                    ),
            )
            .child(
                Button::new("check-ffmpeg")
                    .small()
                    .outline()
                    .icon(Ico::Refresh.icon())
                    .label(i18n::t(lang, "settings.ffmpeg.check"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings
                            .update(cx, |state, cx| state.refresh_ffmpeg(cx));
                    })),
            )
    }

    fn appearance_card(
        &self,
        dark: bool,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        ui::card(cx)
            .child(ui::card_header(
                Ico::Globe,
                cx.theme().info,
                i18n::t(lang, "settings.language.title"),
                Some(i18n::t(lang, "settings.language.hint")),
                cx,
            ))
            .child(ui::field(
                i18n::t(lang, "settings.language.label"),
                None,
                Select::new(&self.ui_language).small(),
                cx,
            ))
            .child(toggle_row(
                "dark-mode",
                if dark { Ico::Moon } else { Ico::Sun },
                i18n::t(lang, "settings.appearance.darkMode"),
                None,
                dark,
                cx.listener(|this, checked: &bool, window, cx| {
                    let checked = *checked;
                    this.settings
                        .update(cx, |state, cx| state.set_dark(checked, window, cx));
                }),
                cx,
            ))
    }

    fn app_data_card(&self, lang: Language, cx: &mut Context<Self>) -> impl IntoElement {
        ui::card(cx)
            .child(ui::card_header(
                Ico::Folder,
                cx.theme().muted_foreground,
                i18n::t(lang, "settings.appData.title"),
                Some(i18n::t(lang, "settings.appData.configFilesHint")),
                cx,
            ))
            .child(
                Button::new("open-config")
                    .small()
                    .outline()
                    .icon(Ico::ExternalLink.icon())
                    .label(i18n::t(lang, "settings.appData.openConfigFolder"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.settings.read(cx).open_config_folder();
                    })),
            )
    }
}

/// Switch row with icon, label and optional description.
fn toggle_row(
    id: &'static str,
    icon: Ico,
    label: String,
    description: Option<String>,
    checked: bool,
    on_change: impl Fn(&bool, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .w_full()
        .gap_3()
        .items_center()
        .child(icon.icon().small().flex_shrink_0().text_color(if checked {
            cx.theme().primary
        } else {
            cx.theme().muted_foreground
        }))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(1.))
                .child(div().text_sm().child(label))
                .when_some(description, |this, description| {
                    this.child(ui::hint(description, cx))
                }),
        )
        .child(
            Switch::new(id)
                .checked(checked)
                .on_click(move |checked, window, cx| on_change(checked, window, cx)),
        )
}
