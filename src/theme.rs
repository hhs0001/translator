//! App theming: gpui-component's palette plus our own brand tweaks.

use gpui::{hsla, px, App, Window};
use gpui_component::{Theme, ThemeMode};

/// Brand accent (indigo/violet), used for primary actions and highlights.
fn accent(dark: bool) -> gpui::Hsla {
    if dark {
        hsla(252. / 360., 0.83, 0.67, 1.)
    } else {
        hsla(252. / 360., 0.72, 0.53, 1.)
    }
}

/// Switches theme mode and applies the app's palette on top of it.
pub fn apply(dark: bool, window: Option<&mut Window>, cx: &mut App) {
    let mode = if dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    Theme::change(mode, window, cx);

    let accent_color = accent(dark);
    let theme = Theme::global_mut(cx);

    theme.radius = px(10.);
    theme.radius_lg = px(16.);

    theme.colors.primary = accent_color;
    theme.colors.primary_hover = accent_color.opacity(0.9);
    theme.colors.primary_active = accent_color.opacity(0.8);
    theme.colors.primary_foreground = hsla(0., 0., 1., 1.);
    theme.colors.ring = accent_color;
    theme.colors.progress_bar = accent_color;
    theme.colors.selection = accent_color.opacity(0.28);
    theme.colors.link = accent_color;
    theme.colors.link_hover = accent_color.opacity(0.85);

    if dark {
        // Slightly deeper, bluer neutrals than the stock dark theme.
        theme.colors.background = hsla(232. / 360., 0.16, 0.08, 1.);
        theme.colors.secondary = hsla(232. / 360., 0.14, 0.13, 1.);
        theme.colors.secondary_hover = hsla(232. / 360., 0.14, 0.16, 1.);
        theme.colors.muted = hsla(232. / 360., 0.12, 0.17, 1.);
        theme.colors.muted_foreground = hsla(230. / 360., 0.12, 0.68, 1.);
        theme.colors.border = hsla(232. / 360., 0.13, 0.22, 1.);
        theme.colors.input = hsla(232. / 360., 0.13, 0.22, 1.);
        theme.colors.popover = hsla(232. / 360., 0.15, 0.11, 1.);
        theme.colors.title_bar = hsla(232. / 360., 0.15, 0.10, 1.);
        theme.colors.list_hover = hsla(232. / 360., 0.14, 0.16, 1.);
    }
}
