use gpui::prelude::*;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::i18n::{self, Language};
use crate::state::queue::QueueFile;

#[derive(IntoElement)]
pub struct SubtitleEditor {
    file: QueueFile,
    lang: Language,
}

impl SubtitleEditor {
    pub fn new(file: QueueFile, lang: Language) -> Self {
        Self { file, lang }
    }
}

impl RenderOnce for SubtitleEditor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let original = self
            .file
            .original
            .as_ref()
            .map(|s| s.entries.clone())
            .unwrap_or_default();
        let translated = self
            .file
            .translated_entries
            .clone()
            .unwrap_or_else(|| original.clone());
        let count = original.len().max(translated.len());

        v_flex()
            .size_full()
            .gap_3()
            .p_4()
            .rounded_xl()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.file.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} {}",
                                count,
                                i18n::t(self.lang, "translation.editor.lines")
                            )),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(self.lang, "translation.editor.original")),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(self.lang, "translation.editor.translated")),
                    ),
            )
            .child(
                div()
                    .id("editor-scroll")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .child(v_flex().w_full().gap_2().children((0..count).map(|i| {
                        let orig = original.get(i).map(|e| e.text.clone()).unwrap_or_default();
                        let trans = translated
                            .get(i)
                            .map(|e| e.text.clone())
                            .unwrap_or_default();
                        let time = original
                            .get(i)
                            .map(|e| format!("{} → {}", e.start_time, e.end_time))
                            .unwrap_or_default();
                        h_flex()
                            .w_full()
                            .gap_3()
                            .p_2()
                            .rounded_md()
                            .bg(cx.theme().muted.opacity(0.25))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(time),
                                    )
                                    .child(div().text_sm().child(orig)),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(div().text_sm().child(trans)),
                            )
                    }))),
            )
    }
}

pub fn empty_editor(lang: Language, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .min_h(px(320.))
        .items_center()
        .justify_center()
        .rounded_xl()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted.opacity(0.2))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(i18n::t(lang, "translation.editor.empty")),
        )
}
