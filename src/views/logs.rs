use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, Sizable};

use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::logs::{LogEntry, LogLevel, LogsState};
use crate::views::ui;

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

fn level_color(level: LogLevel, cx: &App) -> Hsla {
    match level {
        LogLevel::Error => cx.theme().danger,
        LogLevel::Warning => cx.theme().warning,
        LogLevel::Success => cx.theme().success,
        LogLevel::Info => cx.theme().info,
    }
}

fn level_icon(level: LogLevel) -> Ico {
    match level {
        LogLevel::Error => Ico::CircleX,
        LogLevel::Warning => Ico::TriangleAlert,
        LogLevel::Success => Ico::CircleCheck,
        LogLevel::Info => Ico::Info,
    }
}

impl RenderOnce for LogsPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.logs.read(cx);
        let filter = state.filter;
        let total = state.entries.len();
        let mut entries: Vec<LogEntry> = state.visible().cloned().collect();
        entries.reverse();
        entries.truncate(300);
        let counts = [
            (None, "logs.filter.all", total),
            (
                Some(LogLevel::Info),
                "logs.filter.info",
                state.count_of(LogLevel::Info),
            ),
            (
                Some(LogLevel::Success),
                "logs.filter.success",
                state.count_of(LogLevel::Success),
            ),
            (
                Some(LogLevel::Warning),
                "logs.filter.warning",
                state.count_of(LogLevel::Warning),
            ),
            (
                Some(LogLevel::Error),
                "logs.filter.error",
                state.count_of(LogLevel::Error),
            ),
        ];
        let lang = self.lang;
        let logs = self.logs.clone();

        v_flex()
            .w_full()
            .h(px(260.))
            .flex_none()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.5))
            .child(
                h_flex()
                    .w_full()
                    .px_4()
                    .py_2p5()
                    .gap_3()
                    .justify_between()
                    .items_center()
                    .border_b_1()
                    .border_color(cx.theme().border.opacity(0.7))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Ico::ScrollText
                                    .icon()
                                    .small()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(i18n::t(lang, "logs.title")),
                            )
                            .child(
                                h_flex()
                                    .ml_2()
                                    .gap_1()
                                    .p(px(2.))
                                    .rounded(px(8.))
                                    .bg(cx.theme().muted.opacity(0.5))
                                    .children(counts.into_iter().map(|(level, key, count)| {
                                        let logs = logs.clone();
                                        let selected = filter == level;
                                        let color = level
                                            .map(|level| level_color(level, cx))
                                            .unwrap_or(cx.theme().foreground);
                                        h_flex()
                                            .id(SharedString::from(key))
                                            .gap_1p5()
                                            .items_center()
                                            .px_2()
                                            .py(px(3.))
                                            .rounded(px(6.))
                                            .text_xs()
                                            .cursor_pointer()
                                            .when(selected, |this| {
                                                this.bg(cx.theme().background)
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(color)
                                            })
                                            .when(!selected, |this| {
                                                this.text_color(cx.theme().muted_foreground)
                                            })
                                            .child(i18n::t(lang, key))
                                            .child(
                                                div()
                                                    .text_color(
                                                        cx.theme().muted_foreground.opacity(0.8),
                                                    )
                                                    .child(count.to_string()),
                                            )
                                            .on_click(move |_, _, cx| {
                                                logs.update(cx, |logs, cx| {
                                                    logs.set_filter(level, cx)
                                                });
                                            })
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child({
                                let logs = logs.clone();
                                Button::new("copy-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::Save.icon())
                                    .tooltip(i18n::t(lang, "logs.copy"))
                                    .disabled(total == 0)
                                    .on_click(move |_, _, cx| {
                                        let text = logs.read(cx).as_text();
                                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                                    })
                            })
                            .child({
                                let logs = logs.clone();
                                Button::new("clear-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::Trash.icon())
                                    .label(i18n::t(lang, "common.clear"))
                                    .disabled(total == 0)
                                    .on_click(move |_, _, cx| {
                                        logs.update(cx, |logs, cx| logs.clear(cx));
                                    })
                            })
                            .child({
                                let logs = logs.clone();
                                Button::new("close-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::Close.icon())
                                    .tooltip(i18n::t(lang, "logs.close"))
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
                    .px_3()
                    .py_2()
                    .child(if entries.is_empty() {
                        ui::empty_state(Ico::ScrollText, i18n::t(lang, "logs.empty"), None, cx)
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap(px(1.))
                            .children(entries.into_iter().map(|entry| {
                                let color = level_color(entry.level, cx);
                                h_flex()
                                    .w_full()
                                    .gap_2p5()
                                    .items_start()
                                    .px_2()
                                    .py(px(4.))
                                    .rounded(px(6.))
                                    .hover(|this| this.bg(cx.theme().muted.opacity(0.35)))
                                    .child(
                                        ui::mono(entry.timestamp_label.clone(), cx)
                                            .flex_none()
                                            .text_color(cx.theme().muted_foreground.opacity(0.75)),
                                    )
                                    .child(
                                        level_icon(entry.level)
                                            .icon()
                                            .xsmall()
                                            .flex_shrink_0()
                                            .text_color(color),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_xs()
                                            .whitespace_normal()
                                            .child(entry.message.clone()),
                                    )
                                    .when_some(entry.file.clone(), |this, file| {
                                        this.child(ui::meta_chip(file, Some(Ico::FileText), cx))
                                    })
                            }))
                            .into_any_element()
                    }),
            )
    }
}
