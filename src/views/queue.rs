use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, Sizable};

use crate::core::translator::BatchStatus;
use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::queue::{BatchProgress, FileKind, FileStatus, FileSummary, QueueState};
use crate::state::settings::SettingsState;
use crate::views::ui;

#[derive(IntoElement)]
pub struct QueueList {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    /// Single-file mode only shows the selected file.
    compact: bool,
    show_bulk_actions: bool,
}

impl QueueList {
    pub fn new(
        queue: Entity<QueueState>,
        settings: Entity<SettingsState>,
        compact: bool,
        show_bulk_actions: bool,
    ) -> Self {
        Self {
            queue,
            settings,
            compact,
            show_bulk_actions,
        }
    }
}

/// Color that represents a status across the whole UI.
pub fn status_color(status: FileStatus, cx: &App) -> Hsla {
    match status {
        FileStatus::Completed => cx.theme().success,
        FileStatus::Error => cx.theme().danger,
        FileStatus::Cancelled | FileStatus::Paused => cx.theme().warning,
        FileStatus::Pending => cx.theme().muted_foreground,
        _ => cx.theme().primary,
    }
}

pub fn status_icon(status: FileStatus) -> Ico {
    match status {
        FileStatus::Completed => Ico::CircleCheck,
        FileStatus::Error => Ico::TriangleAlert,
        FileStatus::Cancelled => Ico::Ban,
        FileStatus::Paused => Ico::Pause,
        FileStatus::Pending => Ico::Clock,
        FileStatus::Extracting => Ico::Film,
        FileStatus::Muxing => Ico::Layers,
        FileStatus::Saving => Ico::Save,
        FileStatus::DetectingLanguage => Ico::Globe,
        FileStatus::Translating => Ico::Languages,
    }
}

impl RenderOnce for QueueList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let lang = self.settings.read(cx).language();
        let state = self.queue.read(cx);
        let current = state.current_file_id.clone();
        let total = state.files.len();
        let completed = state
            .files
            .iter()
            .filter(|f| f.status == FileStatus::Completed)
            .count();
        let failed = state
            .files
            .iter()
            .filter(|f| f.status == FileStatus::Error)
            .count();
        let running = state.processing_count();
        let is_translating = state.is_translating || running > 0;

        // Summaries keep the parsed subtitle out of the render path.
        let videos_with_tracks = state
            .files
            .iter()
            .filter(|f| f.kind == FileKind::Video && !f.subtitle_tracks.is_empty())
            .count();
        let current_track = state
            .current_file()
            .filter(|f| f.kind == FileKind::Video)
            .and_then(|f| f.selected_track_index)
            .unwrap_or(0);

        let files: Vec<FileSummary> = if self.compact {
            state
                .current_file()
                .map(|file| file.summary())
                .into_iter()
                .chain(
                    state
                        .files
                        .iter()
                        .filter(|f| Some(&f.id) != current.as_ref())
                        .take(2)
                        .map(|file| file.summary()),
                )
                .collect()
        } else {
            state.summaries()
        };
        let hidden = total.saturating_sub(files.len());
        let last_index = total.saturating_sub(1);

        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_between()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(i18n::t(lang, "translation.queue.title")),
                            )
                            .child(ui::meta_chip(total.to_string(), None, cx))
                            .when(running > 0, |this| {
                                this.child(ui::chip(
                                    format!("{running} {}", i18n::t(lang, "common.active")),
                                    cx.theme().primary,
                                    Some(Ico::Zap),
                                    cx,
                                ))
                            })
                            .when(completed > 0, |this| {
                                this.child(ui::chip(
                                    format!("{completed} {}", i18n::t(lang, "common.done")),
                                    cx.theme().success,
                                    Some(Ico::Check),
                                    cx,
                                ))
                            })
                            .when(failed > 0, |this| {
                                this.child(ui::chip(
                                    failed.to_string(),
                                    cx.theme().danger,
                                    Some(Ico::TriangleAlert),
                                    cx,
                                ))
                            }),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .when(
                                self.show_bulk_actions && videos_with_tracks > 1 && !is_translating,
                                |this| {
                                    let queue = self.queue.clone();
                                    this.child(
                                        Button::new("track-for-all")
                                            .ghost()
                                            .xsmall()
                                            .icon(Ico::Film.icon())
                                            .label(i18n::tf(
                                                lang,
                                                "translation.queue.trackForAll",
                                                &[("index", &(current_track + 1).to_string())],
                                            ))
                                            .tooltip(i18n::t(
                                                lang,
                                                "translation.queue.applyTrackAll",
                                            ))
                                            .on_click(move |_, _, cx| {
                                                queue.update(cx, |q, cx| {
                                                    q.set_all_video_tracks(current_track, cx)
                                                });
                                            }),
                                    )
                                },
                            )
                            .when(self.show_bulk_actions && is_translating, |this| {
                                let queue = self.queue.clone();
                                this.child(
                                    Button::new("cancel-all")
                                        .danger()
                                        .xsmall()
                                        .icon(Ico::Ban.icon())
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
                                    .xsmall()
                                    .icon(Ico::Trash.icon())
                                    .label(i18n::t(lang, "common.clear"))
                                    .disabled(total == 0)
                                    .on_click(move |_, _, cx| {
                                        queue.update(cx, |q, cx| q.clear(cx));
                                    })
                            }),
                    ),
            )
            .children(files.into_iter().enumerate().map(|(position, file)| {
                queue_item(
                    file,
                    position,
                    last_index,
                    current.clone(),
                    lang,
                    self.queue.clone(),
                    self.compact,
                    cx,
                )
            }))
            .when(hidden > 0, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(i18n::tf(
                            lang,
                            "translation.queue.moreFiles",
                            &[("count", &hidden.to_string())],
                        )),
                )
            })
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_item(
    file: FileSummary,
    index: usize,
    last_index: usize,
    current: Option<String>,
    lang: Language,
    queue: Entity<QueueState>,
    compact: bool,
    cx: &App,
) -> impl IntoElement {
    let selected = current.as_deref() == Some(file.id.as_str());
    let status = file.status;
    let accent = status_color(status, cx);
    let id = file.id.clone();
    let processing = status.is_processing();

    let meta = h_flex()
        .gap_1p5()
        .items_center()
        .flex_wrap()
        .child(ui::chip(
            status.label(lang),
            accent,
            Some(status_icon(status)),
            cx,
        ))
        .when_some(file.format_label, |this, format| {
            this.child(ui::meta_chip(format, None, cx))
        })
        .when(file.total_lines > 0, |this| {
            this.child(ui::meta_chip(
                format!(
                    "{}/{} {}",
                    file.translated_lines,
                    file.total_lines,
                    i18n::t(lang, "common.lines")
                ),
                Some(Ico::List),
                cx,
            ))
        })
        .when_some(file.eta_secs, |this, eta| {
            this.child(ui::meta_chip(
                format!("~{}", ui::format_duration(eta)),
                Some(Ico::Clock),
                cx,
            ))
        })
        .when(
            status == FileStatus::Completed && file.elapsed_secs.is_some(),
            |this| {
                this.child(ui::meta_chip(
                    ui::format_duration(file.elapsed_secs.unwrap_or(0)),
                    Some(Ico::Clock),
                    cx,
                ))
            },
        )
        .when_some(file.detected_language.clone(), |this, detected| {
            this.child(ui::meta_chip(detected.display_name, Some(Ico::Globe), cx))
        });

    v_flex()
        .id(SharedString::from(format!("qf-{}", file.id)))
        .w_full()
        .gap_3()
        .p_3()
        .rounded(px(14.))
        .border_1()
        .border_color(if selected {
            cx.theme().primary.opacity(0.55)
        } else {
            cx.theme().border.opacity(0.7)
        })
        .bg(if selected {
            ui::tint(cx.theme().primary, 0.08)
        } else {
            cx.theme().secondary.opacity(0.4)
        })
        .hover(|this| {
            this.bg(if selected {
                ui::tint(cx.theme().primary, 0.12)
            } else {
                cx.theme().secondary.opacity(0.6)
            })
        })
        .cursor_pointer()
        .on_click({
            let queue = queue.clone();
            let id = id.clone();
            move |_, _, cx| {
                queue.update(cx, |q, cx| q.select_file(id.clone(), cx));
            }
        })
        .child(
            h_flex()
                .w_full()
                .gap_3()
                .items_start()
                .child(
                    div()
                        .flex_none()
                        .size(px(34.))
                        .rounded(px(10.))
                        .bg(ui::tint(accent, 0.14))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            if file.kind == FileKind::Video {
                                Ico::Film
                            } else {
                                Ico::FileText
                            }
                            .icon()
                            .small()
                            .text_color(accent),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1p5()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .overflow_hidden()
                                .child(file.name.clone()),
                        )
                        .child(meta),
                )
                .child(
                    h_flex()
                        .flex_none()
                        .gap(px(2.))
                        .items_center()
                        .when(!compact && last_index > 0, |this| {
                            let up_queue = queue.clone();
                            let down_queue = queue.clone();
                            let up_id = id.clone();
                            let down_id = id.clone();
                            this.child(
                                Button::new(SharedString::from(format!("up-{id}")))
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::ArrowUp.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.moveUp"))
                                    .disabled(index == 0)
                                    .on_click(move |_, _, cx| {
                                        up_queue.update(cx, |q, cx| q.move_file(&up_id, -1, cx));
                                    }),
                            )
                            .child(
                                Button::new(SharedString::from(format!("down-{id}")))
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::ArrowDown.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.moveDown"))
                                    .disabled(index >= last_index)
                                    .on_click(move |_, _, cx| {
                                        down_queue.update(cx, |q, cx| q.move_file(&down_id, 1, cx));
                                    }),
                            )
                        })
                        .when(file.can_retry, |this| {
                            let queue = queue.clone();
                            let id = id.clone();
                            this.child(
                                Button::new(SharedString::from(format!("retry-{}", file.id)))
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::Refresh.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.retry"))
                                    .on_click(move |_, _, cx| {
                                        queue.update(cx, |q, cx| q.requeue_file(&id, cx));
                                    }),
                            )
                        })
                        .when(file.has_output, |this| {
                            let queue = queue.clone();
                            let id = id.clone();
                            this.child(
                                Button::new(SharedString::from(format!("open-{}", file.id)))
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::FolderOpen.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.revealOutput"))
                                    .on_click(move |_, _, cx| {
                                        queue.update(cx, |q, cx| q.reveal_output(&id, cx));
                                    }),
                            )
                        })
                        .when(processing, |this| {
                            let queue = queue.clone();
                            let id = id.clone();
                            this.child(
                                Button::new(SharedString::from(format!("cancel-{}", file.id)))
                                    .ghost()
                                    .xsmall()
                                    .icon(Ico::Ban.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.cancelFile"))
                                    .on_click(move |_, _, cx| {
                                        queue.update(cx, |q, cx| q.cancel_file(&id, cx));
                                    }),
                            )
                        })
                        .child({
                            let queue = queue.clone();
                            let id = id.clone();
                            Button::new(SharedString::from(format!("rm-{}", file.id)))
                                .ghost()
                                .xsmall()
                                .icon(Ico::Close.icon())
                                .tooltip(i18n::t(lang, "translation.removeFile"))
                                .disabled(processing)
                                .on_click(move |_, _, cx| {
                                    queue.update(cx, |q, cx| q.remove_file(&id, cx));
                                })
                        }),
                ),
        )
        .when(processing || status == FileStatus::Completed, |this| {
            this.child(segmented_progress(&file, lang, cx))
        })
        .when(file.kind == FileKind::Video, |this| {
            this.child(track_picker(
                lang,
                file.id.clone(),
                file.subtitle_tracks.clone(),
                file.selected_track_index,
                file.is_loading_tracks,
                processing,
                queue.clone(),
                cx,
            ))
        })
        .when_some(file.error.clone(), |this, err| {
            this.child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .items_start()
                    .p_2()
                    .rounded(px(8.))
                    .bg(ui::tint(cx.theme().danger, 0.1))
                    .child(
                        Ico::TriangleAlert
                            .icon()
                            .xsmall()
                            .text_color(cx.theme().danger),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .whitespace_normal()
                            .text_color(cx.theme().danger)
                            .child(err),
                    ),
            )
        })
}

/// Progress bar split per translation batch, so parallel requests are visible.
pub fn segmented_progress(file: &FileSummary, lang: Language, cx: &App) -> impl IntoElement {
    let percent = file.progress.clamp(0., 100.);
    let batches: &[BatchProgress] = &file.batches;
    let active = batches
        .iter()
        .filter(|b| b.status == BatchStatus::Active)
        .count();
    let done = batches
        .iter()
        .filter(|b| b.status == BatchStatus::Completed)
        .count();

    let header = h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{}/{} {}",
                            file.translated_lines,
                            file.total_lines,
                            i18n::t(lang, "common.lines")
                        )),
                )
                .when(batches.len() > 1, |this| {
                    this.child(ui::meta_chip(
                        format!(
                            "{done}/{} {}",
                            batches.len(),
                            i18n::t(lang, "common.batches")
                        ),
                        Some(Ico::Layers),
                        cx,
                    ))
                })
                .when(active > 0, |this| {
                    this.child(ui::chip(
                        format!("{active} {}", i18n::t(lang, "common.active")),
                        cx.theme().primary,
                        Some(Ico::Zap),
                        cx,
                    ))
                }),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .child(format!("{}%", percent.round() as i32)),
        );

    let bar = if batches.len() > 1 {
        h_flex()
            .w_full()
            .gap(px(2.))
            .h(px(8.))
            .children(batches.iter().map(|batch| {
                let ratio = if batch.total > 0 {
                    (batch.done as f32 / batch.total as f32).clamp(0., 1.)
                } else {
                    0.
                };
                let (track, fill) = match batch.status {
                    BatchStatus::Completed => {
                        (ui::tint(cx.theme().success, 0.25), Some(cx.theme().success))
                    }
                    BatchStatus::Error => {
                        (ui::tint(cx.theme().danger, 0.25), Some(cx.theme().danger))
                    }
                    BatchStatus::Active => {
                        (ui::tint(cx.theme().primary, 0.2), Some(cx.theme().primary))
                    }
                    BatchStatus::Pending => (cx.theme().muted.opacity(0.8), None),
                };
                div()
                    .flex_1()
                    .h_full()
                    .rounded(px(3.))
                    .bg(track)
                    .overflow_hidden()
                    .when_some(fill, |this, color| {
                        this.child(
                            div()
                                .h_full()
                                .w(relative(if batch.status == BatchStatus::Completed {
                                    1.
                                } else {
                                    ratio.max(0.06)
                                }))
                                .rounded(px(3.))
                                .bg(color),
                        )
                    })
            }))
            .into_any_element()
    } else {
        ui::progress_bar(percent, status_color(file.status, cx), px(6.), cx).into_any_element()
    };

    v_flex().w_full().gap_1p5().child(header).child(bar)
}

#[allow(clippy::too_many_arguments)]
fn track_picker(
    lang: Language,
    file_id: String,
    tracks: Vec<crate::core::ffmpeg::SubtitleTrack>,
    selected: Option<usize>,
    loading: bool,
    disabled: bool,
    queue: Entity<QueueState>,
    cx: &App,
) -> AnyElement {
    if loading {
        return h_flex()
            .gap_2()
            .items_center()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(Ico::Loader.icon().xsmall())
            .child(i18n::t(lang, "translation.video.detectingTracks"))
            .into_any_element();
    }
    if tracks.is_empty() {
        return h_flex()
            .gap_2()
            .items_center()
            .text_xs()
            .text_color(cx.theme().warning)
            .child(Ico::TriangleAlert.icon().xsmall())
            .child(i18n::t(lang, "translation.video.noTracksFound"))
            .into_any_element();
    }

    v_flex()
        .w_full()
        .gap_1p5()
        .child(ui::overline(
            i18n::t(lang, "translation.video.subtitleTrack"),
            cx,
        ))
        .child(
            h_flex()
                .gap_1()
                .flex_wrap()
                .children(tracks.into_iter().map(|track| {
                    let idx = track.index;
                    let file_id = file_id.clone();
                    let queue = queue.clone();
                    let active = selected == Some(idx);
                    let mut label = track
                        .title
                        .clone()
                        .or_else(|| track.language.clone())
                        .unwrap_or_else(|| {
                            i18n::tf(
                                lang,
                                "translation.video.track",
                                &[("index", &(idx + 1).to_string())],
                            )
                        });
                    if let (Some(language), Some(_)) = (&track.language, &track.title) {
                        label = format!("{label} · {language}");
                    }
                    let label = format!("{label} · {}", track.codec_name);

                    let mut btn = Button::new(SharedString::from(format!("trk-{file_id}-{idx}")))
                        .xsmall()
                        .label(label)
                        .disabled(disabled)
                        .on_click(move |_, _, cx| {
                            queue.update(cx, |q, cx| q.set_selected_track(&file_id, idx, cx));
                        });
                    btn = if active { btn.primary() } else { btn.outline() };
                    btn
                })),
        )
        .into_any_element()
}
