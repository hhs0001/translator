use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::progress::Progress;
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::i18n::{self, Language};
use crate::state::queue::{FileKind, QueueFile, QueueState};
use crate::state::settings::SettingsState;

#[derive(IntoElement)]
pub struct QueueList {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    compact: bool,
    show_cancel_all: bool,
}

impl QueueList {
    pub fn new(
        queue: Entity<QueueState>,
        settings: Entity<SettingsState>,
        compact: bool,
        show_cancel_all: bool,
    ) -> Self {
        Self {
            queue,
            settings,
            compact,
            show_cancel_all,
        }
    }
}

impl RenderOnce for QueueList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let lang = self.settings.read(cx).language();
        let state = self.queue.read(cx);
        let files: Vec<QueueFile> = if self.compact {
            state.files.iter().take(1).cloned().collect()
        } else {
            state.files.clone()
        };
        let current = state.current_file_id.clone();
        let extra = if self.compact {
            state.files.len().saturating_sub(1)
        } else {
            0
        };

        v_flex()
            .w_full()
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .items_center()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(format!(
                        "{} · {}",
                        i18n::t(lang, "translation.queue.title"),
                        state.files.len()
                    )))
                    .child(
                        h_flex()
                            .gap_2()
                            .when(self.show_cancel_all && state.is_translating, |this| {
                                let queue = self.queue.clone();
                                this.child(
                                    Button::new("cancel-all")
                                        .danger()
                                        .small()
                                        .label(i18n::t(lang, "translation.queue.cancelAll"))
                                        .on_click(move |_, _, cx| {
                                            queue.update(cx, |q, cx| q.cancel_all(cx));
                                        }),
                                )
                            })
                            .child({
                                let queue = self.queue.clone();
                                Button::new("clear-queue")
                                    .ghost()
                                    .small()
                                    .label(i18n::t(lang, "common.clear"))
                                    .on_click(move |_, _, cx| {
                                        queue.update(cx, |q, cx| q.clear(cx));
                                    })
                            }),
                    ),
            )
            .children(
                files
                    .into_iter()
                    .map(|file| queue_item(file, current.clone(), lang, self.queue.clone(), cx)),
            )
            .when(extra > 0, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(i18n::tf(
                            lang,
                            "translation.queue.moreFiles",
                            &[("count", &extra.to_string())],
                        )),
                )
            })
    }
}

fn queue_item(
    file: QueueFile,
    current: Option<String>,
    lang: Language,
    queue: Entity<QueueState>,
    cx: &App,
) -> impl IntoElement {
    let selected = current.as_deref() == Some(file.id.as_str());
    let id = file.id.clone();
    let id_select = file.id.clone();
    let id_remove = file.id.clone();
    let id_cancel = file.id.clone();
    let queue_select = queue.clone();
    let queue_remove = queue.clone();
    let queue_cancel = queue.clone();
    let tracks = file.subtitle_tracks.clone();
    let selected_track = file.selected_track_index;

    v_flex()
        .id(SharedString::from(format!("qf-{}", file.id)))
        .w_full()
        .gap_1()
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(if selected {
            cx.theme().primary
        } else {
            cx.theme().border
        })
        .bg(if selected {
            cx.theme().accent.opacity(0.2)
        } else {
            cx.theme().background
        })
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            queue_select.update(cx, |q, cx| q.select_file(id_select.clone(), cx));
        })
        .child(
            h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .child(file.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} · {}/{} {}",
                                    file.status.label(lang),
                                    file.translated_lines,
                                    file.total_lines,
                                    i18n::t(lang, "common.lines")
                                )),
                        ),
                )
                .child(
                    h_flex()
                        .gap_1()
                        .when(file.status.is_processing(), |this| {
                            this.child(
                                Button::new(SharedString::from(format!("cancel-{id}")))
                                    .ghost()
                                    .small()
                                    .label(i18n::t(lang, "common.cancel"))
                                    .on_click(move |_, _, cx| {
                                        queue_cancel
                                            .update(cx, |q, cx| q.cancel_file(&id_cancel, cx));
                                    }),
                            )
                        })
                        .child(
                            Button::new(SharedString::from(format!("rm-{id}")))
                                .ghost()
                                .small()
                                .label(i18n::t(lang, "common.delete"))
                                .on_click(move |_, _, cx| {
                                    queue_remove.update(cx, |q, cx| q.remove_file(&id_remove, cx));
                                }),
                        ),
                ),
        )
        .child(Progress::new().value(file.progress).h(px(6.)))
        .when(file.kind == FileKind::Video, |this| {
            this.child(track_picker(
                lang,
                file.id.clone(),
                tracks,
                selected_track,
                file.is_loading_tracks,
                queue,
                cx,
            ))
        })
        .when_some(file.error.clone(), |this, err| {
            this.child(div().text_xs().text_color(cx.theme().danger).child(err))
        })
}

fn track_picker(
    lang: Language,
    file_id: String,
    tracks: Vec<crate::core::ffmpeg::SubtitleTrack>,
    selected: Option<usize>,
    loading: bool,
    queue: Entity<QueueState>,
    cx: &App,
) -> impl IntoElement {
    if loading {
        return div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(i18n::t(lang, "translation.video.detectingTracks"))
            .into_any_element();
    }
    if tracks.is_empty() {
        return div()
            .text_xs()
            .text_color(cx.theme().warning)
            .child(i18n::t(lang, "translation.video.noTracksFound"))
            .into_any_element();
    }

    h_flex()
        .gap_1()
        .flex_wrap()
        .children(tracks.into_iter().map(|track| {
            let idx = track.index;
            let file_id = file_id.clone();
            let queue = queue.clone();
            let active = selected == Some(idx);
            let label = track
                .title
                .clone()
                .or(track.language.clone())
                .unwrap_or_else(|| {
                    i18n::tf(
                        lang,
                        "translation.video.track",
                        &[("index", &idx.to_string())],
                    )
                });
            let mut btn = Button::new(SharedString::from(format!("trk-{file_id}-{idx}")))
                .small()
                .label(label)
                .on_click(move |_, _, cx| {
                    queue.update(cx, |q, cx| q.set_selected_track(&file_id, idx, cx));
                });
            if active {
                btn = btn.primary();
            } else {
                btn = btn.ghost();
            }
            btn
        }))
        .into_any_element()
}
