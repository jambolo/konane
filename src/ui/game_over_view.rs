use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::game::{MoveRecord, PieceColor};

#[derive(Debug, Clone)]
pub enum GameOverMessage {
    Dismiss,
    DownloadText,
    DownloadJson,
}

pub struct GameOverView {
    pub winner: PieceColor,
    pub move_history: Vec<MoveRecord>,
}

impl GameOverView {
    pub fn new(winner: PieceColor, move_history: Vec<MoveRecord>) -> Self {
        Self {
            winner,
            move_history,
        }
    }

    pub fn generate_text_log(&self) -> String {
        let mut log = String::new();

        for (i, move_record) in self.move_history.iter().enumerate() {
            log.push_str(&format!("{}. {}\n", i + 1, move_record.to_algebraic()));
        }

        let result_code = match self.winner {
            PieceColor::Black => "1-0",
            PieceColor::White => "0-1",
        };
        log.push_str(result_code);
        log.push('\n');

        log
    }

    pub fn generate_json_log(&self) -> String {
        #[derive(serde::Serialize)]
        struct GameLog<'a> {
            winner: String,
            total_moves: usize,
            moves: &'a Vec<MoveRecord>,
        }

        let log = GameLog {
            winner: self.winner.to_string(),
            total_moves: self.move_history.len(),
            moves: &self.move_history,
        };

        serde_json::to_string_pretty(&log).unwrap_or_else(|_| "Error generating JSON".to_string())
    }

    pub fn view(&self) -> Element<'_, GameOverMessage> {
        let title = text("Game Over!").size(36);

        let winner_text = text(format!("{} wins!", self.winner)).size(28);

        let moves_text = text(format!("Total moves: {}", self.move_history.len())).size(18);

        let download_label = text("Download game log:").size(16);

        let text_button = button(text("Text").size(16))
            .padding(10)
            .on_press(GameOverMessage::DownloadText);

        let json_button = button(text("JSON").size(16))
            .padding(10)
            .on_press(GameOverMessage::DownloadJson);

        let download_row = row![download_label, text_button, json_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let dismiss_button = button(text("New Game").size(18))
            .padding(15)
            .on_press(GameOverMessage::Dismiss);

        let content = column![
            title,
            text("").height(Length::Fixed(20.0)),
            winner_text,
            moves_text,
            text("").height(Length::Fixed(30.0)),
            download_row,
            text("").height(Length::Fixed(20.0)),
            dismiss_button,
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fixed(400.0))
            .height(Length::Fixed(350.0))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(30)
            .style(container::bordered_box)
            .into()
    }
}
