use iced::widget::{button, column, container, pick_list, radio, row, text};
use iced::{Alignment, Element, Length};
use rand::Rng;

use crate::game::PieceColor;

#[derive(Debug, Clone)]
pub enum SetupMessage {
    BoardSizeSelected(usize),
    ColorOptionSelected(ColorOption),
    StartGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOption {
    Black,
    White,
    Random,
}

impl std::fmt::Display for ColorOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorOption::Black => write!(f, "Black"),
            ColorOption::White => write!(f, "White"),
            ColorOption::Random => write!(f, "Random"),
        }
    }
}

impl ColorOption {
    pub fn to_piece_color(self) -> PieceColor {
        match self {
            ColorOption::Black => PieceColor::Black,
            ColorOption::White => PieceColor::White,
            ColorOption::Random => {
                if rand::rng().random_bool(0.5) {
                    PieceColor::Black
                } else {
                    PieceColor::White
                }
            }
        }
    }
}

pub struct SetupView {
    pub board_size: usize,
    pub color_option: ColorOption,
}

impl Default for SetupView {
    fn default() -> Self {
        Self {
            board_size: 8,
            color_option: ColorOption::Black,
        }
    }
}

impl SetupView {
    pub fn view(&self) -> Element<'_, SetupMessage> {
        let title = text("Kōnane").size(48);

        let subtitle = text("Hawaiian Checkers").size(24);

        // Board size selector
        let board_sizes: Vec<usize> = (4..=16).step_by(2).collect();
        let size_label = text("Board Size:").size(18);
        let size_picker = pick_list(
            board_sizes,
            Some(self.board_size),
            SetupMessage::BoardSizeSelected,
        )
        .width(Length::Fixed(80.0));

        let size_row = row![size_label, size_picker]
            .spacing(10)
            .align_y(Alignment::Center);

        // Player 1 color selector
        let color_label = text("Player 1 Color:").size(18);
        let black_radio = radio(
            "Black (moves first)",
            ColorOption::Black,
            Some(self.color_option),
            SetupMessage::ColorOptionSelected,
        );
        let white_radio = radio(
            "White",
            ColorOption::White,
            Some(self.color_option),
            SetupMessage::ColorOptionSelected,
        );
        let random_radio = radio(
            "Random",
            ColorOption::Random,
            Some(self.color_option),
            SetupMessage::ColorOptionSelected,
        );

        let color_column = column![color_label, black_radio, white_radio, random_radio].spacing(8);

        // Start button
        let start_button = button(text("Start Game").size(20))
            .padding(15)
            .on_press(SetupMessage::StartGame);

        // Layout
        let content = column![
            title,
            subtitle,
            text("").height(Length::Fixed(30.0)),
            size_row,
            text("").height(Length::Fixed(20.0)),
            color_column,
            text("").height(Length::Fixed(30.0)),
            start_button,
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
