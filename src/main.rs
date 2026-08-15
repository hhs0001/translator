#![allow(dead_code)]

mod app;
mod core;
mod i18n;
mod state;
mod views;

use gpui::*;
use gpui_component::Root;

use app::RootView;

fn main() {
    let app = Application::new();
    app.run(move |cx| {
        gpui_component::init(cx);
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);

        let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("SubTranslator".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| RootView::new(window, cx));
                // Entity<RootView> implements Into<AnyView>; `.into()` breaks inference.
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
    });
}
