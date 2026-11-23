use iced::widget::{Canvas, Space, button, column, container, image, opaque, pick_list, radio, row, stack, text, text_input};
use iced::{Alignment, Element, Length};

use crate::game::PieceColor;
use crate::ui::board_view::stone_handle;
use crate::ui::theme;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SetupMessage {
    BoardSizeSelected(usize),
    ColorOptionSelected(ColorOption),
    BlackPlayerTypeSelected(PlayerType),
    WhitePlayerTypeSelected(PlayerType),
    StartGame,
    ImportGame,
    ShowImportModal,
    ImportPathChanged(String),
    CancelImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerType {
    #[default]
    Human,
    Ai,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
                if rand::random() {
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
    pub black_player_type: PlayerType,
    pub white_player_type: PlayerType,
    pub show_import_modal: bool,
    pub import_path: String,
    pub import_error: Option<String>,
}

impl Default for SetupView {
    fn default() -> Self {
        Self {
            board_size: 8,
            color_option: ColorOption::Black,
            black_player_type: PlayerType::Human,
            white_player_type: PlayerType::Ai,
            show_import_modal: false,
            import_path: String::new(),
            import_error: None,
        }
    }
}

/// A board size as the size picker shows it
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoardSize(usize);

impl std::fmt::Display for BoardSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0} × {0}", self.0)
    }
}

impl SetupView {
    pub fn view(&self) -> Element<'_, SetupMessage> {
        let mark = Canvas::new(theme::Mark).width(52.0).height(52.0);
        let title = text("Kōnane").font(theme::DISPLAY).size(56);
        let subtitle = text("The Hawaiian game of lava stone and coral")
            .size(17)
            .color(theme::CORAL_300);
        let header = column![mark, title, subtitle].spacing(4).align_x(Alignment::Center);

        // Board size selector
        let board_sizes: Vec<BoardSize> = (4..=16).step_by(2).map(BoardSize).collect();
        let size_picker = pick_list(board_sizes, Some(BoardSize(self.board_size)), |size| {
            SetupMessage::BoardSizeSelected(size.0)
        })
        .font(theme::BODY)
        .text_size(16)
        .padding([7, 12])
        .width(Length::Fixed(110.0))
        .style(theme::pick_list_style)
        .menu_style(theme::menu_style);
        let size_row = row![
            text("Board size").font(theme::LABEL).size(16),
            Space::new().width(Length::Fill),
            size_picker
        ]
        .align_y(Alignment::Center);

        let black_row = player_row(
            PieceColor::Black,
            self.black_player_type,
            SetupMessage::BlackPlayerTypeSelected,
        );
        let white_row = player_row(
            PieceColor::White,
            self.white_player_type,
            SetupMessage::WhitePlayerTypeSelected,
        );

        let start_button = button(
            text("Play")
                .font(theme::LABEL)
                .size(18)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .padding([11, 20])
        .width(Length::Fill)
        .style(theme::primary_button)
        .on_press(SetupMessage::StartGame);

        let card = container(column![size_row, column![black_row, white_row].spacing(8), start_button].spacing(20))
            .width(Length::Fixed(380.0))
            .padding(26)
            .style(theme::panel);

        let import_button = button(text("Open a saved game").size(15))
            .padding([6, 10])
            .style(theme::link_button)
            .on_press(SetupMessage::ShowImportModal);

        let content = column![header, card, import_button].spacing(26).align_x(Alignment::Center);

        let main_view = stack![
            Canvas::new(theme::Pattern).width(Length::Fill).height(Length::Fill),
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        ];

        if self.show_import_modal {
            stack![main_view, self.import_modal_view()].into()
        } else {
            main_view.into()
        }
    }

    fn import_modal_view(&self) -> Element<'_, SetupMessage> {
        let title = text("Open a saved game").font(theme::HEADING).size(22);

        let path_input = text_input("File name, for example konane-game.json", &self.import_path)
            .on_input(SetupMessage::ImportPathChanged)
            .on_submit(SetupMessage::ImportGame)
            .font(theme::BODY)
            .size(16)
            .padding(10)
            .style(theme::text_input_style);

        let import_btn = button(text("Open").font(theme::LABEL).size(15))
            .padding([8, 18])
            .style(theme::primary_button)
            .on_press(SetupMessage::ImportGame);

        let cancel_btn = button(text("Cancel").font(theme::LABEL).size(15))
            .padding([8, 18])
            .style(theme::secondary_button)
            .on_press(SetupMessage::CancelImport);

        let buttons = row![Space::new().width(Length::Fill), cancel_btn, import_btn].spacing(10);
        let mut modal_content = column![title, path_input].spacing(16);

        if let Some(error) = &self.import_error {
            modal_content = modal_content.push(
                column![
                    text(import_error_headline(error)).size(15).color(theme::EMBER_300),
                    text(error.as_str()).size(13).color(theme::CORAL_500),
                ]
                .spacing(2),
            );
        }

        let popup = container(modal_content.push(buttons))
            .width(Length::Fixed(420.0))
            .padding(26)
            .style(theme::panel);

        opaque(
            container(popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(theme::scrim),
        )
    }
}

/// One player's row: stone, color name, and who plays it
fn player_row<'a>(color: PieceColor, selected: PlayerType, on_select: fn(PlayerType) -> SetupMessage) -> Element<'a, SetupMessage> {
    let choice = |label: &'a str, player_type: PlayerType| {
        radio(label, player_type, Some(selected), on_select)
            .font(theme::BODY)
            .text_size(16)
            .size(18)
            .spacing(8)
            .style(theme::radio_style)
    };
    row![
        image(stone_handle(color)).width(40.0).height(40.0),
        text(color.to_string()).font(theme::LABEL).size(16).width(Length::Fill),
        choice("Person", PlayerType::Human),
        choice("Computer", PlayerType::Ai),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

/// Says in plain words why a saved game could not be opened
fn import_error_headline(error: &str) -> &'static str {
    if error.starts_with("Failed to read file") {
        "That file couldn't be opened. Check the name and try again."
    } else {
        "That file isn't a saved kōnane game."
    }
}
