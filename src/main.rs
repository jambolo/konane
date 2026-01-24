mod ui;

use clap::Parser;
use konane::game;

use iced::window;
use ui::KonaneApp;

#[derive(Parser)]
#[command(name = "konane")]
#[command(about = "Kōnane - Traditional Hawaiian Strategy Board Game")]
struct Args {
    /// AI search depth (default: 8)
    #[arg(long, default_value_t = 8)]
    ai_depth: i32,
}

fn main() -> iced::Result {
    let args = Args::parse();

    iced::application(
        move || KonaneApp::new(args.ai_depth),
        KonaneApp::update,
        KonaneApp::view,
    )
    .title(KonaneApp::title)
    .subscription(KonaneApp::subscription)
    .window(window::Settings {
        size: iced::Size::new(800.0, 700.0),
        min_size: Some(iced::Size::new(600.0, 500.0)),
        ..Default::default()
    })
    .run()
}
