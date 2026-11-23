use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use iced::widget::{Space, button, column, container, row, scrollable, stack, text};
use iced::{Alignment, Element, Length, Task};
use konane::import;

use crate::game::player::{Player, PlayerMove};
use crate::game::rules::Jump;
use crate::game::{AiPlayer, GamePhase, KonaneState, MoveHistory, MoveRecord, PieceColor, Position, Rules, UndoRedoStack};
use crate::ui::board_view::{BoardMessage, BoardView};
use crate::ui::game_over_view::{ExportFormat, GameOverMessage, GameOverView};
use crate::ui::setup_view::{PlayerType, SetupMessage, SetupView};

// Held for the duration of an AI search so that searches run one at a time
static AI_SEARCH_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
pub enum Message {
    Setup(SetupMessage),
    Board(BoardMessage),
    GameOver(GameOverMessage),
    AiMoveComputed(u64, Option<PlayerMove>),
    NewGame,
}

pub enum AppView {
    Setup,
    Playing,
    GameOver,
}

pub struct KonaneApp {
    view: AppView,
    setup: SetupView,
    game_state: Option<KonaneState>,
    board_view: BoardView,
    game_over_view: Option<GameOverView>,
    status_message: String,
    move_history: MoveHistory,
    // Move list text, kept in step with move_history
    move_labels: Vec<String>,
    undo_stack: UndoRedoStack,
    redo_stack: UndoRedoStack,
    black_player_type: PlayerType,
    white_player_type: PlayerType,
    ai_computing: bool,
    ai_depth: u32,
    // Incremented whenever a pending AI result must be discarded (game canceled/restarted, undo/redo)
    ai_generation: Arc<AtomicU64>,
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
            move_history: Vec::new(),
            move_labels: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            black_player_type: PlayerType::Human,
            white_player_type: PlayerType::Human,
            ai_computing: false,
            ai_depth: 8,
            ai_generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl KonaneApp {
    pub fn new(ai_depth: u32) -> (Self, iced::Task<Message>) {
        let app = Self {
            ai_depth,
            ..Default::default()
        };
        (app, iced::Task::none())
    }

    pub fn title(&self) -> String {
        "Kōnane".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Setup(msg) => self.handle_setup(msg),
            Message::Board(msg) => self.handle_board(msg),
            Message::GameOver(msg) => self.handle_game_over(msg),
            Message::AiMoveComputed(generation, maybe_move) => self.handle_ai_move(generation, maybe_move),
            Message::NewGame => {
                self.cancel_ai_move();
                self.view = AppView::Setup;
                self.game_state = None;
                self.game_over_view = None;
                self.board_view = BoardView::default();
                self.set_history(Vec::new());
                self.undo_stack.clear();
                self.redo_stack.clear();
                Task::none()
            }
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
            SetupMessage::BlackPlayerTypeSelected(player_type) => {
                self.setup.black_player_type = player_type;
            }
            SetupMessage::WhitePlayerTypeSelected(player_type) => {
                self.setup.white_player_type = player_type;
            }
            SetupMessage::StartGame => {
                let first_player = self.setup.color_option.to_piece_color();
                self.game_state = Some(KonaneState::new(self.setup.board_size, first_player));
                self.board_view = BoardView::default();
                self.set_history(Vec::new());
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.black_player_type = self.setup.black_player_type;
                self.white_player_type = self.setup.white_player_type;
                self.ai_computing = false;
                self.view = AppView::Playing;
                self.update_status();
                return self.maybe_trigger_ai_move();
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
                    Ok((state, move_history, undo_stack)) => {
                        self.game_state = Some(state);
                        self.board_view = BoardView::default();
                        self.set_history(move_history);
                        self.undo_stack = undo_stack;
                        self.redo_stack.clear();
                        self.black_player_type = self.setup.black_player_type;
                        self.white_player_type = self.setup.white_player_type;
                        self.ai_computing = false;
                        self.view = AppView::Playing;
                        self.update_status();
                        self.setup.show_import_modal = false;
                        self.setup.import_path.clear();
                        self.setup.import_error = None;
                        return self.maybe_trigger_ai_move();
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
        if self.game_state.is_none() || !matches!(self.view, AppView::Playing) {
            return Task::none();
        }

        // Block UI input if AI is computing or if it's an AI's turn
        if self.ai_computing || self.is_current_player_ai() {
            match msg {
                BoardMessage::Undo | BoardMessage::Redo => {}
                _ => return Task::none(),
            }
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
            && let GamePhase::GameOver { winner } = state.current_phase()
        {
            self.game_over_view = Some(GameOverView::new(winner, self.move_history.clone(), state.board().size()));
            self.view = AppView::GameOver;
            return Task::none();
        }

        // Trigger AI if needed
        self.maybe_trigger_ai_move()
    }

    fn handle_cell_click(&mut self, pos: Position) {
        let Some(ref mut state) = self.game_state else {
            return;
        };

        match state.current_phase() {
            GamePhase::OpeningBlackRemoval => {
                let valid = Rules::valid_black_opening_removals(state);
                if valid.contains(&pos) {
                    self.save_state_for_undo();
                    let state = self.game_state.as_mut().unwrap();
                    let color = state.board().get_piece_color(pos).unwrap_or(PieceColor::Black);
                    if let Ok(record) = Rules::apply_opening_removal(state, pos) {
                        self.push_move(record);
                    }
                    self.board_view.animate_removal(pos, color);
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            GamePhase::OpeningWhiteRemoval => {
                let valid = Rules::valid_white_opening_removals(state);
                if valid.contains(&pos) {
                    self.save_state_for_undo();
                    let state = self.game_state.as_mut().unwrap();
                    let color = state.board().get_piece_color(pos).unwrap_or(PieceColor::White);
                    if let Ok(record) = Rules::apply_opening_removal(state, pos) {
                        self.push_move(record);
                    }
                    self.board_view.animate_removal(pos, color);
                    self.board_view.clear_selection();
                    self.update_status();
                }
            }
            GamePhase::Play => {
                let jumps = Rules::valid_jumps_from(state, pos);
                if !jumps.is_empty() {
                    self.board_view.select_piece(pos, jumps);
                    self.status_message = format!("{}'s turn - Select destination", state.current_player());
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

        // Get removed piece colors and positions before the move
        let removed_info: Vec<(Position, PieceColor)> = jump
            .removed()
            .filter_map(|pos| state.board().get_piece_color(pos).map(|color| (pos, color)))
            .collect();

        self.save_state_for_undo();
        let state = self.game_state.as_mut().unwrap();

        // Apply the jump and record it
        let record = Rules::apply_jump(state, &jump);
        self.push_move(record);

        // Animate all removed pieces
        for (pos, color) in removed_info {
            self.board_view.animate_removal(pos, color);
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
                    view.export_error = None;
                }
            }
            GameOverMessage::ExportPathChanged(path) => {
                if let Some(ref mut view) = self.game_over_view {
                    view.export_path = path;
                    view.export_error = None;
                }
            }
            GameOverMessage::CancelExport => {
                if let Some(ref mut view) = self.game_over_view {
                    view.show_export_modal = false;
                    view.export_path.clear();
                    view.export_format = None;
                    view.export_error = None;
                }
            }
            GameOverMessage::ConfirmExport => {
                if let Some(ref mut view) = self.game_over_view {
                    let content = match view.export_format {
                        Some(ExportFormat::Text) => view.generate_text_log(),
                        Some(ExportFormat::Json) => view.generate_json_log(),
                        None => return Task::none(),
                    };
                    let result = if view.export_path.trim().is_empty() {
                        Err("No file path entered".to_string())
                    } else {
                        std::fs::write(&view.export_path, content).map_err(|error| error.to_string())
                    };
                    // The modal stays open on failure so the error is visible
                    match result {
                        Ok(()) => {
                            view.show_export_modal = false;
                            view.export_path.clear();
                            view.export_format = None;
                        }
                        Err(error) => view.export_error = Some(error),
                    }
                }
            }
        }
        Task::none()
    }

    fn save_state_for_undo(&mut self) {
        if let Some(ref state) = self.game_state {
            self.undo_stack.push((state.clone(), self.move_history.clone()));
            self.redo_stack.clear();
        }
    }

    // Invalidates any in-flight AI computation so its result is discarded
    fn cancel_ai_move(&mut self) {
        self.ai_generation.fetch_add(1, Ordering::Relaxed);
        self.ai_computing = false;
    }

    fn push_move(&mut self, record: MoveRecord) {
        self.move_labels.push(move_label(self.move_history.len(), &record));
        self.move_history.push(record);
    }

    fn set_history(&mut self, history: MoveHistory) {
        self.move_history = history;
        self.rebuild_move_labels();
    }

    fn rebuild_move_labels(&mut self) {
        self.move_labels = self
            .move_history
            .iter()
            .enumerate()
            .map(|(i, record)| move_label(i, record))
            .collect();
    }

    // Index of the latest undo snapshot in which a human is to move
    fn undo_target(&self) -> Option<usize> {
        self.undo_stack
            .iter()
            .rposition(|(state, _)| !self.is_ai(state.current_player()))
    }

    // Rolls back to before the last human move, so the AI does not immediately replay the move that was undone
    fn handle_undo(&mut self) {
        let Some(target) = self.undo_target() else {
            return;
        };
        // Snapshots in which the AI is to move are skipped, but kept for redo
        let skipped = self.undo_stack.split_off(target + 1);
        let Some((previous_state, previous_history)) = self.undo_stack.pop() else {
            return;
        };

        self.cancel_ai_move();
        if let Some(current_state) = self.game_state.replace(previous_state) {
            self.redo_stack.push((current_state, std::mem::take(&mut self.move_history)));
        }
        self.redo_stack.extend(skipped.into_iter().rev());
        self.set_history(previous_history);
        self.board_view.clear_selection();
        self.board_view.invalidate_foreground_caches();
        self.update_status();
    }

    // Steps forward to the next state in which a human is to move, or to the newest state
    fn handle_redo(&mut self) {
        if self.redo_stack.is_empty() {
            return;
        }

        self.cancel_ai_move();
        while let Some((next_state, next_history)) = self.redo_stack.pop() {
            let current_history = std::mem::replace(&mut self.move_history, next_history);
            if let Some(current_state) = self.game_state.replace(next_state) {
                self.undo_stack.push((current_state, current_history));
            }
            if !self.is_current_player_ai() {
                break;
            }
        }
        self.rebuild_move_labels();
        self.board_view.clear_selection();
        self.board_view.invalidate_foreground_caches();
        self.update_status();
    }

    fn can_undo(&self) -> bool {
        self.undo_target().is_some()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn update_status(&mut self) {
        let Some(ref state) = self.game_state else {
            return;
        };

        let is_ai = self.is_current_player_ai();
        let ai_suffix = if is_ai { " (AI)" } else { "" };

        self.status_message = match state.current_phase() {
            GamePhase::OpeningBlackRemoval => {
                format!("Black{}: Remove a black piece from the center or a corner", ai_suffix)
            }
            GamePhase::OpeningWhiteRemoval => {
                format!("White{}: Remove a white piece adjacent to the empty space", ai_suffix)
            }
            GamePhase::Play => {
                if self.ai_computing {
                    format!("{}{} is thinking...", state.current_player(), ai_suffix)
                } else {
                    format!("{}{}'s turn - Select a piece to move", state.current_player(), ai_suffix)
                }
            }
            GamePhase::GameOver { winner } => {
                format!("{} wins!", winner)
            }
            _ => String::new(),
        };
    }

    fn is_ai(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::Black => self.black_player_type == PlayerType::Ai,
            PieceColor::White => self.white_player_type == PlayerType::Ai,
        }
    }

    fn is_current_player_ai(&self) -> bool {
        self.game_state
            .as_ref()
            .is_some_and(|state| self.is_ai(state.current_player()))
    }

    fn maybe_trigger_ai_move(&mut self) -> Task<Message> {
        if self.ai_computing {
            return Task::none();
        }

        let Some(ref state) = self.game_state else {
            return Task::none();
        };

        if matches!(state.current_phase(), GamePhase::GameOver { .. }) {
            return Task::none();
        }

        if !self.is_current_player_ai() {
            return Task::none();
        }

        let state_clone = state.clone();
        let depth = self.ai_depth;
        let current_generation = Arc::clone(&self.ai_generation);
        let generation = current_generation.load(Ordering::Relaxed);

        self.ai_computing = true;
        self.update_status();

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    // Wait for any earlier search to finish, then skip this one if it was canceled while waiting
                    let _guard = AI_SEARCH_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
                    if current_generation.load(Ordering::Relaxed) != generation {
                        return None;
                    }
                    let mut ai = AiPlayer::new(state_clone.current_player(), depth);
                    ai.request_move(&state_clone)
                })
                .await
                .ok()
                .flatten()
            },
            move |maybe_move| Message::AiMoveComputed(generation, maybe_move),
        )
    }

    fn handle_ai_move(&mut self, generation: u64, maybe_move: Option<PlayerMove>) -> Task<Message> {
        // Result from a canceled game — ignore it
        if generation != self.ai_generation.load(Ordering::Relaxed) {
            return Task::none();
        }

        self.ai_computing = false;

        let Some(player_move) = maybe_move else {
            self.update_status();
            return Task::none();
        };

        match player_move {
            PlayerMove::OpeningRemoval(pos) => {
                let Some(ref mut state) = self.game_state else {
                    return Task::none();
                };
                let color = state.board().get_piece_color(pos).unwrap_or(state.current_player());
                self.save_state_for_undo();
                let state = self.game_state.as_mut().unwrap();
                if let Ok(record) = Rules::apply_opening_removal(state, pos) {
                    self.push_move(record);
                }
                self.board_view.animate_removal(pos, color);
                self.board_view.clear_selection();
            }
            PlayerMove::Jump(jump) => {
                let Some(ref state) = self.game_state else {
                    return Task::none();
                };
                let removed_info: Vec<(Position, PieceColor)> = jump
                    .removed()
                    .filter_map(|pos| state.board().get_piece_color(pos).map(|color| (pos, color)))
                    .collect();

                self.save_state_for_undo();
                let state = self.game_state.as_mut().unwrap();
                let record = Rules::apply_jump(state, &jump);
                self.push_move(record);

                for (pos, color) in removed_info {
                    self.board_view.animate_removal(pos, color);
                }

                self.board_view.clear_selection();
            }
        }

        self.update_status();

        // Check for game over
        if let Some(ref state) = self.game_state
            && let GamePhase::GameOver { winner } = state.current_phase()
        {
            self.game_over_view = Some(GameOverView::new(winner, self.move_history.clone(), state.board().size()));
            self.view = AppView::GameOver;
            return Task::none();
        }

        // Continue with next AI move if needed
        self.maybe_trigger_ai_move()
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
        let player_indicator = row![text("Current: ").size(16), text(state.current_player().to_string()).size(16),].spacing(5);

        let new_game_btn = button(text("New Game").size(14)).on_press(Message::NewGame);

        let info_bar = row![undo_btn, redo_btn, player_indicator, new_game_btn]
            .spacing(15)
            .align_y(Alignment::Center);

        // Board canvas (labels rendered inside canvas)
        let board = self.board_view.view(state).map(Message::Board);

        // Move list
        let mut move_list = column![].spacing(4);
        for label in &self.move_labels {
            move_list = move_list.push(text(label.as_str()).size(14));
        }
        let move_list_padded = row![move_list, Space::new().width(15.0)];
        let move_panel = container(scrollable(move_list_padded).height(Length::Fill).width(Length::Fill))
            .width(Length::Fixed(150.0))
            .height(Length::Fill)
            .padding(10);

        let board_row = row![board, move_panel].spacing(0);

        let content = column![status, info_bar, board_row]
            .spacing(10)
            .padding(20)
            .align_x(Alignment::Center);

        container(content).width(Length::Fill).height(Length::Fill).into()
    }
}

fn move_label(index: usize, record: &MoveRecord) -> String {
    format!("{}. {}", index + 1, record.to_algebraic())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start(black: PlayerType, white: PlayerType) -> KonaneApp {
        let (mut app, _) = KonaneApp::new(2);
        app.setup.board_size = 6;
        app.setup.black_player_type = black;
        app.setup.white_player_type = white;
        let _ = app.update(Message::Setup(SetupMessage::StartGame));
        app
    }

    // Plays the first legal move for the human to move
    fn human_move(app: &mut KonaneApp) {
        let state = app.game_state.as_ref().unwrap();
        let message = match state.current_phase() {
            GamePhase::OpeningBlackRemoval => BoardMessage::CellClicked(Rules::valid_black_opening_removals(state)[0]),
            GamePhase::OpeningWhiteRemoval => BoardMessage::CellClicked(Rules::valid_white_opening_removals(state)[0]),
            _ => BoardMessage::JumpSelected(Rules::valid_jumps_from(state, Rules::movable_pieces(state)[0])[0]),
        };
        let _ = app.update(Message::Board(message));
    }

    // Completes the pending AI search synchronously
    fn ai_move(app: &mut KonaneApp) {
        assert!(app.ai_computing);
        let state = app.game_state.as_ref().unwrap();
        let player_move = AiPlayer::new(state.current_player(), app.ai_depth).request_move(state);
        let generation = app.ai_generation.load(Ordering::Relaxed);
        let _ = app.update(Message::AiMoveComputed(generation, player_move));
    }

    fn play(app: &mut KonaneApp, moves: usize) {
        while app.move_history.len() < moves {
            if app.is_current_player_ai() {
                ai_move(app)
            } else {
                human_move(app)
            }
        }
    }

    fn undo(app: &mut KonaneApp) {
        let _ = app.update(Message::Board(BoardMessage::Undo));
    }

    fn redo(app: &mut KonaneApp) {
        let _ = app.update(Message::Board(BoardMessage::Redo));
    }

    #[test]
    fn undo_against_ai_rolls_back_to_before_the_human_move() {
        let mut app = start(PlayerType::Human, PlayerType::Ai);
        play(&mut app, 6);
        let labels = app.move_labels.clone();

        undo(&mut app);
        assert_eq!(app.move_history.len(), 4);
        assert_eq!(app.move_labels, labels[..4]);
        assert!(!app.is_current_player_ai());
        assert!(!app.ai_computing);

        redo(&mut app);
        assert_eq!(app.move_labels, labels);
        assert!(!app.is_current_player_ai());
        assert!(!app.ai_computing);
        assert!(app.redo_stack.is_empty());
        assert_eq!(app.undo_stack.len(), 6);
    }

    #[test]
    fn undo_while_ai_is_thinking_cancels_the_search() {
        let mut app = start(PlayerType::Human, PlayerType::Ai);
        play(&mut app, 4);
        human_move(&mut app);
        assert!(app.ai_computing);
        let generation = app.ai_generation.load(Ordering::Relaxed);

        undo(&mut app);
        assert_eq!(app.move_history.len(), 4);
        assert!(!app.ai_computing);
        assert_ne!(app.ai_generation.load(Ordering::Relaxed), generation);

        // Redo returns to the AI's turn and restarts the search
        redo(&mut app);
        assert_eq!(app.move_history.len(), 5);
        assert!(app.ai_computing);
    }

    #[test]
    fn undo_is_unavailable_without_a_human_move() {
        let mut app = start(PlayerType::Ai, PlayerType::Human);
        play(&mut app, 1);
        assert!(!app.can_undo());
        undo(&mut app);
        assert_eq!(app.move_history.len(), 1);

        play(&mut app, 3);
        assert!(app.can_undo());
        undo(&mut app);
        assert_eq!(app.move_history.len(), 1);
        assert!(!app.can_undo());
    }

    #[test]
    fn undo_is_unavailable_when_both_players_are_ai() {
        let mut app = start(PlayerType::Ai, PlayerType::Ai);
        play(&mut app, 4);
        assert!(!app.can_undo());
        undo(&mut app);
        assert_eq!(app.move_history.len(), 4);
    }

    #[test]
    fn undo_between_humans_steps_one_move() {
        let mut app = start(PlayerType::Human, PlayerType::Human);
        play(&mut app, 5);
        undo(&mut app);
        assert_eq!(app.move_history.len(), 4);
        redo(&mut app);
        assert_eq!(app.move_history.len(), 5);
        assert_eq!(app.move_labels.len(), 5);
    }

    #[test]
    fn failed_export_keeps_the_modal_open() {
        let mut app = start(PlayerType::Human, PlayerType::Human);
        while matches!(app.view, AppView::Playing) {
            human_move(&mut app);
        }
        let _ = app.update(Message::GameOver(GameOverMessage::ShowExportModal(ExportFormat::Text)));
        let _ = app.update(Message::GameOver(GameOverMessage::ConfirmExport));
        let view = app.game_over_view.as_ref().unwrap();
        assert!(view.show_export_modal);
        assert!(view.export_error.is_some());
    }
}
