//! Shared visual primitives: every screen is built out of these so the app
//! reads as one design system instead of a pile of ad-hoc divs.

use gpui::prelude::*;
use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, Sizable};

use crate::icons::Ico;

/// Tints a color for use as a soft background behind an icon or chip.
pub fn tint(color: Hsla, alpha: f32) -> Hsla {
    color.opacity(alpha)
}

/// The main surface used for every panel: subtle border, soft fill.
pub fn card(cx: &App) -> Div {
    v_flex()
        .w_full()
        .gap_4()
        .p_5()
        .rounded(px(16.))
        .border_1()
        .border_color(cx.theme().border.opacity(0.8))
        .bg(cx.theme().secondary.opacity(0.35))
}

/// Card heading with a tinted icon badge, title and optional description.
pub fn card_header(
    icon: Ico,
    accent: Hsla,
    title: impl Into<SharedString>,
    description: Option<String>,
    cx: &App,
) -> impl IntoElement {
    h_flex()
        .flex_1()
        .min_w_0()
        .gap_3()
        .items_start()
        .child(icon_badge(icon, accent, cx))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title.into()),
                )
                .when_some(description, |this, description| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(description),
                    )
                }),
        )
}

/// Rounded square holding an icon, tinted with the given accent.
pub fn icon_badge(icon: Ico, accent: Hsla, _cx: &App) -> impl IntoElement {
    div()
        .flex_none()
        .size(px(32.))
        .rounded(px(10.))
        .bg(tint(accent, 0.14))
        .flex()
        .items_center()
        .justify_center()
        .child(icon.icon().text_color(accent).small())
}

/// Tiny uppercase label used above groups of controls.
pub fn overline(label: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().muted_foreground.opacity(0.85))
        .child(label.into())
}

/// Secondary explanatory text below a control.
pub fn hint(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    div()
        .text_xs()
        .whitespace_normal()
        .text_color(cx.theme().muted_foreground.opacity(0.9))
        .child(text.into())
}

/// Label + control + optional hint, the standard settings row.
pub fn field(
    label: impl Into<SharedString>,
    hint_text: Option<String>,
    control: impl IntoElement,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .w_full()
        .gap(px(6.))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(label.into()),
        )
        .child(control)
        .when_some(hint_text, |this, text| this.child(hint(text, cx)))
}

/// Colored pill with optional leading icon — status, counters, metadata.
pub fn chip(
    label: impl Into<SharedString>,
    color: Hsla,
    icon: Option<Ico>,
    cx: &App,
) -> impl IntoElement {
    let _ = cx;
    h_flex()
        .flex_none()
        .gap_1p5()
        .items_center()
        .px(px(8.))
        .py(px(3.))
        .rounded_full()
        .bg(tint(color, 0.14))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(color)
        .when_some(icon, |this, icon| {
            this.child(icon.icon().text_color(color).xsmall())
        })
        .child(div().child(label.into()))
}

/// Neutral metadata pill (format, line counts, timings).
pub fn meta_chip(label: impl Into<SharedString>, icon: Option<Ico>, cx: &App) -> impl IntoElement {
    chip(label, cx.theme().muted_foreground, icon, cx)
}

/// Monospaced inline text, for timecodes and paths.
pub fn mono(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .font_family(cx.theme().mono_font_family.clone())
        .text_xs()
        .child(text.into())
}

/// Thin horizontal rule.
pub fn rule(cx: &App) -> Div {
    div().w_full().h(px(1.)).bg(cx.theme().border.opacity(0.6))
}

/// Large placeholder shown when there is nothing to display yet.
pub fn empty_state(
    icon: Ico,
    title: impl Into<SharedString>,
    description: Option<String>,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .size_full()
        .min_h(px(240.))
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .size(px(56.))
                .rounded(px(18.))
                .bg(cx.theme().muted.opacity(0.5))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    icon.icon()
                        .size_6()
                        .text_color(cx.theme().muted_foreground.opacity(0.8)),
                ),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(title.into()),
        )
        .when_some(description, |this, description| {
            this.child(
                div()
                    .max_w(px(320.))
                    .text_xs()
                    .text_center()
                    .text_color(cx.theme().muted_foreground)
                    .child(description),
            )
        })
}

/// Progress bar with a rounded track, used where a plain `Progress` is enough.
pub fn progress_bar(value: f32, color: Hsla, height: Pixels, cx: &App) -> impl IntoElement {
    let value = value.clamp(0., 100.);
    div()
        .w_full()
        .h(height)
        .rounded_full()
        .bg(cx.theme().muted.opacity(0.7))
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(value / 100.))
                .rounded_full()
                .bg(color),
        )
}

/// A key/value line used in headers ("42 / 900 lines").
pub fn stat(label: impl Into<SharedString>, value: impl Into<SharedString>, cx: &App) -> Div {
    v_flex()
        .gap(px(2.))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(label.into()),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .child(value.into()),
        )
}

/// Formats a duration for the queue/editor headers.
pub fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

/// Collapses whitespace and truncates free-form text so a long prompt can
/// never blow up the layout it is previewed in.
pub fn preview_text(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }
    let truncated: String = collapsed
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect();
    format!("{}…", truncated.trim_end())
}

/// Shortens a long path for display, keeping the file name visible.
pub fn short_path(path: &str, max: usize) -> String {
    if path.chars().count() <= max {
        return path.to_string();
    }
    let tail: String = path
        .chars()
        .rev()
        .take(max.saturating_sub(1))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

/// Applies the standard scroll container styling.
pub trait ScrollableExt: Styled + Sized {
    fn scroll_pane(self) -> Self {
        self.min_h_0().flex_1()
    }
}

impl<T: Styled> ScrollableExt for T {}

/// Soft elevation used by floating surfaces (editor, queue cards).
pub fn elevate(el: Div, cx: &App) -> Div {
    let _ = cx;
    el.shadow_sm()
}
