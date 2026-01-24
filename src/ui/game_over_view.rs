use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::game::{MoveRecord, PieceColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub enum GameOverMessage {
    Dismiss,
    ShowExportModal(ExportFormat),
    ExportPathChanged(String),
    CancelExport,
    ConfirmExport,
}

pub struct GameOverView {
    pub winner: PieceColor,
    pub move_history: Vec<MoveRecord>,
    pub board_size: usize,
    pub show_export_modal: bool,
    pub export_path: String,
    pub export_format: Option<ExportFormat>,
}

impl GameOverView {
    pub fn new(winner: PieceColor, move_history: Vec<MoveRecord>, board_size: usize) -> Self {
        Self {
            winner,
            move_history,
            board_size,
            show_export_modal: false,
            export_path: String::new(),
            export_format: None,
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
            board_size: usize,
            winner: String,
            total_moves: usize,
            moves: &'a Vec<MoveRecord>,
        }

        let log = GameLog {
            board_size: self.board_size,
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
            .on_press(GameOverMessage::ShowExportModal(ExportFormat::Text));
        let json_button = button(text("JSON").size(16))
            .padding(10)
            .on_press(GameOverMessage::ShowExportModal(ExportFormat::Json));
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

        // Popup dialog box
        let popup = container(content).width(Length::Fixed(400.0)).padding(30).style(popup_style);

        // Semi-transparent backdrop
        let main_view = container(popup)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(backdrop_style);

        if self.show_export_modal {
            let modal = self.export_modal_view();
            iced::widget::stack![main_view, modal].into()
        } else {
            main_view.into()
        }
    }

    fn export_modal_view(&self) -> Element<'_, GameOverMessage> {
        let format_name = match self.export_format {
            Some(ExportFormat::Text) => "Text",
            Some(ExportFormat::Json) => "JSON",
            None => "File",
        };
        let title = text(format!("Export {}", format_name)).size(24);

        let default_name = match self.export_format {
            Some(ExportFormat::Text) => "konane_game.txt",
            Some(ExportFormat::Json) => "konane_game.json",
            None => "konane_game.txt",
        };
        let placeholder = format!("Enter file path (e.g., {})", default_name);

        let path_input = text_input(&placeholder, &self.export_path)
            .on_input(GameOverMessage::ExportPathChanged)
            .on_submit(GameOverMessage::ConfirmExport)
            .padding(10)
            .width(Length::Fixed(300.0));

        let export_btn = button(text("Save").size(16))
            .padding(10)
            .on_press(GameOverMessage::ConfirmExport);

        let cancel_btn = button(text("Cancel").size(16))
            .padding(10)
            .on_press(GameOverMessage::CancelExport);

        let buttons = row![cancel_btn, export_btn].spacing(10);

        let modal_content = column![title, path_input, buttons].spacing(15).align_x(Alignment::Center);

        let inner_popup = container(modal_content)
            .width(Length::Fixed(400.0))
            .padding(30)
            .style(popup_style);

        container(inner_popup)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(export_backdrop_style)
            .into()
    }
}

fn export_backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
        ..Default::default()
    }
}

// Semi-transparent dark backdrop
fn backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
        ..Default::default()
    }
}

// Popup dialog style with shadow
fn popup_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 2.0,
            radius: 8.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    }
}
