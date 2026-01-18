use iced::widget::{button, column, container, pick_list, radio, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};
use rand::Rng;

use crate::game::PieceColor;

#[derive(Debug, Clone)]
pub enum SetupMessage {
    BoardSizeSelected(usize),
    ColorOptionSelected(ColorOption),
    StartGame,
    ImportGame,
    ShowImportModal,
    ImportPathChanged(String),
    CancelImport,
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
    pub show_import_modal: bool,
    pub import_path: String,
    pub import_error: Option<String>,
}

impl Default for SetupView {
    fn default() -> Self {
        Self {
            board_size: 8,
            color_option: ColorOption::Black,
            show_import_modal: false,
            import_path: String::new(),
            import_error: None,
        }
    }
}

impl SetupView {
    pub fn view(&self) -> Element<'_, SetupMessage> {
        let title = text("Kōnane").size(48);

        let subtitle = text("Traditional Hawaiian Board Game").size(24);

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

        // Import button
        let import_button = button(text("Import Game").size(16))
            .padding(10)
            .on_press(SetupMessage::ShowImportModal);

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
            text("").height(Length::Fixed(10.0)),
            import_button,
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        let main_view = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        if self.show_import_modal {
            let modal = self.import_modal_view();
            iced::widget::stack![main_view, modal].into()
        } else {
            main_view.into()
        }
    }

    fn import_modal_view(&self) -> Element<'_, SetupMessage> {
        let title = text("Import Game").size(24);

        let path_input = text_input("Enter file path...", &self.import_path)
            .on_input(SetupMessage::ImportPathChanged)
            .on_submit(SetupMessage::ImportGame)
            .padding(10)
            .width(Length::Fixed(300.0));

        let import_btn = button(text("Import").size(16))
            .padding(10)
            .on_press(SetupMessage::ImportGame);

        let cancel_btn = button(text("Cancel").size(16))
            .padding(10)
            .on_press(SetupMessage::CancelImport);

        let buttons = row![cancel_btn, import_btn].spacing(10);
        let mut modal_content = column![title, path_input];

        if let Some(error) = &self.import_error {
            modal_content = modal_content.push(text(format!("Error: {}", error)));
        }

        let modal_content = modal_content
            .push(buttons)
            .spacing(15)
            .align_x(Alignment::Center);

        let popup = container(modal_content)
            .width(Length::Fixed(400.0))
            .padding(30)
            .style(popup_style);

        container(popup)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(backdrop_style)
            .into()
    }
}

fn backdrop_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
        ..Default::default()
    }
}

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
