//! SubTranslator: subtitle translation desktop app built on GPUI.
//!
//! Everything lives in the library so integration tests can drive the core
//! pipeline; `src/main.rs` is just the entry point.

#![allow(dead_code)]

pub mod app;
pub mod assets;
pub mod core;
pub mod i18n;
pub mod icons;
pub mod state;
pub mod theme;
pub mod views;

use gpui::*;
use gpui_component::Root;

use app::RootView;
use assets::Assets;

/// Boots the window and runs the app loop.
pub fn run() {
    let app = Application::new().with_assets(Assets);
    app.run(move |cx| {
        gpui_component::init(cx);

        let dark = core::settings::load()
            .map(|settings| settings.theme != "light")
            .unwrap_or(true);
        theme::apply(dark, None, cx);

        let bounds = Bounds::centered(None, size(px(1360.), px(880.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("SubTranslator".into()),
                    ..Default::default()
                }),
                window_min_size: Some(size(px(980.), px(640.))),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| RootView::new(window, cx));
                // Entity<RootView> implements Into<AnyView>; `.into()` breaks inference.
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
