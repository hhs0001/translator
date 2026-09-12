use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::queue::QueueState;
use crate::state::settings::SettingsState;
use crate::views::ui;

#[derive(IntoElement)]
pub struct DropZone {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    compact: bool,
}

impl DropZone {
    pub fn new(queue: Entity<QueueState>, settings: Entity<SettingsState>) -> Self {
        Self {
            queue,
            settings,
            compact: false,
        }
    }

    /// Slimmer variant, used once the queue already has files.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    fn language(&self, cx: &App) -> Language {
        self.settings.read(cx).language()
    }

    pub fn pick_files(queue: Entity<QueueState>, cx: &mut App) {
        let rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Select subtitle or video files".into()),
        });
        cx.spawn(async move |cx| {
            if let Ok(Ok(Some(paths))) = rx.await {
                let _ = queue.update(cx, |queue, cx| queue.add_paths(paths, cx));
            }
        })
        .detach();
    }
}

impl RenderOnce for DropZone {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let lang = self.language(cx);
        let queue = self.queue.clone();
        let queue_drop = self.queue.clone();
        let accent = cx.theme().primary;

        let container = div()
            .id("dropzone")
            .w_full()
            .rounded(px(16.))
            .border_1()
            .border_dashed()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted.opacity(0.25))
            .cursor_pointer()
            .hover(|this| {
                this.border_color(accent.opacity(0.65))
                    .bg(ui::tint(accent, 0.06))
            })
            .drag_over::<ExternalPaths>(move |style, _, _, _| {
                style.border_color(accent).bg(ui::tint(accent, 0.12))
            })
            .on_drop(move |paths: &ExternalPaths, _, cx| {
                let paths = paths.paths().to_vec();
                queue_drop.update(cx, |queue, cx| queue.add_paths(paths, cx));
            })
            .on_click({
                let queue = queue.clone();
                move |_, _, cx| DropZone::pick_files(queue.clone(), cx)
            });

        if self.compact {
            return container
                .px_4()
                .py_3()
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_center()
                        .child(ui::icon_badge(Ico::CloudUpload, accent, cx))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(i18n::t(lang, "translation.dropzone.title")),
                                )
                                .child(ui::hint(
                                    i18n::t(lang, "translation.dropzone.subtitle"),
                                    cx,
                                )),
                        )
                        .child(
                            Button::new("pick-files-compact")
                                .small()
                                .outline()
                                .icon(Ico::FolderOpen.icon())
                                .label(i18n::t(lang, "translation.dropzone.selectFiles"))
                                .on_click({
                                    let queue = queue.clone();
                                    move |_, _, cx| DropZone::pick_files(queue.clone(), cx)
                                }),
                        ),
                )
                .into_any_element();
        }

        container
            .px_6()
            .py_8()
            .child(
                v_flex()
                    .w_full()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .size(px(56.))
                            .rounded(px(18.))
                            .bg(ui::tint(accent, 0.12))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Ico::CloudUpload.icon().size_6().text_color(accent)),
                    )
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(i18n::t(lang, "translation.dropzone.title")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_center()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(lang, "translation.dropzone.subtitle")),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(ui::meta_chip("SRT · ASS · SSA", Some(Ico::FileText), cx))
                            .child(ui::meta_chip("MKV · MP4 · AVI", Some(Ico::Film), cx)),
                    )
                    .child(
                        Button::new("pick-files")
                            .primary()
                            .icon(Ico::FolderOpen.icon())
                            .label(i18n::t(lang, "translation.dropzone.selectFiles"))
                            .on_click(move |_, _, cx| DropZone::pick_files(queue.clone(), cx)),
                    ),
            )
            .into_any_element()
    }
}
