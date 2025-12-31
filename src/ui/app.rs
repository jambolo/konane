use std::time::Duration;

use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length, Subscription, Task};

use crate::audio::GameAudio;
use crate::game::rules::Jump;
use crate::game::{GamePhase, GameState, PieceColor, Position, Rules};
use crate::ui::board_view::{BoardMessage, BoardView};
use crate::ui::game_over_view::{GameOverMessage, GameOverView};
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
                self.view = AppView::Playing;
                self.update_status();
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
        }

        // Check for game over
        if let Some(ref state) = self.game_state
            && let GamePhase::GameOver { winner } = state.phase
        {
            self.game_over_view = Some(GameOverView::new(winner, state.move_history.clone()));
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
                    // Get the piece color before removal for animation
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
                // Check if clicking on a piece with valid moves
                let jumps = Rules::valid_jumps_from(state, pos);
                if !jumps.is_empty() {
                    self.board_view.select_piece(pos, jumps);
                    self.status_message =
                        format!("{}'s turn - Select destination", state.current_player);
                } else {
                    // Clicking elsewhere clears selection
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            _ => {}
        }
    }

    fn handle_jump_selected(&mut self, jump: Jump) {
        let Some(ref mut state) = self.game_state else {
            return;
        };

        // Get captured piece colors and positions before the move
        let captured_info: Vec<(Position, PieceColor)> = jump
            .captured
            .iter()
            .filter_map(|&pos| state.board.get_piece_color(pos).map(|color| (pos, color)))
            .collect();

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
            GameOverMessage::DownloadText => {
                if let Some(ref view) = self.game_over_view {
                    let log = view.generate_text_log();
                    self.save_log(&log, "konane_game.txt");
                }
            }
            GameOverMessage::DownloadJson => {
                if let Some(ref view) = self.game_over_view {
                    let log = view.generate_json_log();
                    self.save_log(&log, "konane_game.json");
                }
            }
        }
        Task::none()
    }

    fn save_log(&self, content: &str, filename: &str) {
        // Save to current directory
        if let Err(e) = std::fs::write(filename, content) {
            eprintln!("Failed to save log: {}", e);
        } else {
            println!("Game log saved to {}", filename);
        }
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
                // Show game board with overlay
                if let Some(ref game_over) = self.game_over_view {
                    let overlay = game_over.view().map(Message::GameOver);
                    container(column![overlay])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .into()
                } else {
                    self.playing_view()
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

        // Current player indicator
        let player_indicator = row![
            text("Current: ").size(16),
            text(state.current_player.to_string()).size(16),
        ]
        .spacing(5);

        let info_bar = row![player_indicator]
            .spacing(30)
            .align_y(Alignment::Center);

        // Board
        let board = self.board_view.view(state).map(Message::Board);

        let content = column![status, info_bar, board,]
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
