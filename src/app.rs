use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{Root, Sizable};

use crate::i18n::{self, Language};
use crate::state::logs::LogsState;
use crate::state::queue::QueueState;
use crate::state::settings::SettingsState;
use crate::views::{logs::LogsPanel, settings::SettingsView, translation::TranslationView};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Translation,
    Settings,
}

pub struct RootView {
    tab: AppTab,
    settings: Entity<SettingsState>,
    queue: Entity<QueueState>,
    logs: Entity<LogsState>,
    translation: Entity<TranslationView>,
    settings_view: Entity<SettingsView>,
    _subscriptions: Vec<Subscription>,
}

impl RootView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = cx.new(SettingsState::new);
        let logs = cx.new(LogsState::new);
        let queue = cx.new(|cx| QueueState::new(settings.clone(), logs.clone(), cx));

        let language = settings.read(cx).language();
        gpui_component::set_locale(language.code());

        let translation =
            cx.new(|cx| TranslationView::new(queue.clone(), settings.clone(), window, cx));
        let settings_view = cx.new(|cx| SettingsView::new(settings.clone(), window, cx));

        let mut this = Self {
            tab: AppTab::Translation,
            settings: settings.clone(),
            queue: queue.clone(),
            logs: logs.clone(),
            translation,
            settings_view,
            _subscriptions: Vec::new(),
        };

        this._subscriptions
            .push(cx.observe(&settings, |_, _, cx| cx.notify()));
        this._subscriptions
            .push(cx.observe(&queue, |_, _, cx| cx.notify()));
        this._subscriptions
            .push(cx.observe(&logs, |_, _, cx| cx.notify()));
        this
    }

    fn language(&self, cx: &App) -> Language {
        self.settings.read(cx).language()
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.language(cx);
        let bg = rgb(0x0c0c0e);
        let surface = rgb(0x16161a);
        let border = rgb(0x27272a);
        let fg = rgb(0xe4e4e7);

        let queue = self.queue.read(cx);
        let is_processing = queue.processing_count() > 0 || queue.is_translating;
        let is_paused = queue.is_paused;
        let pending = queue.pending_count();
        let ffmpeg_ok = self.settings.read(cx).ffmpeg_ok();
        let error_count = self.logs.read(cx).error_count();
        let logs_open = self.logs.read(cx).drawer_open;

        let translate_label = if is_processing && is_paused {
            i18n::t(lang, "navbar.resume")
        } else if is_processing {
            i18n::t(lang, "navbar.pause")
        } else {
            i18n::t(lang, "navbar.translate")
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(bg)
            .text_color(fg)
            .font_family(".SystemUIFont")
            .child(
                div()
                    .w_full()
                    .h(px(56.))
                    .px_4()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .bg(surface)
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(i18n::t(lang, "navbar.title")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .child(tab_button(
                                        "tab-translation",
                                        i18n::t(lang, "navbar.translation"),
                                        self.tab == AppTab::Translation,
                                        cx.listener(|this, _, _, cx| {
                                            this.tab = AppTab::Translation;
                                            cx.notify();
                                        }),
                                    ))
                                    .child(tab_button(
                                        "tab-settings",
                                        i18n::t(lang, "navbar.settings"),
                                        self.tab == AppTab::Settings,
                                        cx.listener(|this, _, _, cx| {
                                            this.tab = AppTab::Settings;
                                            cx.notify();
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xa1a1aa))
                                    .child(if ffmpeg_ok {
                                        String::new()
                                    } else {
                                        i18n::t(lang, "settings.ffmpeg.notFound")
                                    }),
                            )
                            .child(
                                Button::new("btn-translate")
                                    .label(format!(
                                        "{} ({pending} {})",
                                        translate_label,
                                        i18n::t(lang, "navbar.inQueue")
                                    ))
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.queue.update(cx, |queue, cx| {
                                            if is_processing {
                                                if is_paused {
                                                    queue.resume(cx);
                                                } else {
                                                    queue.pause(cx);
                                                }
                                            } else {
                                                queue.start(cx);
                                            }
                                        });
                                    })),
                            )
                            .child(
                                Button::new("btn-logs")
                                    .label(if error_count > 0 {
                                        format!("{} ({error_count})", i18n::t(lang, "navbar.logs"))
                                    } else {
                                        i18n::t(lang, "navbar.logs")
                                    })
                                    .small()
                                    .ghost()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.logs.update(cx, |logs, cx| logs.toggle_drawer(cx));
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("app-body")
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .p_4()
                    .child(match self.tab {
                        AppTab::Translation => this_view(&self.translation),
                        AppTab::Settings => this_view(&self.settings_view),
                    }),
            )
            .when(logs_open, |this| {
                this.child(LogsPanel::new(self.logs.clone(), lang))
            })
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
    }
}

fn this_view<V: Render>(entity: &Entity<V>) -> AnyElement {
    entity.clone().into_any_element()
}

fn tab_button(
    id: &'static str,
    label: String,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Button {
    let btn = Button::new(id).label(label).small().on_click(on_click);
    if selected {
        btn.primary()
    } else {
        btn.ghost()
    }
}
