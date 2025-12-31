mod audio;
mod game;
mod ui;

use iced::window;
use ui::KonaneApp;

fn main() -> iced::Result {
    iced::application(KonaneApp::new, KonaneApp::update, KonaneApp::view)
        .title(KonaneApp::title)
        .subscription(KonaneApp::subscription)
        .window(window::Settings {
            size: iced::Size::new(800.0, 700.0),
            min_size: Some(iced::Size::new(600.0, 500.0)),
            ..Default::default()
        })
        .run()
}
