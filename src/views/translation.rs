use gpui::prelude::*;
use gpui::*;
use gpui_component::tab::{Tab, TabBar};
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::i18n;
use crate::state::queue::QueueState;
use crate::state::settings::SettingsState;
use crate::views::dropzone::DropZone;
use crate::views::editor::{empty_editor, SubtitleEditor};
use crate::views::queue::QueueList;

pub struct TranslationView {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    batch: bool,
    _subs: Vec<Subscription>,
}

impl TranslationView {
    pub fn new(
        queue: Entity<QueueState>,
        settings: Entity<SettingsState>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subs = Vec::new();
        subs.push(cx.observe(&queue, |_, _, cx| cx.notify()));
        subs.push(cx.observe(&settings, |_, _, cx| cx.notify()));
        Self {
            queue,
            settings,
            batch: false,
            _subs: subs,
        }
    }
}

impl Render for TranslationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.settings.read(cx).language();
        let has_files = !self.queue.read(cx).files.is_empty();
        let current = self.queue.read(cx).current_file().cloned();
        let selected = if self.batch { 1 } else { 0 };

        v_flex()
            .size_full()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(i18n::t(lang, "translation.title")),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(lang, "translation.subtitle")),
                    ),
            )
            .child(
                TabBar::new("translation-mode")
                    .selected_index(selected)
                    .child(Tab::new().label(i18n::t(lang, "translation.mode.single")))
                    .child(Tab::new().label(i18n::t(lang, "translation.mode.batch")))
                    .on_click(cx.listener(|this, ix, _, cx| {
                        this.batch = *ix == 1;
                        cx.notify();
                    })),
            )
            .child(if self.batch {
                v_flex()
                    .size_full()
                    .gap_4()
                    .child(DropZone::new(self.queue.clone(), self.settings.clone()))
                    .when(has_files, |this| {
                        this.child(QueueList::new(
                            self.queue.clone(),
                            self.settings.clone(),
                            false,
                            true,
                        ))
                    })
                    .into_any_element()
            } else {
                h_flex()
                    .size_full()
                    .gap_4()
                    .items_start()
                    .child(
                        v_flex()
                            .w(relative(0.42))
                            .gap_4()
                            .child(DropZone::new(self.queue.clone(), self.settings.clone()))
                            .when(has_files, |this| {
                                this.child(QueueList::new(
                                    self.queue.clone(),
                                    self.settings.clone(),
                                    true,
                                    false,
                                ))
                            }),
                    )
                    .child(div().flex_1().h_full().child(
                        if let Some(file) = current.filter(|f| f.original.is_some()) {
                            SubtitleEditor::new(file, lang).into_any_element()
                        } else {
                            empty_editor(lang, cx).into_any_element()
                        },
                    ))
                    .into_any_element()
            })
    }
}
