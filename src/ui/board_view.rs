use std::sync::OnceLock;
use std::time::Instant;

use iced::mouse;
use iced::widget::canvas::{
    self, Action, Canvas, Event, Frame, Geometry, Image, Path, Stroke, Text,
};
use iced::widget::image::Handle;
use iced::widget::Stack;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::game::rules::Jump;
use crate::game::{Cell, GamePhase, GameState, PieceColor, Position, Rules};

static BLACK_STONE_PATH: &str = "data/black-stone-15.png";
static WHITE_STONE_PATH: &str = "data/white-stone-15.png";
static BACKGROUND_PATH: &str = "data/background.jpg";

static BLACK_STONE: OnceLock<Handle> = OnceLock::new();
static WHITE_STONE: OnceLock<Handle> = OnceLock::new();
static BACKGROUND_TILE: OnceLock<Handle> = OnceLock::new();

fn get_black_stone() -> Handle {
    BLACK_STONE
        .get_or_init(|| {
            let bytes = std::fs::read(BLACK_STONE_PATH).expect("Failed to load black stone");
            Handle::from_bytes(bytes)
        })
        .clone()
}

fn get_white_stone() -> Handle {
    WHITE_STONE
        .get_or_init(|| {
            let bytes = std::fs::read(WHITE_STONE_PATH).expect("Failed to load white stone");
            Handle::from_bytes(bytes)
        })
        .clone()
}

fn get_background_tile() -> Handle {
    BACKGROUND_TILE
        .get_or_init(|| {
            let bytes = std::fs::read(BACKGROUND_PATH).expect("Failed to load background");
            Handle::from_bytes(bytes)
        })
        .clone()
}

const ANIMATION_DURATION_MS: u64 = 300;
const LABEL_MARGIN: f32 = 20.0;

#[derive(Debug, Clone)]
pub enum BoardMessage {
    CellClicked(Position),
    JumpSelected(Jump),
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub enum SelectionState {
    None,
    PieceSelected(Position, Vec<Jump>),
}

/// Animation state for a piece being removed
#[derive(Debug, Clone)]
pub struct RemovalAnimation {
    pub position: Position,
    pub color: PieceColor,
    pub start_time: Instant,
}

impl RemovalAnimation {
    pub fn new(position: Position, color: PieceColor) -> Self {
        Self {
            position,
            color,
            start_time: Instant::now(),
        }
    }

    /// Returns progress from 0.0 to 1.0
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let duration = ANIMATION_DURATION_MS as f32;
        (elapsed / duration).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }
}

pub struct BoardView {
    selection: SelectionState,
    pub animations: Vec<RemovalAnimation>,
    background_cache: canvas::Cache,
    stone_cache: canvas::Cache,
    highlight_cache: canvas::Cache,
}

impl Default for BoardView {
    fn default() -> Self {
        Self {
            selection: SelectionState::None,
            animations: Vec::new(),
            background_cache: canvas::Cache::new(),
            stone_cache: canvas::Cache::new(),
            highlight_cache: canvas::Cache::new(),
        }
    }
}

impl BoardView {
    pub fn select_piece(&mut self, pos: Position, jumps: Vec<Jump>) {
        self.selection = SelectionState::PieceSelected(pos, jumps);
        self.highlight_cache.clear();
    }

    pub fn clear_selection(&mut self) {
        self.selection = SelectionState::None;
        self.highlight_cache.clear();
    }

    pub fn _selection(&self) -> &SelectionState {
        &self.selection
    }

    /// Start an animation for a removed piece
    pub fn animate_removal(&mut self, position: Position, color: PieceColor) {
        self.animations.push(RemovalAnimation::new(position, color));
        self.stone_cache.clear();
        self.highlight_cache.clear();
    }

    /// Update animations and remove completed ones
    pub fn update_animations(&mut self) {
        let had_animations = !self.animations.is_empty();
        self.animations.retain(|anim| !anim.is_complete());
        if had_animations {
            self.stone_cache.clear();
        }
    }

    /// Check if any animations are running
    pub fn has_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    /// Invalidate all caches (call when game state changes externally)
    pub fn invalidate_foreground_caches(&mut self) {
        self.stone_cache.clear();
        self.highlight_cache.clear();
    }
    pub fn view<'a>(&'a self, state: &'a GameState) -> Element<'a, BoardMessage> {
        // Use Stack to layer canvases - iced's canvas batches primitives by type,
        // so images always render on top of paths within the same Frame.
        // Separate Canvas widgets in a Stack give true z-ordering.
        let background = Canvas::new(BackgroundCanvas {
            state,
            cache: &self.background_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let highlights = Canvas::new(HighlightCanvas {
            state,
            selection: &self.selection,
            cache: &self.highlight_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let stones = Canvas::new(StoneCanvas {
            state,
            selection: &self.selection,
            animations: &self.animations,
            cache: &self.stone_cache,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        Stack::new()
            .push(background)
            .push(highlights)
            .push(stones)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct BackgroundCanvas<'a> {
    state: &'a GameState,
    cache: &'a canvas::Cache,
}

struct HighlightCanvas<'a> {
    state: &'a GameState,
    selection: &'a SelectionState,
    cache: &'a canvas::Cache,
}

struct StoneCanvas<'a> {
    state: &'a GameState,
    selection: &'a SelectionState,
    animations: &'a Vec<RemovalAnimation>,
    cache: &'a canvas::Cache,
}

/// Compute board and cell size to fit board in bounds, reserving space for labels
fn compute_board_layout(board_size: usize, bounds: Rectangle) -> (f32, f32, f32) {
    let available_width = (bounds.width - LABEL_MARGIN).max(0.0);
    let available_height = (bounds.height - LABEL_MARGIN).max(0.0);
    let available = available_width.min(available_height);

    let cell_size = available / board_size as f32;

    let board_pixel_size = board_size as f32 * cell_size + LABEL_MARGIN;
    let offset_x = (bounds.width - board_pixel_size) / 2.0 + LABEL_MARGIN;
    let offset_y = (bounds.height - board_pixel_size) / 2.0;
    (cell_size, offset_x, offset_y)
}

/// Convert board position to screen coordinates
/// Row 0 is at the BOTTOM of the screen, row N-1 is at the TOP
fn board_to_screen(
    pos: Position,
    board_size: usize,
    cell_size: f32,
    offset_x: f32,
    offset_y: f32,
) -> Point {
    let screen_row = (board_size - 1) - pos.row;
    Point::new(
        offset_x + pos.col as f32 * cell_size + cell_size / 2.0,
        offset_y + screen_row as f32 * cell_size + cell_size / 2.0,
    )
}

/// Convert screen coordinates to board position
fn screen_to_board(
    cursor_x: f32,
    cursor_y: f32,
    board_size: usize,
    cell_size: f32,
    offset_x: f32,
    offset_y: f32,
) -> Option<Position> {
    let col = ((cursor_x - offset_x) / cell_size).floor() as isize;
    let screen_row = ((cursor_y - offset_y) / cell_size).floor() as isize;

    if screen_row >= 0 && screen_row < board_size as isize && col >= 0 && col < board_size as isize
    {
        let board_row = (board_size - 1) - screen_row as usize;
        Some(Position::new(board_row, col as usize))
    } else {
        None
    }
}

// BackgroundCanvas: draws background image and labels
impl<'a> canvas::Program<BoardMessage> for BackgroundCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let board_size = self.state.board.size();
        let (cell_size, offset_x, offset_y) = compute_board_layout(board_size, bounds);

        vec![self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_tiled_background_with_labels(board_size, cell_size, offset_x, offset_y, frame);
        })]
    }
}

impl<'a> BackgroundCanvas<'a> {
    fn draw_tiled_background_with_labels(&self, board_size: usize, cell_size: f32, offset_x: f32, offset_y: f32, frame: &mut Frame<Renderer>) {
        let image = Image::new(get_background_tile())
            .filter_method(iced::widget::image::FilterMethod::Linear);

        for row in 0..board_size {
            for col in 0..board_size {
                let pos = Position::new(row, col);
                let center = board_to_screen(pos, board_size, cell_size, offset_x, offset_y);
                let top_left = Point::new(
                    center.x - cell_size / 2.0,
                    center.y - cell_size / 2.0,
                );
                frame.draw_image(Rectangle::new(top_left, Size::new(cell_size, cell_size)), image.clone());
            }
        }

        // Draw labels
        self.draw_labels(board_size, cell_size, offset_x, offset_y, frame);
    }

    fn draw_labels(&self, board_size: usize, cell_size: f32, offset_x: f32, offset_y: f32, frame: &mut Frame<Renderer>) {
        let label_size = 14.0;
        let label_color = Color::from_rgb(0.3, 0.3, 0.3);
        let label_margin = 4.0;

        // Row labels (1, 2, 3... from bottom to top) - left of board
        let row_label_offset_x = offset_x - label_margin - cell_size / 2.0;
        for row in 0..board_size {
            let label = (row + 1).to_string();
            let label_position = board_to_screen(Position::new(row, 0), board_size, cell_size, row_label_offset_x, offset_y);
            frame.fill_text(Text {
                content: label,
                position: label_position,
                color: label_color,
                size: label_size.into(),
                align_x: iced::alignment::Horizontal::Right.into(),
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
        }

        // Column labels (a, b, c...) - below board
        let col_label_offset_y = offset_y + label_margin + cell_size / 2.0;
        for col in 0..board_size {
            let label = ((b'a' + col as u8) as char).to_string();
            let label_position = board_to_screen(Position::new(0, col), board_size, cell_size, offset_x, col_label_offset_y);
            frame.fill_text(Text {
                content: label,
                position: label_position,
                color: label_color,
                size: label_size.into(),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Top,
                ..Text::default()
            });
        }
    }
}

// HighlightCanvas: draws selection highlights (paths only, no images)
impl<'a> canvas::Program<BoardMessage> for HighlightCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let board_size = self.state.board.size();
        let (cell_size, offset_x, offset_y) = compute_board_layout(board_size, bounds);

        vec![self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_highlights(board_size, cell_size, offset_x, offset_y, frame);
        })]
    }
}

impl<'a> HighlightCanvas<'a> {
    fn draw_highlights(&self, board_size: usize, cell_size: f32, offset_x: f32, offset_y: f32, frame: &mut Frame<Renderer>) {
        let hole_radius = cell_size * 0.44;
        let valid_removals = match self.state.phase {
            GamePhase::OpeningBlackRemoval => Rules::valid_black_opening_removals(self.state),
            GamePhase::OpeningWhiteRemoval => Rules::valid_white_opening_removals(self.state),
            _ => Vec::new(),
        };

        let movable_pieces = if matches!(self.state.phase, GamePhase::Play) {
            Rules::movable_pieces(self.state)
        } else {
            Vec::new()
        };

        let (selected_pos, valid_destinations): (Option<Position>, Vec<Position>) =
            match self.selection {
                SelectionState::PieceSelected(pos, jumps) => {
                    let destinations: Vec<Position> = jumps.iter().map(|j| j.to).collect();
                    (Some(*pos), destinations)
                }
                SelectionState::None => (None, Vec::new()),
            };

        for row in 0..board_size {
            for col in 0..board_size {
                let pos = Position::new(row, col);
                let center = board_to_screen(pos, board_size, cell_size, offset_x, offset_y);

                if valid_removals.contains(&pos) {
                    let highlight = Path::circle(center, hole_radius + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                if movable_pieces.contains(&pos) && selected_pos.is_none() {
                    let highlight = Path::circle(center, hole_radius + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.3, 0.7, 1.0))
                            .with_width(2.0),
                    );
                }

                if selected_pos == Some(pos) {
                    let highlight = Path::circle(center, hole_radius + 3.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(1.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                if valid_destinations.contains(&pos) {
                    let highlight = Path::circle(center, hole_radius);
                    frame.fill(&highlight, Color::from_rgba(0.0, 1.0, 0.0, 0.3));
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.0, 0.8, 0.0))
                            .with_width(2.0),
                    );
                }
            }
        }
    }
}

// StoneCanvas: draws stones and handles click events (top layer)
impl<'a> canvas::Program<BoardMessage> for StoneCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let board_size = self.state.board.size();
        let (cell_size, offset_x, offset_y) = compute_board_layout(board_size, bounds);

        vec![self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_animated_stones(board_size, cell_size, offset_x, offset_y, frame);
        })]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<BoardMessage>> {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(cursor_position) = cursor.position_in(bounds)
        {
            let board_size = self.state.board.size();
            let (cell_size, offset_x, offset_y) = compute_board_layout(board_size, bounds);

            if let Some(pos) = screen_to_board(
                cursor_position.x,
                cursor_position.y,
                board_size,
                cell_size,
                offset_x,
                offset_y,
            ) {
                if let SelectionState::PieceSelected(_, jumps) = self.selection {
                    for jump in jumps {
                        if jump.to == pos {
                            return Some(
                                Action::publish(BoardMessage::JumpSelected(jump.clone()))
                                    .and_capture(),
                            );
                        }
                    }
                }

                return Some(Action::publish(BoardMessage::CellClicked(pos)).and_capture());
            }
        }

        None
    }
}

impl<'a> StoneCanvas<'a> {
    fn draw_animated_stones(&self, board_size: usize, cell_size: f32, offset_x: f32, offset_y: f32, frame: &mut Frame<Renderer>) {
        let piece_radius = cell_size * 0.4;
        for row in 0..board_size {
            for col in 0..board_size {
                let pos = Position::new(row, col);
                let is_animating = self.animations.iter().any(|a| a.position == pos);
                if !is_animating
                    && let Some(Cell::Occupied(color)) = self.state.board.get(pos)
                {
                    let center =
                        board_to_screen(pos, board_size, cell_size, offset_x, offset_y);
                    draw_piece(frame, center, piece_radius, color, 1.0);
                }
            }
        }
        for anim in self.animations {
            let center =
                board_to_screen(anim.position, board_size, cell_size, offset_x, offset_y);
            let progress = anim.progress();
            let alpha = 1.0 - progress;
            let scale = 1.0 - (progress * 0.5);
            draw_piece_animated(frame, center, piece_radius, anim.color, alpha, scale);
        }
    }
}

fn draw_piece(frame: &mut Frame, center: Point, piece_radius: f32, color: PieceColor, alpha: f32) {
    draw_piece_animated(frame, center, piece_radius, color, alpha, 1.0);
}

fn draw_piece_animated(frame: &mut Frame, center: Point, radius: f32, color: PieceColor, alpha: f32, scale: f32) {
    let stone_size = radius * 2.0 * scale;
    let half_size = stone_size / 2.0;

    let handle = match color {
        PieceColor::Black => get_black_stone(),
        PieceColor::White => get_white_stone(),
    };

    let image = Image::new(handle)
        .opacity(alpha)
        .filter_method(iced::widget::image::FilterMethod::Linear);

    let top_left = Point::new(center.x - half_size, center.y - half_size);
    frame.draw_image(Rectangle::new(top_left, Size::new(stone_size, stone_size)), image);
}
