use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::i18n::{self, Language};
use crate::state::logs::{LogLevel, LogsState};

#[derive(IntoElement)]
pub struct LogsPanel {
    logs: Entity<LogsState>,
    lang: Language,
}

impl LogsPanel {
    pub fn new(logs: Entity<LogsState>, lang: Language) -> Self {
        Self { logs, lang }
    }
}

impl RenderOnce for LogsPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.logs.read(cx);
        let filter = state.filter;
        let entries: Vec<_> = state.visible().cloned().collect();
        let lang = self.lang;
        let logs = self.logs.clone();

        v_flex()
            .w_full()
            .h(px(240.))
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .p_3()
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(i18n::t(lang, "logs.title")),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .children(
                                [
                                    (None, "logs.filter.all"),
                                    (Some(LogLevel::Info), "logs.filter.info"),
                                    (Some(LogLevel::Success), "logs.filter.success"),
                                    (Some(LogLevel::Warning), "logs.filter.warning"),
                                    (Some(LogLevel::Error), "logs.filter.error"),
                                ]
                                .into_iter()
                                .map(|(level, key)| {
                                    let logs = logs.clone();
                                    let selected = filter == level;
                                    let mut btn = Button::new(key)
                                        .small()
                                        .label(i18n::t(lang, key))
                                        .on_click(move |_, _, cx| {
                                            logs.update(cx, |logs, cx| logs.set_filter(level, cx));
                                        });
                                    if selected {
                                        btn = btn.primary();
                                    } else {
                                        btn = btn.ghost();
                                    }
                                    btn
                                }),
                            )
                            .child({
                                let logs = self.logs.clone();
                                Button::new("clear-logs")
                                    .ghost()
                                    .small()
                                    .label(i18n::t(lang, "common.clear"))
                                    .on_click(move |_, _, cx| {
                                        logs.update(cx, |logs, cx| logs.clear(cx));
                                    })
                            })
                            .child({
                                let logs = self.logs.clone();
                                Button::new("close-logs")
                                    .ghost()
                                    .small()
                                    .label(i18n::t(lang, "common.cancel"))
                                    .on_click(move |_, _, cx| {
                                        logs.update(cx, |logs, cx| logs.set_drawer_open(false, cx));
                                    })
                            }),
                    ),
            )
            .child(
                div()
                    .id("logs-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(if entries.is_empty() {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(lang, "logs.empty"))
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap_1()
                            .children(entries.into_iter().rev().take(80).map(|entry| {
                                let color = match entry.level {
                                    LogLevel::Error => cx.theme().danger,
                                    LogLevel::Warning => cx.theme().warning,
                                    LogLevel::Success => cx.theme().success,
                                    LogLevel::Info => cx.theme().muted_foreground,
                                };
                                h_flex()
                                    .gap_2()
                                    .items_start()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(entry.timestamp_label),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(color)
                                            .child(LogsState::level_label(lang, entry.level)),
                                    )
                                    .child(div().text_sm().flex_1().child(entry.message))
                            }))
                            .into_any_element()
                    }),
            )
    }
}
