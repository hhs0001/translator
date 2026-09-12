use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::notification::Notification;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, Root, Sizable, WindowExt};

use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::logs::LogsState;
use crate::state::queue::{QueueNotice, QueueState};
use crate::state::settings::SettingsState;
use crate::views::ui;
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

        // Files passed on the command line (also covers drag-onto-exe / "Open with").
        let startup_paths: Vec<std::path::PathBuf> = std::env::args()
            .skip(1)
            .map(std::path::PathBuf::from)
            .filter(|path| path.exists())
            .collect();
        if !startup_paths.is_empty() {
            queue.update(cx, |queue, cx| queue.add_paths(startup_paths, cx));
        }

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
        this._subscriptions
            .push(cx.subscribe_in(&queue, window, Self::on_queue_notice));
        this
    }

    fn on_queue_notice(
        &mut self,
        _: &Entity<QueueState>,
        notice: &QueueNotice,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let lang = self.language(cx);
        let note = match notice {
            QueueNotice::FileCompleted { name, output } => Notification::success(
                output
                    .clone()
                    .map(|path| ui::short_path(&path, 64))
                    .unwrap_or_default(),
            )
            .title(i18n::tf(
                lang,
                "notifications.fileCompleted",
                &[("fileName", name)],
            )),
            QueueNotice::FileFailed { name, error } => Notification::error(error.clone()).title(
                i18n::tf(lang, "notifications.fileFailed", &[("fileName", name)]),
            ),
            QueueNotice::AllDone { count } => {
                if *count == 0 {
                    return;
                }
                Notification::success(i18n::tf(
                    lang,
                    "notifications.allDone",
                    &[("count", &count.to_string())],
                ))
            }
        };
        window.push_notification(note, cx);
    }

    fn language(&self, cx: &App) -> Language {
        self.settings.read(cx).language()
    }

    fn nav_tab(
        &self,
        tab: AppTab,
        icon: Ico,
        label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.tab == tab;
        let id = SharedString::from(match tab {
            AppTab::Translation => "tab-translation",
            AppTab::Settings => "tab-settings",
        });
        let (fg, bg) = if active {
            (cx.theme().foreground, cx.theme().background.opacity(0.9))
        } else {
            (cx.theme().muted_foreground, gpui::transparent_black())
        };

        h_flex()
            .id(id)
            .gap_2()
            .items_center()
            .px_3()
            .py(px(6.))
            .rounded(px(8.))
            .bg(bg)
            .text_sm()
            .text_color(fg)
            .when(active, |this| this.font_weight(FontWeight::MEDIUM))
            .cursor_pointer()
            .hover(|this| this.text_color(cx.theme().foreground))
            .child(icon.icon().small())
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.tab = tab;
                cx.notify();
            }))
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.language(cx);
        let queue = self.queue.read(cx);
        let is_processing = queue.processing_count() > 0;
        let is_running = queue.is_translating || is_processing;
        let is_paused = queue.is_paused;
        let pending = queue.pending_count();
        let has_files = !queue.files.is_empty();
        let ffmpeg_ok = self.settings.read(cx).ffmpeg_ok();
        let ffmpeg_checked = self.settings.read(cx).ffmpeg.is_some();
        let dark = self.settings.read(cx).is_dark();
        let error_count = self.logs.read(cx).error_count();
        let logs_open = self.logs.read(cx).drawer_open;

        let (action_label, action_icon) = if is_running && is_paused {
            (i18n::t(lang, "navbar.resume"), Ico::Play)
        } else if is_running {
            (i18n::t(lang, "navbar.pause"), Ico::Pause)
        } else {
            (i18n::t(lang, "navbar.translate"), Ico::Play)
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(".SystemUIFont")
            .child(
                h_flex()
                    .w_full()
                    .h(px(58.))
                    .flex_none()
                    .px_5()
                    .items_center()
                    .justify_between()
                    .bg(cx.theme().title_bar)
                    .border_b_1()
                    .border_color(cx.theme().border.opacity(0.8))
                    .child(
                        h_flex()
                            .gap_5()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2p5()
                                    .items_center()
                                    .child(
                                        div()
                                            .size(px(30.))
                                            .rounded(px(9.))
                                            .bg(cx.theme().primary)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                Ico::Languages
                                                    .icon()
                                                    .small()
                                                    .text_color(cx.theme().primary_foreground),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(i18n::t(lang, "navbar.title")),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .p(px(3.))
                                    .rounded(px(10.))
                                    .bg(cx.theme().muted.opacity(0.55))
                                    .child(self.nav_tab(
                                        AppTab::Translation,
                                        Ico::FileText,
                                        i18n::t(lang, "navbar.translation"),
                                        cx,
                                    ))
                                    .child(self.nav_tab(
                                        AppTab::Settings,
                                        Ico::Settings,
                                        i18n::t(lang, "navbar.settings"),
                                        cx,
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .when(ffmpeg_checked, |this| {
                                this.child(ui::chip(
                                    "FFmpeg",
                                    if ffmpeg_ok {
                                        cx.theme().success
                                    } else {
                                        cx.theme().danger
                                    },
                                    Some(if ffmpeg_ok {
                                        Ico::CircleCheck
                                    } else {
                                        Ico::CircleX
                                    }),
                                    cx,
                                ))
                            })
                            .when(pending > 0, |this| {
                                this.child(ui::chip(
                                    format!("{pending} {}", i18n::t(lang, "navbar.inQueue")),
                                    cx.theme().info,
                                    Some(Ico::Layers),
                                    cx,
                                ))
                            })
                            .child(
                                Button::new("toggle-theme")
                                    .ghost()
                                    .small()
                                    .icon(if dark {
                                        Ico::Sun.icon()
                                    } else {
                                        Ico::Moon.icon()
                                    })
                                    .tooltip(i18n::t(lang, "navbar.toggleTheme"))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.settings.update(cx, |settings, cx| {
                                            settings.set_dark(!dark, window, cx);
                                        });
                                    })),
                            )
                            .child({
                                let label = if error_count > 0 {
                                    format!("{} · {error_count}", i18n::t(lang, "navbar.logs"))
                                } else {
                                    i18n::t(lang, "navbar.logs")
                                };
                                let mut btn = Button::new("btn-logs")
                                    .small()
                                    .icon(Ico::ScrollText.icon())
                                    .label(label)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.logs.update(cx, |logs, cx| logs.toggle_drawer(cx));
                                    }));
                                btn = if logs_open {
                                    btn.with_variant(
                                        gpui_component::button::ButtonVariant::Secondary,
                                    )
                                } else if error_count > 0 {
                                    btn.danger()
                                } else {
                                    btn.ghost()
                                };
                                btn
                            })
                            .when(is_running, |this| {
                                this.child(
                                    Button::new("btn-stop")
                                        .small()
                                        .danger()
                                        .icon(Ico::Ban.icon())
                                        .tooltip(i18n::t(lang, "translation.queue.cancelAll"))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.queue.update(cx, |queue, cx| queue.cancel_all(cx));
                                        })),
                                )
                            })
                            .child(
                                Button::new("btn-translate")
                                    .label(action_label)
                                    .icon(action_icon.icon())
                                    .small()
                                    .primary()
                                    .disabled(!has_files || (!is_running && pending == 0))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.queue.update(cx, |queue, cx| {
                                            if is_running {
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
                            ),
                    ),
            )
            .child(
                div()
                    .id("app-body")
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .px_6()
                    .py_5()
                    .child(match self.tab {
                        AppTab::Translation => self.translation.clone().into_any_element(),
                        AppTab::Settings => self.settings_view.clone().into_any_element(),
                    }),
            )
            .when(logs_open, |this| {
                this.child(LogsPanel::new(self.logs.clone(), lang))
            })
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
