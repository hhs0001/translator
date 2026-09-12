use std::rc::Rc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, Sizable};

use crate::core::text_cleaner::{display_text, format_range};
use crate::i18n::{self, Language};
use crate::icons::Ico;
use crate::state::queue::{FileStatus, QueueState};
use crate::state::settings::SettingsState;
use crate::views::queue::{status_color, status_icon};
use crate::views::ui;

/// One line of the side-by-side editor.
///
/// Rows are cached between renders: the original text is formatted once per
/// file, and only lines whose translation actually changed are recomputed.
#[derive(Clone)]
struct Row {
    /// Position in the subtitle file (0-based).
    position: usize,
    /// Entry index as stored in the subtitle.
    entry_index: usize,
    time: SharedString,
    original: SharedString,
    /// Formatted translation shown to the user.
    translated: SharedString,
    /// Raw translation, used when editing and to detect changes.
    translated_raw: String,
    done: bool,
}

/// Scalars the header needs; avoids cloning the parsed subtitle per frame.
#[derive(Clone)]
struct FileHeader {
    id: String,
    name: String,
    status: FileStatus,
    progress: f32,
    total_lines: usize,
    translated_lines: usize,
    format_label: Option<&'static str>,
    has_translation: bool,
    has_unsaved_edits: bool,
    has_output: bool,
    output_path: Option<String>,
}

pub struct SubtitleEditor {
    queue: Entity<QueueState>,
    settings: Entity<SettingsState>,
    show_raw: bool,
    follow: bool,
    editing: Option<usize>,
    edit_input: Entity<InputState>,
    search: Entity<InputState>,
    query: String,

    /// All rows for the current file, in file order. Rows are shared by
    /// reference so handing them to the virtual list costs nothing per frame.
    rows: Vec<Rc<Row>>,
    /// Snapshot handed to the list closure; rebuilt only when rows change.
    snapshot: Rc<Vec<Rc<Row>>>,
    /// Indices into `rows` that match the current query.
    visible: Rc<Vec<usize>>,
    /// Identifies the data the cache was built from.
    cache_key: Option<(String, usize, bool)>,
    list: ListState,
    last_revealed: usize,
    _subs: Vec<Subscription>,
}

impl SubtitleEditor {
    pub fn new(
        queue: Entity<QueueState>,
        settings: Entity<SettingsState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let edit_input = cx.new(|cx| InputState::new(window, cx).multi_line(true).auto_grow(1, 4));
        let lang = settings.read(cx).language();
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder(i18n::t(lang, "translation.editor.search"))
        });

        let mut subs = Vec::new();
        subs.push(cx.observe(&queue, |this, _, cx| {
            this.refresh(cx);
            cx.notify();
        }));
        subs.push(cx.observe(&settings, |_, _, cx| cx.notify()));
        subs.push(
            cx.subscribe(&edit_input, |this, input, event, cx| match event {
                InputEvent::PressEnter { secondary } if !*secondary => {
                    let value = input.read(cx).value().to_string();
                    this.commit_edit(Some(value), cx);
                }
                InputEvent::Blur => {
                    let value = input.read(cx).value().to_string();
                    this.commit_edit(Some(value), cx);
                }
                _ => {}
            }),
        );
        subs.push(cx.subscribe(&search, |this, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                let query = input.read(cx).value().trim().to_lowercase();
                if query != this.query {
                    this.query = query;
                    this.rebuild_visible(true);
                    cx.notify();
                }
            }
        }));

        Self {
            queue,
            settings,
            show_raw: false,
            follow: true,
            editing: None,
            edit_input,
            search,
            query: String::new(),
            rows: Vec::new(),
            snapshot: Rc::new(Vec::new()),
            visible: Rc::new(Vec::new()),
            cache_key: None,
            list: ListState::new(0, ListAlignment::Top, px(400.)),
            last_revealed: 0,
            _subs: subs,
        }
    }

    fn header(&self, cx: &App) -> Option<FileHeader> {
        let file = self.queue.read(cx).current_file()?;
        file.original.as_ref()?;
        Some(FileHeader {
            id: file.id.clone(),
            name: file.name.clone(),
            status: file.status,
            progress: file.progress,
            total_lines: file.total_lines,
            translated_lines: file.translated_lines,
            format_label: file.format_label(),
            has_translation: file.translated_entries.is_some(),
            has_unsaved_edits: file.has_unsaved_edits,
            has_output: file.output_subtitle_path.is_some() || file.output_video_path.is_some(),
            output_path: file.output_subtitle_path.clone(),
        })
    }

    /// Rebuilds or patches the row cache from the queue state.
    ///
    /// A full rebuild only happens when the file, its length or the raw/clean
    /// toggle changes; otherwise this walks the entries and touches just the
    /// lines whose translation is new, which keeps streaming updates cheap.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let show_raw = self.show_raw;
        let state = self.queue.read(cx);
        let Some(original) = state.current_file().and_then(|file| file.original.as_ref()) else {
            if !self.rows.is_empty() {
                self.rows.clear();
                self.snapshot = Rc::new(Vec::new());
                self.cache_key = None;
                self.rebuild_visible(true);
            }
            return;
        };
        let file = state.current_file().expect("checked above");

        let key = (file.id.clone(), original.entries.len(), show_raw);
        let full_rebuild = self.cache_key.as_ref() != Some(&key);
        let translated = file.translated_entries.as_deref().unwrap_or(&[]);

        let mut changed: Vec<usize> = Vec::new();

        if full_rebuild {
            self.rows = original
                .entries
                .iter()
                .enumerate()
                .map(|(position, entry)| {
                    let translated_raw = translated
                        .get(position)
                        .filter(|t| t.index == entry.index)
                        .map(|t| t.text.clone())
                        .unwrap_or_default();
                    let done = !translated_raw.is_empty() && translated_raw != entry.text;
                    Rc::new(Row {
                        position,
                        entry_index: entry.index,
                        time: SharedString::from(format_range(&entry.start_time, &entry.end_time)),
                        original: SharedString::from(format_line(&entry.text, show_raw)),
                        translated: SharedString::from(if done {
                            format_line(&translated_raw, show_raw)
                        } else {
                            String::new()
                        }),
                        translated_raw,
                        done,
                    })
                })
                .collect();
            self.cache_key = Some(key);
        } else {
            for (position, entry) in original.entries.iter().enumerate() {
                let Some(row) = self.rows.get(position) else {
                    continue;
                };
                let new_raw = translated
                    .get(position)
                    .filter(|t| t.index == entry.index)
                    .map(|t| t.text.as_str())
                    .unwrap_or("");
                if new_raw == row.translated_raw {
                    continue;
                }
                let done = !new_raw.is_empty() && new_raw != entry.text;
                self.rows[position] = Rc::new(Row {
                    position,
                    entry_index: row.entry_index,
                    time: row.time.clone(),
                    original: row.original.clone(),
                    translated: SharedString::from(if done {
                        format_line(new_raw, show_raw)
                    } else {
                        String::new()
                    }),
                    translated_raw: new_raw.to_string(),
                    done,
                });
                changed.push(position);
            }
        }

        if full_rebuild || !changed.is_empty() {
            self.snapshot = Rc::new(self.rows.clone());
        }

        let processing = file.status.is_processing();
        let translated_lines = file.translated_lines;

        if full_rebuild || !self.query.is_empty() {
            // Filtering can change the visible set whenever a row changes.
            self.rebuild_visible(full_rebuild || !changed.is_empty());
        } else if !changed.is_empty() {
            self.patch_list(&changed);
        }

        if self.follow && processing && translated_lines > self.last_revealed {
            self.last_revealed = translated_lines;
            if let Some(target) = self
                .visible
                .iter()
                .rposition(|ix| self.rows.get(*ix).is_some_and(|row| row.done))
            {
                self.list.scroll_to_reveal_item(target);
            }
        }
        if !processing {
            self.last_revealed = 0;
        }
    }

    /// Recomputes which rows pass the search filter.
    fn rebuild_visible(&mut self, force_relayout: bool) {
        let visible: Vec<usize> = if self.query.is_empty() {
            (0..self.rows.len()).collect()
        } else {
            self.rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.original.to_lowercase().contains(&self.query)
                        || row.translated.to_lowercase().contains(&self.query)
                })
                .map(|(ix, _)| ix)
                .collect()
        };

        let len_changed = visible.len() != self.visible.len();
        self.visible = Rc::new(visible);

        if len_changed {
            self.list.reset(self.visible.len());
        } else if force_relayout {
            self.list.splice(0..self.visible.len(), self.visible.len());
        }
    }

    /// Re-measures only the rows that changed.
    fn patch_list(&self, changed: &[usize]) {
        if self.visible.len() != self.rows.len() {
            return;
        }
        // Many rows at once: one splice is cheaper than dozens.
        if changed.len() > 32 {
            self.list.splice(0..self.rows.len(), self.rows.len());
            return;
        }
        for position in changed {
            self.list.splice(*position..position + 1, 1);
        }
    }

    fn start_edit(&mut self, position: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.rows.get(position) else {
            return;
        };
        let value = if self.show_raw || !row.done {
            row.translated_raw.clone()
        } else {
            row.translated.to_string()
        };
        self.editing = Some(row.entry_index);
        self.edit_input.update(cx, |input, cx| {
            input.set_value(value, window, cx);
            input.focus(window, cx);
        });
        self.patch_list(&[position]);
        cx.notify();
    }

    fn commit_edit(&mut self, value: Option<String>, cx: &mut Context<Self>) {
        let Some(entry_index) = self.editing.take() else {
            return;
        };
        let Some(file_id) = self.queue.read(cx).current_file_id.clone() else {
            return;
        };
        if let Some(value) = value {
            self.queue.update(cx, |queue, cx| {
                queue.update_translated_line(&file_id, entry_index, value, cx);
            });
        }
        cx.notify();
    }

    fn render_header(
        &self,
        file: &FileHeader,
        lang: Language,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let accent = status_color(file.status, cx);
        let percent = file.progress.clamp(0., 100.).round() as i32;
        let id_save = file.id.clone();
        let id_reveal = file.id.clone();
        let show_raw = self.show_raw;
        let follow = self.follow;
        let processing = file.status.is_processing();

        v_flex()
            .w_full()
            .flex_none()
            .gap_3()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.8))
            .bg(cx.theme().secondary.opacity(0.45))
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .flex_none()
                            .size(px(34.))
                            .rounded(px(10.))
                            .bg(ui::tint(accent, 0.14))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(status_icon(file.status).icon().small().text_color(accent)),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .overflow_hidden()
                                    .child(file.name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_1p5()
                                    .items_center()
                                    .flex_wrap()
                                    .child(ui::chip(
                                        file.status.label(lang),
                                        accent,
                                        Some(status_icon(file.status)),
                                        cx,
                                    ))
                                    .when_some(file.format_label, |this, format| {
                                        this.child(ui::meta_chip(format, None, cx))
                                    })
                                    .child(ui::meta_chip(
                                        format!(
                                            "{}/{} {}",
                                            file.translated_lines,
                                            file.total_lines,
                                            i18n::t(lang, "translation.editor.lines")
                                        ),
                                        Some(Ico::List),
                                        cx,
                                    ))
                                    .when(file.has_unsaved_edits, |this| {
                                        this.child(ui::chip(
                                            i18n::t(lang, "translation.editor.unsaved"),
                                            cx.theme().warning,
                                            Some(Ico::Pencil),
                                            cx,
                                        ))
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .flex_none()
                            .gap_1()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(accent)
                                    .child(format!("{percent}%")),
                            )
                            .child({
                                let mut btn = Button::new("toggle-follow")
                                    .xsmall()
                                    .icon(Ico::ArrowDown.icon())
                                    .tooltip(i18n::t(lang, "translation.editor.follow"));
                                btn = if follow && processing {
                                    btn.primary()
                                } else {
                                    btn.ghost()
                                };
                                btn.on_click(cx.listener(|this, _, _, cx| {
                                    this.follow = !this.follow;
                                    cx.notify();
                                }))
                            })
                            .child({
                                let mut btn = Button::new("toggle-raw")
                                    .xsmall()
                                    .icon(Ico::Wand.icon())
                                    .tooltip(if show_raw {
                                        i18n::t(lang, "translation.editor.cleanText")
                                    } else {
                                        i18n::t(lang, "translation.editor.rawTags")
                                    });
                                btn = if show_raw { btn.primary() } else { btn.ghost() };
                                btn.on_click(cx.listener(|this, _, _, cx| {
                                    this.show_raw = !this.show_raw;
                                    this.cache_key = None;
                                    this.refresh(cx);
                                    cx.notify();
                                }))
                            })
                            .child(
                                Button::new("save-translation")
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::Save.icon())
                                    .tooltip(i18n::t(lang, "translation.editor.save"))
                                    .disabled(!file.has_translation)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let id = id_save.clone();
                                        this.queue
                                            .update(cx, |queue, cx| queue.save_translated(&id, cx));
                                    })),
                            )
                            .child(
                                Button::new("reveal-output")
                                    .xsmall()
                                    .ghost()
                                    .icon(Ico::FolderOpen.icon())
                                    .tooltip(i18n::t(lang, "translation.queue.revealOutput"))
                                    .disabled(!file.has_output)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let id = id_reveal.clone();
                                        this.queue
                                            .update(cx, |queue, cx| queue.reveal_output(&id, cx));
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_center()
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search).small().cleanable(true).prefix(
                                Ico::Search
                                    .icon()
                                    .xsmall()
                                    .text_color(cx.theme().muted_foreground),
                            ),
                        ),
                    )
                    .when(file.progress > 0., |this| {
                        this.child(div().w(px(140.)).child(ui::progress_bar(
                            file.progress,
                            accent,
                            px(6.),
                            cx,
                        )))
                    }),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .pl(px(52.))
                    .child(
                        h_flex()
                            .flex_1()
                            .gap_2()
                            .items_center()
                            .child(dot(cx.theme().muted_foreground.opacity(0.6)))
                            .child(ui::overline(
                                i18n::t(lang, "translation.editor.original"),
                                cx,
                            )),
                    )
                    .child(
                        h_flex()
                            .flex_1()
                            .gap_2()
                            .items_center()
                            .child(dot(cx.theme().primary))
                            .child(ui::overline(
                                i18n::t(lang, "translation.editor.translated"),
                                cx,
                            )),
                    ),
            )
    }
}

/// Renders a single row. Free function so the virtual list can call it during
/// layout without re-entering the view's borrow.
#[allow(clippy::too_many_arguments)]
fn render_row(
    row: &Row,
    editing: bool,
    editor: &Entity<SubtitleEditor>,
    edit_input: &Entity<InputState>,
    waiting_label: &SharedString,
    cx: &mut App,
) -> AnyElement {
    let zebra = row.position % 2 == 0;
    let position = row.position;
    let editor = editor.clone();

    h_flex()
        .id(("row", row.entry_index))
        .w_full()
        .items_start()
        .gap_3()
        .px_4()
        .py_2p5()
        .bg(if zebra {
            cx.theme().muted.opacity(0.16)
        } else {
            gpui::transparent_black()
        })
        .hover(|this| this.bg(cx.theme().muted.opacity(0.3)))
        .child(
            v_flex()
                .flex_none()
                .w(px(40.))
                .gap(px(4.))
                .items_end()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(cx.theme().muted_foreground.opacity(0.8))
                        .child(SharedString::from((row.position + 1).to_string())),
                )
                .child(dot(if row.done {
                    cx.theme().success
                } else {
                    cx.theme().muted_foreground.opacity(0.35)
                })),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    ui::mono(row.time.clone(), cx)
                        .text_color(cx.theme().muted_foreground.opacity(0.75)),
                )
                .child(
                    h_flex()
                        .w_full()
                        .items_start()
                        .gap_4()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_sm()
                                .line_height(relative(1.45))
                                .whitespace_normal()
                                .child(row.original.clone()),
                        )
                        .child(
                            div()
                                .flex_none()
                                .w(px(1.))
                                .min_h(px(18.))
                                .bg(cx.theme().border.opacity(0.7)),
                        )
                        .child(if editing {
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Input::new(edit_input).small())
                                .into_any_element()
                        } else {
                            div()
                                .id(("edit", row.entry_index))
                                .flex_1()
                                .min_w_0()
                                .px_1p5()
                                .py(px(2.))
                                .rounded(px(6.))
                                .text_sm()
                                .line_height(relative(1.45))
                                .whitespace_normal()
                                .cursor_pointer()
                                .hover(|this| this.bg(cx.theme().muted.opacity(0.6)))
                                .text_color(if row.done {
                                    cx.theme().foreground
                                } else {
                                    cx.theme().muted_foreground.opacity(0.7)
                                })
                                .child(if row.done {
                                    row.translated.clone()
                                } else {
                                    waiting_label.clone()
                                })
                                .on_click(move |_, window, cx| {
                                    editor.update(cx, |editor, cx| {
                                        editor.start_edit(position, window, cx);
                                    });
                                })
                                .into_any_element()
                        }),
                ),
        )
        .into_any_element()
}

impl Render for SubtitleEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.settings.read(cx).language();
        let Some(file) = self.header(cx) else {
            return v_flex()
                .size_full()
                .min_h_0()
                .rounded(px(16.))
                .border_1()
                .border_dashed()
                .border_color(cx.theme().border)
                .bg(cx.theme().muted.opacity(0.15))
                .child(ui::empty_state(
                    Ico::FileText,
                    i18n::t(lang, "translation.editor.empty"),
                    Some(i18n::t(lang, "translation.editor.emptyHint")),
                    cx,
                ))
                .into_any_element();
        };

        if self.cache_key.is_none() {
            self.refresh(cx);
        }

        let status = file.status;
        let output_path = file.output_path.clone();
        let header = self.render_header(&file, lang, cx);

        let visible = Rc::clone(&self.visible);
        let rows = Rc::clone(&self.snapshot);
        let editor = cx.entity();
        let edit_input = self.edit_input.clone();
        let editing = self.editing;
        let waiting_label = SharedString::from(i18n::t(lang, "translation.editor.waiting"));
        let is_empty = visible.is_empty();

        v_flex()
            .size_full()
            .min_h_0()
            .rounded(px(16.))
            .border_1()
            .border_color(cx.theme().border.opacity(0.8))
            .bg(cx.theme().background)
            .overflow_hidden()
            .child(header)
            .child(if is_empty {
                div()
                    .flex_1()
                    .min_h_0()
                    .child(ui::empty_state(
                        Ico::Search,
                        i18n::t(lang, "translation.editor.noMatches"),
                        None,
                        cx,
                    ))
                    .into_any_element()
            } else {
                list(self.list.clone(), move |ix, _window, cx| {
                    let Some(row) = visible.get(ix).and_then(|pos| rows.get(*pos)) else {
                        return div().into_any_element();
                    };
                    let row = row.as_ref();
                    let editing = editing == Some(row.entry_index);
                    render_row(row, editing, &editor, &edit_input, &waiting_label, cx)
                })
                .with_sizing_behavior(ListSizingBehavior::Auto)
                .flex_1()
                .min_h_0()
                .into_any_element()
            })
            .when(status == FileStatus::Completed, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .flex_none()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .items_center()
                        .border_t_1()
                        .border_color(cx.theme().border.opacity(0.8))
                        .bg(cx.theme().secondary.opacity(0.35))
                        .child(
                            Ico::CircleCheck
                                .icon()
                                .xsmall()
                                .text_color(cx.theme().success),
                        )
                        .when_some(output_path, |this, path| {
                            this.child(
                                ui::mono(ui::short_path(&path, 72), cx)
                                    .flex_1()
                                    .min_w_0()
                                    .text_color(cx.theme().muted_foreground),
                            )
                        }),
                )
            })
            .into_any_element()
    }
}

fn dot(color: Hsla) -> Div {
    div().flex_none().size(px(6.)).rounded_full().bg(color)
}

fn format_line(raw: &str, show_raw: bool) -> String {
    if show_raw {
        raw.trim().to_string()
    } else {
        let clean = display_text(raw);
        if clean == "—" {
            String::new()
        } else {
            clean
        }
    }
}
