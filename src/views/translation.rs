use gpui::prelude::*;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::queue::QueueState;
use crate::state::settings::SettingsState;
use crate::views::dropzone::DropZone;
use crate::views::editor::SubtitleEditor;
use crate::views::queue::QueueList;
use crate::views::ui;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Single,
    Batch,
}

pub struct TranslationView {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    editor: Entity<SubtitleEditor>,
    mode: Mode,
    _subs: Vec<Subscription>,
}

impl TranslationView {
    pub fn new(
        queue: Entity<QueueState>,
        settings: Entity<SettingsState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| SubtitleEditor::new(queue.clone(), settings.clone(), window, cx));

        let subs = vec![
            cx.observe(&queue, |_, _, cx| cx.notify()),
            cx.observe(&settings, |_, _, cx| cx.notify()),
        ];

        Self {
            queue,
            settings,
            editor,
            mode: Mode::Single,
            _subs: subs,
        }
    }

    fn mode_tab(
        &self,
        mode: Mode,
        icon: Ico,
        label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.mode == mode;
        let id = SharedString::from(match mode {
            Mode::Single => "mode-single",
            Mode::Batch => "mode-batch",
        });
        h_flex()
            .id(id)
            .gap_2()
            .items_center()
            .px_3()
            .py(px(6.))
            .rounded(px(8.))
            .text_sm()
            .cursor_pointer()
            .when(active, |this| {
                this.bg(cx.theme().background)
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(cx.theme().foreground)
            })
            .when(!active, |this| {
                this.text_color(cx.theme().muted_foreground)
                    .hover(|this| this.text_color(cx.theme().foreground))
            })
            .child(icon.icon().small())
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.mode = mode;
                cx.notify();
            }))
    }
}

impl Render for TranslationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang: Language = self.settings.read(cx).language();
        let has_files = !self.queue.read(cx).files.is_empty();

        v_flex()
            .size_full()
            .min_h_0()
            .gap_4()
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .justify_between()
                    .items_end()
                    .child(
                        v_flex()
                            .gap(px(2.))
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
                        h_flex()
                            .flex_none()
                            .gap_1()
                            .p(px(3.))
                            .rounded(px(10.))
                            .bg(cx.theme().muted.opacity(0.55))
                            .child(self.mode_tab(
                                Mode::Single,
                                Ico::FileText,
                                i18n::t(lang, "translation.mode.single"),
                                cx,
                            ))
                            .child(self.mode_tab(
                                Mode::Batch,
                                Ico::Layers,
                                i18n::t(lang, "translation.mode.batch"),
                                cx,
                            )),
                    ),
            )
            .child(match self.mode {
                Mode::Batch => v_flex()
                    .size_full()
                    .min_h_0()
                    .gap_4()
                    .child(
                        DropZone::new(self.queue.clone(), self.settings.clone()).compact(has_files),
                    )
                    .child(if has_files {
                        div()
                            .id("batch-queue")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .pr_1()
                            .child(QueueList::new(
                                self.queue.clone(),
                                self.settings.clone(),
                                false,
                                true,
                            ))
                            .into_any_element()
                    } else {
                        ui::empty_state(
                            Ico::Layers,
                            i18n::t(lang, "translation.queue.emptyTitle"),
                            Some(i18n::t(lang, "translation.queue.emptyHint")),
                            cx,
                        )
                        .into_any_element()
                    })
                    .into_any_element(),
                Mode::Single => h_flex()
                    .size_full()
                    .min_h_0()
                    .gap_4()
                    .child(
                        v_flex()
                            .w(px(376.))
                            .flex_none()
                            .h_full()
                            .min_h_0()
                            .gap_3()
                            .child(
                                DropZone::new(self.queue.clone(), self.settings.clone())
                                    .compact(has_files),
                            )
                            .when(has_files, |this| {
                                this.child(
                                    div()
                                        .id("single-queue")
                                        .flex_1()
                                        .min_h_0()
                                        .overflow_y_scroll()
                                        .pr_1()
                                        .child(QueueList::new(
                                            self.queue.clone(),
                                            self.settings.clone(),
                                            true,
                                            false,
                                        )),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .min_h_0()
                            .child(self.editor.clone()),
                    )
                    .into_any_element(),
            })
    }
}
