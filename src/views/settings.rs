use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::core::translator::{ApiFormat, ReasoningEffort};
use crate::i18n::{self, Language};
use crate::state::settings::SettingsState;

pub struct SettingsView {
    settings: Entity<SettingsState>,
    base_url: Entity<InputState>,
    api_key: Entity<InputState>,
    model: Entity<InputState>,
    custom_model: Entity<InputState>,
    detection_model: Entity<InputState>,
    prompt: Entity<InputState>,
    batch_size: Entity<InputState>,
    parallel: Entity<InputState>,
    concurrency: Entity<InputState>,
    retries: Entity<InputState>,
    mux_language: Entity<InputState>,
    mux_title: Entity<InputState>,
    output_dir: Entity<InputState>,
    ignored_styles: Entity<InputState>,
    custom_tags: Entity<InputState>,
    thinking_budget: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl SettingsView {
    pub fn new(
        settings: Entity<SettingsState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let snap = settings.read(cx).settings.clone();

        let base_url = field(window, cx, &snap.base_url, false, false);
        let api_key = field(window, cx, &snap.api_key, true, false);
        let model = field(window, cx, &snap.model, false, false);
        let custom_model = field(window, cx, &snap.custom_model, false, false);
        let detection_model = field(window, cx, &snap.language_detection_model, false, false);
        let prompt = field(window, cx, &snap.prompt, false, true);
        let batch_size = field(window, cx, &snap.batch_size.to_string(), false, false);
        let parallel = field(
            window,
            cx,
            &snap.parallel_requests.to_string(),
            false,
            false,
        );
        let concurrency = field(window, cx, &snap.concurrency.to_string(), false, false);
        let retries = field(window, cx, &snap.max_retries.to_string(), false, false);
        let mux_language = field(window, cx, &snap.mux_language, false, false);
        let mux_title = field(window, cx, &snap.mux_title, false, false);
        let output_dir = field(window, cx, &snap.separate_output_dir, false, false);
        let ignored_styles = field(
            window,
            cx,
            &snap.text_cleaner_ignored_styles.join(", "),
            false,
            false,
        );
        let custom_tags = field(
            window,
            cx,
            &snap.text_cleaner_tags_to_remove.join(", "),
            false,
            false,
        );
        let thinking_budget = field(
            window,
            cx,
            &snap.anthropic_thinking_budget_tokens.to_string(),
            false,
            false,
        );

        let mut this = Self {
            settings: settings.clone(),
            base_url: base_url.clone(),
            api_key: api_key.clone(),
            model: model.clone(),
            custom_model: custom_model.clone(),
            detection_model: detection_model.clone(),
            prompt: prompt.clone(),
            batch_size: batch_size.clone(),
            parallel: parallel.clone(),
            concurrency: concurrency.clone(),
            retries: retries.clone(),
            mux_language: mux_language.clone(),
            mux_title: mux_title.clone(),
            output_dir: output_dir.clone(),
            ignored_styles: ignored_styles.clone(),
            custom_tags: custom_tags.clone(),
            thinking_budget: thinking_budget.clone(),
            _subs: Vec::new(),
        };

        this.bind_string(base_url, |s, v| s.base_url = v, cx);
        this.bind_string(this.api_key.clone(), |s, v| s.api_key = v, cx);
        this.bind_string(this.model.clone(), |s, v| s.model = v, cx);
        this.bind_string(this.custom_model.clone(), |s, v| s.custom_model = v, cx);
        this.bind_string(
            this.detection_model.clone(),
            |s, v| s.language_detection_model = v,
            cx,
        );
        this.bind_string(this.prompt.clone(), |s, v| s.prompt = v, cx);
        this.bind_string(this.mux_language.clone(), |s, v| s.mux_language = v, cx);
        this.bind_string(this.mux_title.clone(), |s, v| s.mux_title = v, cx);
        this.bind_string(
            this.output_dir.clone(),
            |s, v| s.separate_output_dir = v,
            cx,
        );
        this.bind_usize(this.batch_size.clone(), |s, v| s.batch_size = v.max(1), cx);
        this.bind_usize(
            this.parallel.clone(),
            |s, v| s.parallel_requests = v.max(1),
            cx,
        );
        this.bind_usize(
            this.concurrency.clone(),
            |s, v| s.concurrency = v.max(1),
            cx,
        );
        this.bind_usize(this.retries.clone(), |s, v| s.max_retries = v, cx);
        this.bind_csv(
            this.ignored_styles.clone(),
            |s, v| s.text_cleaner_ignored_styles = v,
            cx,
        );
        this.bind_csv(
            this.custom_tags.clone(),
            |s, v| s.text_cleaner_tags_to_remove = v,
            cx,
        );
        this.bind_u32(
            this.thinking_budget.clone(),
            |s, v| s.anthropic_thinking_budget_tokens = v.max(1024),
            cx,
        );
        this._subs
            .push(cx.observe(&settings, |_, _, cx| cx.notify()));

        this
    }

    fn bind_string(
        &mut self,
        input: Entity<InputState>,
        write: fn(&mut crate::core::settings::AppSettings, String),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs
            .push(cx.subscribe(&input, move |_, input, event, cx| {
                if matches!(event, InputEvent::Blur) {
                    let value = input.read(cx).value().to_string();
                    settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                }
            }));
    }

    fn bind_usize(
        &mut self,
        input: Entity<InputState>,
        write: fn(&mut crate::core::settings::AppSettings, usize),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs
            .push(cx.subscribe(&input, move |_, input, event, cx| {
                if matches!(event, InputEvent::Blur) {
                    if let Ok(value) = input.read(cx).value().parse::<usize>() {
                        settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                    }
                }
            }));
    }

    fn bind_u32(
        &mut self,
        input: Entity<InputState>,
        write: fn(&mut crate::core::settings::AppSettings, u32),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs
            .push(cx.subscribe(&input, move |_, input, event, cx| {
                if matches!(event, InputEvent::Blur) {
                    if let Ok(value) = input.read(cx).value().parse::<u32>() {
                        settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                    }
                }
            }));
    }

    fn bind_csv(
        &mut self,
        input: Entity<InputState>,
        write: fn(&mut crate::core::settings::AppSettings, Vec<String>),
        cx: &mut Context<Self>,
    ) {
        let settings = self.settings.clone();
        self._subs
            .push(cx.subscribe(&input, move |_, input, event, cx| {
                if matches!(event, InputEvent::Blur) {
                    let value = input
                        .read(cx)
                        .value()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    settings.update(cx, |state, cx| state.update(|s| write(s, value), cx));
                }
            }));
    }
}

fn field(
    window: &mut Window,
    cx: &mut Context<SettingsView>,
    value: &str,
    masked: bool,
    multiline: bool,
) -> Entity<InputState> {
    let value = value.to_string();
    cx.new(|cx| {
        let mut state = InputState::new(window, cx).default_value(value.clone());
        if masked {
            state = state.masked(true);
        }
        if multiline {
            state = state.multi_line(true);
        }
        state
    })
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.settings.read(cx).language();
        let snap = self.settings.read(cx).settings.clone();
        let ffmpeg = self.settings.read(cx).ffmpeg.clone();
        let models = self.settings.read(cx).models.clone();
        let models_loading = self.settings.read(cx).models_loading;
        let templates = self.settings.read(cx).templates.clone();

        div()
            .id("settings-scroll")
            .size_full()
            .overflow_y_scroll()
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(920.))
                    .gap_4()
                    .child(page_header(lang, cx))
                    .child(card(
                        i18n::t(lang, "settings.api.title"),
                        v_flex()
                            .gap_3()
                            .child(labeled(
                                lang,
                                "settings.api.baseUrl",
                                Input::new(&self.base_url),
                            ))
                            .child(labeled(
                                lang,
                                "settings.api.apiKey",
                                Input::new(&self.api_key),
                            ))
                            .child(format_picker(lang, snap.api_format, self.settings.clone()))
                            .child(labeled(lang, "settings.api.model", Input::new(&self.model)))
                            .child(labeled(
                                lang,
                                "settings.api.customModel",
                                Input::new(&self.custom_model),
                            ))
                            .child(labeled(
                                lang,
                                "settings.api.languageDetectionModel",
                                Input::new(&self.detection_model),
                            ))
                            .child(models_row(
                                lang,
                                models_loading,
                                models,
                                self.settings.clone(),
                                cx,
                            ))
                            .into_any_element(),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.language.title"),
                        language_picker(lang, self.settings.clone()),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.prompt.title"),
                        v_flex()
                            .gap_3()
                            .child(Input::new(&self.prompt).h(px(140.)))
                            .child(templates_row(lang, templates, self.settings.clone(), cx))
                            .into_any_element(),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.ffmpeg.title"),
                        ffmpeg_row(lang, ffmpeg, self.settings.clone(), cx),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.translationSettings.title"),
                        v_flex()
                            .gap_3()
                            .child(labeled(
                                lang,
                                "settings.translationSettings.batchSize",
                                Input::new(&self.batch_size),
                            ))
                            .child(labeled(
                                lang,
                                "settings.translationSettings.parallelRequests",
                                Input::new(&self.parallel),
                            ))
                            .child(labeled(
                                lang,
                                "settings.translationSettings.concurrency",
                                Input::new(&self.concurrency),
                            ))
                            .child(labeled(
                                lang,
                                "settings.translationSettings.maxRetries",
                                Input::new(&self.retries),
                            ))
                            .child(reasoning_picker(
                                lang,
                                snap.reasoning_effort,
                                self.settings.clone(),
                            ))
                            .child(
                                Switch::new("streaming")
                                    .label(i18n::t(lang, "settings.translationSettings.streaming"))
                                    .checked(snap.streaming)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(|cfg| cfg.streaming = *checked, cx)
                                        });
                                    })),
                            )
                            .child(
                                Switch::new("auto-continue")
                                    .label(i18n::t(
                                        lang,
                                        "settings.translationSettings.autoContinue",
                                    ))
                                    .checked(snap.auto_continue)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(|cfg| cfg.auto_continue = *checked, cx)
                                        });
                                    })),
                            )
                            .child(
                                Switch::new("continue-error")
                                    .label(i18n::t(
                                        lang,
                                        "settings.translationSettings.continueOnError",
                                    ))
                                    .checked(snap.continue_on_error)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(|cfg| cfg.continue_on_error = *checked, cx)
                                        });
                                    })),
                            )
                            .child(
                                Switch::new("anthropic-thinking")
                                    .label(i18n::t(
                                        lang,
                                        "settings.translationSettings.anthropicThinking",
                                    ))
                                    .checked(snap.anthropic_thinking_enabled)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(
                                                |cfg| cfg.anthropic_thinking_enabled = *checked,
                                                cx,
                                            )
                                        });
                                    })),
                            )
                            .child(labeled(
                                lang,
                                "settings.translationSettings.anthropicThinkingBudget",
                                Input::new(&self.thinking_budget),
                            ))
                            .into_any_element(),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.output.title"),
                        v_flex()
                            .gap_3()
                            .child(output_mode_picker(
                                lang,
                                &snap.output_mode,
                                self.settings.clone(),
                            ))
                            .child(labeled(
                                lang,
                                "settings.output.languageCode",
                                Input::new(&self.mux_language),
                            ))
                            .child(labeled(
                                lang,
                                "settings.output.trackTitle",
                                Input::new(&self.mux_title),
                            ))
                            .child(labeled(
                                lang,
                                "settings.output.outputFolder",
                                Input::new(&self.output_dir),
                            ))
                            .child(
                                Switch::new("cleanup-extracted")
                                    .label(i18n::t(lang, "settings.output.cleanupExtracted"))
                                    .checked(snap.cleanup_extracted_subtitles)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(
                                                |cfg| cfg.cleanup_extracted_subtitles = *checked,
                                                cx,
                                            )
                                        });
                                    })),
                            )
                            .child(
                                Switch::new("cleanup-mux")
                                    .label(i18n::t(lang, "settings.output.cleanupMux"))
                                    .checked(snap.cleanup_mux_artifacts)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(|cfg| cfg.cleanup_mux_artifacts = *checked, cx)
                                        });
                                    })),
                            )
                            .into_any_element(),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.textCleaner.title"),
                        v_flex()
                            .gap_3()
                            .child(
                                Switch::new("cleaner")
                                    .label(i18n::t(lang, "settings.textCleaner.enabled"))
                                    .checked(snap.text_cleaner_enabled)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(|cfg| cfg.text_cleaner_enabled = *checked, cx)
                                        });
                                    })),
                            )
                            .child(
                                Checkbox::new("preserve-basic")
                                    .label(i18n::t(lang, "settings.textCleaner.preserveBasic"))
                                    .checked(snap.text_cleaner_preserve_basic_formatting)
                                    .on_click(cx.listener(|this, checked, _, cx| {
                                        this.settings.update(cx, |s, cx| {
                                            s.update(
                                                |cfg| {
                                                    cfg.text_cleaner_preserve_basic_formatting =
                                                        *checked
                                                },
                                                cx,
                                            )
                                        });
                                    })),
                            )
                            .child(labeled(
                                lang,
                                "settings.textCleaner.ignoredStyles",
                                Input::new(&self.ignored_styles),
                            ))
                            .child(labeled(
                                lang,
                                "settings.textCleaner.customTags",
                                Input::new(&self.custom_tags),
                            ))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(i18n::t(
                                        lang,
                                        "settings.textCleaner.whatItDoesDescription",
                                    )),
                            )
                            .into_any_element(),
                        cx,
                    ))
                    .child(card(
                        i18n::t(lang, "settings.appData.title"),
                        v_flex()
                            .gap_2()
                            .child(
                                Button::new("open-config")
                                    .outline()
                                    .label(i18n::t(lang, "settings.appData.openConfigFolder"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.read(cx).open_config_folder();
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(i18n::t(lang, "settings.appData.configFilesHint")),
                            )
                            .into_any_element(),
                        cx,
                    )),
            )
    }
}

fn page_header(lang: Language, cx: &App) -> impl IntoElement {
    v_flex()
        .gap_1()
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
        )
}

fn card(title: String, body: impl IntoElement, cx: &App) -> impl IntoElement {
    v_flex()
        .w_full()
        .gap_3()
        .p_5()
        .rounded_xl()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(body)
}

fn labeled(lang: Language, key: &str, input: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(div().text_sm().child(i18n::t(lang, key)))
        .child(input)
}

fn format_picker(
    lang: Language,
    current: ApiFormat,
    settings: Entity<SettingsState>,
) -> impl IntoElement {
    h_flex().gap_2().children(
        [
            (ApiFormat::Auto, "settings.api.autoDetect"),
            (ApiFormat::OpenAI, "OpenAI"),
            (ApiFormat::Anthropic, "Anthropic"),
        ]
        .into_iter()
        .map(|(format, key)| {
            let settings = settings.clone();
            let selected = current == format;
            let label = if key.contains('.') {
                i18n::t(lang, key)
            } else {
                key.to_string()
            };
            let mut btn = Button::new(key)
                .small()
                .label(label)
                .on_click(move |_, _, cx| {
                    settings.update(cx, |s, cx| {
                        s.update(|cfg| cfg.api_format = format.clone(), cx)
                    });
                });
            if selected {
                btn = btn.primary();
            } else {
                btn = btn.ghost();
            }
            btn
        }),
    )
}

fn language_picker(lang: Language, settings: Entity<SettingsState>) -> impl IntoElement {
    h_flex().gap_2().children(
        [
            (Language::English, "English"),
            (Language::PortugueseBrazil, "Português (Brasil)"),
        ]
        .into_iter()
        .map(|(item, label)| {
            let settings = settings.clone();
            let selected = lang == item;
            let mut btn = Button::new(label)
                .small()
                .label(label)
                .on_click(move |_, _, cx| {
                    settings.update(cx, |s, cx| s.set_language(item, cx));
                });
            if selected {
                btn = btn.primary();
            } else {
                btn = btn.ghost();
            }
            btn
        }),
    )
}

fn reasoning_picker(
    lang: Language,
    current: ReasoningEffort,
    settings: Entity<SettingsState>,
) -> impl IntoElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .text_sm()
                .child(i18n::t(lang, "settings.translationSettings.thinkingMode")),
        )
        .child(
            h_flex().gap_1().flex_wrap().children(
                [
                    (
                        ReasoningEffort::Default,
                        "settings.translationSettings.thinkingDefault",
                    ),
                    (
                        ReasoningEffort::None,
                        "settings.translationSettings.thinkingNone",
                    ),
                    (
                        ReasoningEffort::Minimal,
                        "settings.translationSettings.thinkingMinimal",
                    ),
                    (
                        ReasoningEffort::Low,
                        "settings.translationSettings.thinkingLow",
                    ),
                    (
                        ReasoningEffort::Medium,
                        "settings.translationSettings.thinkingMedium",
                    ),
                    (
                        ReasoningEffort::High,
                        "settings.translationSettings.thinkingHigh",
                    ),
                    (
                        ReasoningEffort::Xhigh,
                        "settings.translationSettings.thinkingXHigh",
                    ),
                ]
                .into_iter()
                .map(|(effort, key)| {
                    let settings = settings.clone();
                    let selected = current == effort;
                    let mut btn = Button::new(key).small().label(i18n::t(lang, key)).on_click(
                        move |_, _, cx| {
                            settings.update(cx, |s, cx| {
                                s.update(|cfg| cfg.reasoning_effort = effort.clone(), cx)
                            });
                        },
                    );
                    if selected {
                        btn = btn.primary();
                    } else {
                        btn = btn.ghost();
                    }
                    btn
                }),
            ),
        )
}

fn output_mode_picker(
    lang: Language,
    current: &str,
    settings: Entity<SettingsState>,
) -> impl IntoElement {
    h_flex().gap_2().children(
        [
            ("separate", "settings.output.separateFile"),
            ("mux", "settings.output.muxVideo"),
        ]
        .into_iter()
        .map(|(mode, key)| {
            let settings = settings.clone();
            let selected = current == mode;
            let mut btn = Button::new(mode)
                .small()
                .label(i18n::t(lang, key))
                .on_click(move |_, _, cx| {
                    settings.update(cx, |s, cx| {
                        s.update(|cfg| cfg.output_mode = mode.to_string(), cx)
                    });
                });
            if selected {
                btn = btn.primary();
            } else {
                btn = btn.ghost();
            }
            btn
        }),
    )
}

fn models_row(
    lang: Language,
    loading: bool,
    models: Vec<crate::core::translator::LlmModel>,
    settings: Entity<SettingsState>,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .gap_2()
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("refresh-models")
                        .small()
                        .outline()
                        .label(if loading {
                            i18n::t(lang, "common.loading")
                        } else {
                            i18n::t(lang, "settings.api.searchModels")
                        })
                        .on_click({
                            let settings = settings.clone();
                            move |_, _, cx| {
                                settings.update(cx, |s, cx| s.refresh_models(cx));
                            }
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} models", models.len())),
                ),
        )
        .when(!models.is_empty(), |this| {
            this.child(
                h_flex()
                    .gap_1()
                    .flex_wrap()
                    .children(models.into_iter().take(12).map(|model| {
                        let settings = settings.clone();
                        let id = model.id.clone();
                        Button::new(SharedString::from(id.clone()))
                            .small()
                            .ghost()
                            .label(model.name.unwrap_or(id.clone()))
                            .on_click(move |_, _, cx| {
                                let id = id.clone();
                                settings.update(cx, |s, cx| s.update(|cfg| cfg.model = id, cx));
                            })
                    })),
            )
        })
}

fn templates_row(
    lang: Language,
    templates: Vec<crate::core::templates::PromptTemplate>,
    settings: Entity<SettingsState>,
    _cx: &App,
) -> impl IntoElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .text_sm()
                .child(i18n::t(lang, "settings.templates.title")),
        )
        .when(templates.is_empty(), |this| {
            this.child(
                div()
                    .text_xs()
                    .child(i18n::t(lang, "settings.templates.noTemplates")),
            )
        })
        .children(templates.into_iter().map(|template| {
            let settings_apply = settings.clone();
            let settings_del = settings.clone();
            let id = template.id.clone();
            let id_del = template.id.clone();
            h_flex()
                .gap_2()
                .items_center()
                .child(div().flex_1().text_sm().child(template.name))
                .child(
                    Button::new(SharedString::from(format!("apply-{}", id)))
                        .small()
                        .ghost()
                        .label(i18n::t(lang, "common.select"))
                        .on_click(move |_, _, cx| {
                            settings_apply.update(cx, |s, cx| s.apply_template(&id, cx));
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("del-{}", id_del)))
                        .small()
                        .danger()
                        .label(i18n::t(lang, "common.delete"))
                        .on_click(move |_, _, cx| {
                            let _ = settings_del.update(cx, |s, cx| s.delete_template(&id_del, cx));
                        }),
                )
        }))
}

fn ffmpeg_row(
    lang: Language,
    ffmpeg: Option<Result<String, String>>,
    settings: Entity<SettingsState>,
    cx: &App,
) -> impl IntoElement {
    let (label, color) = match &ffmpeg {
        Some(Ok(v)) => (
            format!("{}: {v}", i18n::t(lang, "settings.ffmpeg.installed")),
            cx.theme().success,
        ),
        Some(Err(_)) => (
            i18n::t(lang, "settings.ffmpeg.notFoundDescription"),
            cx.theme().danger,
        ),
        None => (
            i18n::t(lang, "settings.ffmpeg.checking"),
            cx.theme().muted_foreground,
        ),
    };

    v_flex()
        .gap_2()
        .child(div().text_sm().text_color(color).child(label))
        .child(
            Button::new("check-ffmpeg")
                .small()
                .outline()
                .label(i18n::t(lang, "settings.ffmpeg.check"))
                .on_click(move |_, _, cx| {
                    settings.update(cx, |s, cx| s.refresh_ffmpeg(cx));
                }),
        )
}
