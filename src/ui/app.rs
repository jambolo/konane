use std::time::Duration;

use iced::widget::{button, column, container, row, scrollable, stack, text};
use iced::{Alignment, Element, Length, Subscription, Task};
use konane::import;

use crate::audio::GameAudio;
use crate::game::rules::Jump;
use crate::game::{GamePhase, GameState, PieceColor, Position, Rules};
use crate::ui::board_view::{BoardMessage, BoardView};
use crate::ui::game_over_view::{ExportFormat, GameOverMessage, GameOverView};
use crate::ui::setup_view::{SetupMessage, SetupView};

#[derive(Debug, Clone)]
pub enum Message {
    Setup(SetupMessage),
    Board(BoardMessage),
    GameOver(GameOverMessage),
    Tick,
}

pub enum AppView {
    Setup,
    Playing,
    GameOver,
}

pub struct KonaneApp {
    view: AppView,
    setup: SetupView,
    game_state: Option<GameState>,
    board_view: BoardView,
    game_over_view: Option<GameOverView>,
    status_message: String,
    audio: GameAudio,
    undo_stack: Vec<GameState>,
    redo_stack: Vec<GameState>,
}

impl Default for KonaneApp {
    fn default() -> Self {
        Self {
            view: AppView::Setup,
            setup: SetupView::default(),
            game_state: None,
            board_view: BoardView::default(),
            game_over_view: None,
            status_message: String::new(),
            audio: GameAudio::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl KonaneApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&self) -> String {
        "Kōnane".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Setup(msg) => self.handle_setup(msg),
            Message::Board(msg) => self.handle_board(msg),
            Message::GameOver(msg) => self.handle_game_over(msg),
            Message::Tick => {
                self.board_view.update_animations();
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Only subscribe to ticks when there are animations running
        if self.board_view.has_animations() {
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn handle_setup(&mut self, msg: SetupMessage) -> Task<Message> {
        match msg {
            SetupMessage::BoardSizeSelected(size) => {
                self.setup.board_size = size;
            }
            SetupMessage::ColorOptionSelected(option) => {
                self.setup.color_option = option;
            }
            SetupMessage::StartGame => {
                let first_player = self.setup.color_option.to_piece_color();
                self.game_state = Some(GameState::new(self.setup.board_size, first_player));
                self.board_view = BoardView::default();
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.view = AppView::Playing;
                self.update_status();
            }
            SetupMessage::ShowImportModal => {
                self.setup.show_import_modal = true;
                self.setup.import_path.clear();
                self.setup.import_error = None;
            }
            SetupMessage::ImportPathChanged(path) => {
                self.setup.import_path = path;
                self.setup.import_error = None;
            }
            SetupMessage::CancelImport => {
                self.setup.show_import_modal = false;
                self.setup.import_path.clear();
                self.setup.import_error = None;
            }
            SetupMessage::ImportGame => {
                let path = self.setup.import_path.clone();
                match import::import_game_from_path(&path) {
                    Ok((state, history)) => {
                        self.game_state = Some(state);
                        self.board_view = BoardView::default();
                        self.undo_stack = history;
                        self.redo_stack.clear();
                        self.view = AppView::Playing;
                        self.update_status();
                        self.setup.show_import_modal = false;
                        self.setup.import_path.clear();
                        self.setup.import_error = None;
                    }
                    Err(error) => {
                        self.setup.show_import_modal = true;
                        self.setup.import_error = Some(error);
                    }
                }
            }
        }
        Task::none()
    }

    fn handle_board(&mut self, msg: BoardMessage) -> Task<Message> {
        if self.game_state.is_none() {
            return Task::none();
        }

        match msg {
            BoardMessage::CellClicked(pos) => {
                self.handle_cell_click(pos);
            }
            BoardMessage::JumpSelected(jump) => {
                self.handle_jump_selected(jump);
            }
            BoardMessage::Undo => {
                self.handle_undo();
            }
            BoardMessage::Redo => {
                self.handle_redo();
            }
        }

        // Check for game over
        if let Some(ref state) = self.game_state
            && let GamePhase::GameOver { winner } = state.phase
        {
            self.game_over_view = Some(GameOverView::new(
                winner,
                state.move_history.clone(),
                state.board.size(),
            ));
            self.view = AppView::GameOver;
        }

        Task::none()
    }

    fn handle_cell_click(&mut self, pos: Position) {
        let Some(ref mut state) = self.game_state else {
            return;
        };

        match state.phase {
            GamePhase::OpeningBlackRemoval => {
                let valid = Rules::valid_black_opening_removals(state);
                if valid.contains(&pos) {
                    self.save_state_for_undo();
                    let state = self.game_state.as_mut().unwrap();
                    let color = state
                        .board
                        .get_piece_color(pos)
                        .unwrap_or(PieceColor::Black);
                    let _ = Rules::apply_opening_removal(state, pos);
                    self.board_view.animate_removal(pos, color);
                    self.audio.play_capture();
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            GamePhase::OpeningWhiteRemoval => {
                let valid = Rules::valid_white_opening_removals(state);
                if valid.contains(&pos) {
                    self.save_state_for_undo();
                    let state = self.game_state.as_mut().unwrap();
                    let color = state
                        .board
                        .get_piece_color(pos)
                        .unwrap_or(PieceColor::White);
                    let _ = Rules::apply_opening_removal(state, pos);
                    self.board_view.animate_removal(pos, color);
                    self.audio.play_capture();
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            GamePhase::Play => {
                let jumps = Rules::valid_jumps_from(state, pos);
                if !jumps.is_empty() {
                    self.board_view.select_piece(pos, jumps);
                    self.status_message =
                        format!("{}'s turn - Select destination", state.current_player);
                } else {
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            _ => {}
        }
    }

    fn handle_jump_selected(&mut self, jump: Jump) {
        let Some(ref state) = self.game_state else {
            return;
        };

        // Get captured piece colors and positions before the move
        let captured_info: Vec<(Position, PieceColor)> = jump
            .captured
            .iter()
            .filter_map(|&pos| state.board.get_piece_color(pos).map(|color| (pos, color)))
            .collect();

        self.save_state_for_undo();
        let state = self.game_state.as_mut().unwrap();

        // Apply the jump
        Rules::apply_jump(state, &jump);

        // Animate all captured pieces
        for (pos, color) in captured_info {
            self.board_view.animate_removal(pos, color);
        }

        // Play sounds
        self.audio.play_move();
        for _ in 0..jump.captured.len() {
            self.audio.play_capture();
        }

        self.board_view.clear_selection();
        self.update_status();
    }

    fn handle_game_over(&mut self, msg: GameOverMessage) -> Task<Message> {
        match msg {
            GameOverMessage::Dismiss => {
                self.view = AppView::Setup;
                self.game_state = None;
                self.game_over_view = None;
                self.board_view = BoardView::default();
            }
            GameOverMessage::ShowExportModal(format) => {
                if let Some(ref mut view) = self.game_over_view {
                    view.show_export_modal = true;
                    view.export_format = Some(format);
                    view.export_path.clear();
                }
            }
            GameOverMessage::ExportPathChanged(path) => {
                if let Some(ref mut view) = self.game_over_view {
                    view.export_path = path;
                }
            }
            GameOverMessage::CancelExport => {
                if let Some(ref mut view) = self.game_over_view {
                    view.show_export_modal = false;
                    view.export_path.clear();
                    view.export_format = None;
                }
            }
            GameOverMessage::ConfirmExport => {
                if let Some(ref mut view) = self.game_over_view {
                    let path = view.export_path.clone();
                    let content = match view.export_format {
                        Some(ExportFormat::Text) => view.generate_text_log(),
                        Some(ExportFormat::Json) => view.generate_json_log(),
                        None => return Task::none(),
                    };
                    let _ = std::fs::write(&path, content);
                    view.show_export_modal = false;
                    view.export_path.clear();
                    view.export_format = None;
                }
            }
        }
        Task::none()
    }

    fn save_state_for_undo(&mut self) {
        if let Some(ref state) = self.game_state {
            self.undo_stack.push(state.clone());
            self.redo_stack.clear();
        }
    }

    fn handle_undo(&mut self) {
        if let Some(previous_state) = self.undo_stack.pop() {
            if let Some(current_state) = self.game_state.take() {
                self.redo_stack.push(current_state);
            }
            self.game_state = Some(previous_state);
            self.board_view.clear_selection();
            self.update_status();
        }
    }

    fn handle_redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            if let Some(current_state) = self.game_state.take() {
                self.undo_stack.push(current_state);
            }
            self.game_state = Some(next_state);
            self.board_view.clear_selection();
            self.update_status();
        }
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn update_status(&mut self) {
        let Some(ref state) = self.game_state else {
            return;
        };

        self.status_message = match state.phase {
            GamePhase::OpeningBlackRemoval => {
                "Black: Remove a black piece from the center or a corner".to_string()
            }
            GamePhase::OpeningWhiteRemoval => {
                "White: Remove a white piece adjacent to the empty space".to_string()
            }
            GamePhase::Play => {
                format!("{}'s turn - Select a piece to move", state.current_player)
            }
            GamePhase::GameOver { winner } => {
                format!("{} wins!", winner)
            }
            _ => String::new(),
        };
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self.view {
            AppView::Setup => self.setup.view().map(Message::Setup),
            AppView::Playing => self.playing_view(),
            AppView::GameOver => {
                // Show game board with popup overlay
                let board = self.playing_view();
                if let Some(ref game_over) = self.game_over_view {
                    let overlay = game_over.view().map(Message::GameOver);
                    let overlay_container = container(overlay)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill);
                    stack![board, overlay_container].into()
                } else {
                    board
                }
            }
        }
    }

    fn playing_view(&self) -> Element<'_, Message> {
        let Some(ref state) = self.game_state else {
            return text("No game in progress").into();
        };

        // Status bar
        let status = text(&self.status_message).size(20);

        // Undo/Redo buttons
        let undo_btn = button(text("Undo").size(14));
        let undo_btn = if self.can_undo() {
            undo_btn.on_press(Message::Board(BoardMessage::Undo))
        } else {
            undo_btn
        };

        let redo_btn = button(text("Redo").size(14));
        let redo_btn = if self.can_redo() {
            redo_btn.on_press(Message::Board(BoardMessage::Redo))
        } else {
            redo_btn
        };

        // Current player indicator
        let player_indicator = row![
            text("Current: ").size(16),
            text(state.current_player.to_string()).size(16),
        ]
        .spacing(5);

        let info_bar = row![undo_btn, redo_btn, player_indicator]
            .spacing(15)
            .align_y(Alignment::Center);

        // Board
        let board = self.board_view.view(state).map(Message::Board);

        // Move list
        let mut move_list = column![].spacing(4);
        for (i, record) in state.move_history.iter().enumerate() {
            move_list = move_list.push(text(format!("{}. {}", i + 1, record.to_algebraic())).size(14));
        }
        let move_list_padded = row![move_list, iced::widget::Space::new().width(15.0)];
        let move_panel = container(
            scrollable(move_list_padded)
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .width(Length::Fixed(150.0))
        .height(Length::Fill)
        .padding(10);

        let board_row = row![board, move_panel].spacing(10);

        let content = column![status, info_bar, board_row]
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
