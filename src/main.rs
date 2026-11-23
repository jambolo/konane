mod ui;

use clap::Parser;
use konane::game;

use iced::window;
use ui::{KonaneApp, theme};

#[derive(Parser)]
#[command(name = "konane")]
#[command(about = "Kōnane: the Hawaiian game of lava stone and coral")]
struct Args {
    /// Search depth of the computer player, in plies (default: 8)
    #[arg(long, default_value_t = 8)]
    ai_depth: u32,
}

fn main() -> iced::Result {
    let args = Args::parse();

    let mut app = iced::application(move || KonaneApp::new(args.ai_depth), KonaneApp::update, KonaneApp::view)
        .title(KonaneApp::title)
        .theme(|_: &KonaneApp| theme::theme())
        .default_font(theme::BODY)
        .antialiasing(true)
        .window(window::Settings {
            size: iced::Size::new(860.0, 720.0),
            min_size: Some(iced::Size::new(640.0, 520.0)),
            icon: window::icon::from_file_data(include_bytes!("../data/app-icon.png"), None).ok(),
            ..Default::default()
        });
    for font in theme::FONT_FILES {
        app = app.font(font);
    }
    app.run()
}
