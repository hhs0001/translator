use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{v_flex, ActiveTheme};

use crate::i18n::{self, Language};
use crate::state::queue::QueueState;
use crate::state::settings::SettingsState;

#[derive(IntoElement)]
pub struct DropZone {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
}

impl DropZone {
    pub fn new(queue: Entity<QueueState>, settings: Entity<SettingsState>) -> Self {
        Self { queue, settings }
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
        cx.spawn(async move |cx| match rx.await {
            Ok(Ok(Some(paths))) => {
                let _ = queue.update(cx, |queue, cx| queue.add_paths(paths, cx));
            }
            _ => {}
        })
        .detach();
    }
}

impl RenderOnce for DropZone {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let lang = self.language(cx);
        let queue = self.queue.clone();

        v_flex()
            .w_full()
            .gap_3()
            .p_8()
            .rounded_xl()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted.opacity(0.3))
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(i18n::t(lang, "translation.dropzone.title")),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(i18n::t(lang, "translation.dropzone.subtitle")),
            )
            .child(
                Button::new("pick-files")
                    .primary()
                    .label(i18n::t(lang, "translation.dropzone.selectFiles"))
                    .on_click(move |_, _, cx| {
                        DropZone::pick_files(queue.clone(), cx);
                    }),
            )
    }
}
