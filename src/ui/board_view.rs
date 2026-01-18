use std::time::Instant;

use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Event, Frame, Geometry, Image, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::game::rules::Jump;
use crate::game::{Cell, GamePhase, GameState, PieceColor, Position, Rules};

static BLACK_STONE_PATH: &str = "data/black-stone-15.png";
static WHITE_STONE_PATH: &str = "data/white-stone-15.png";

const PADDING: f32 = 20.0;
const ANIMATION_DURATION_MS: u64 = 300;

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
}

impl Default for BoardView {
    fn default() -> Self {
        Self {
            selection: SelectionState::None,
            animations: Vec::new(),
        }
    }
}

impl BoardView {
    pub fn select_piece(&mut self, pos: Position, jumps: Vec<Jump>) {
        self.selection = SelectionState::PieceSelected(pos, jumps);
    }

    pub fn clear_selection(&mut self) {
        self.selection = SelectionState::None;
    }

    pub fn _selection(&self) -> &SelectionState {
        &self.selection
    }

    /// Start an animation for a removed piece
    pub fn animate_removal(&mut self, position: Position, color: PieceColor) {
        self.animations.push(RemovalAnimation::new(position, color));
    }

    /// Update animations and remove completed ones
    pub fn update_animations(&mut self) {
        self.animations.retain(|anim| !anim.is_complete());
    }

    /// Check if any animations are running
    pub fn has_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    pub fn view<'a>(&'a self, state: &'a GameState) -> Element<'a, BoardMessage> {
        Canvas::new(BoardCanvas {
            state,
            selection: &self.selection,
            animations: &self.animations,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

struct BoardCanvas<'a> {
    state: &'a GameState,
    selection: &'a SelectionState,
    animations: &'a Vec<RemovalAnimation>,
}

/// Compute cell size to fit board in bounds
fn compute_cell_size(board_size: usize, bounds: Rectangle) -> f32 {
    let available = (bounds.width.min(bounds.height) - PADDING * 2.0).max(0.0);
    available / board_size as f32
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

impl<'a> canvas::Program<BoardMessage> for BoardCanvas<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let board_color = Color::from_rgb(0.2, 0.18, 0.15);
        let hole_color = Color::from_rgb(0.18, 0.16, 0.13);

        // Clear entire canvas
        let canvas_bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&canvas_bg, board_color);

        let board_size = self.state.board.size();
        let cell_size = compute_cell_size(board_size, bounds);
        let board_pixel_size = board_size as f32 * cell_size;
        let piece_radius = cell_size * 0.4;
        let hole_radius = cell_size * 0.44;

        // Center the board
        let offset_x = (bounds.width - board_pixel_size) / 2.0;
        let offset_y = (bounds.height - board_pixel_size) / 2.0;

        // Get valid positions for highlighting
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

        // Draw cells and pieces
        for row in 0..board_size {
            for col in 0..board_size {
                let pos = Position::new(row, col);
                let center = board_to_screen(pos, board_size, cell_size, offset_x, offset_y);

                // Draw hole (indentation)
                let hole = Path::circle(center, hole_radius);
                frame.fill(&hole, hole_color);

                // Highlight valid removal positions
                if valid_removals.contains(&pos) {
                    let highlight = Path::circle(center, hole_radius + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                // Highlight movable pieces
                if movable_pieces.contains(&pos) && selected_pos.is_none() {
                    let highlight = Path::circle(center, hole_radius + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.3, 0.7, 1.0))
                            .with_width(2.0),
                    );
                }

                // Highlight selected piece
                if selected_pos == Some(pos) {
                    let highlight = Path::circle(center, hole_radius + 3.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(1.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                // Highlight valid destinations
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

                // Draw piece if present (and not being animated away)
                let is_animating = self.animations.iter().any(|a| a.position == pos);
                if !is_animating && let Some(Cell::Occupied(color)) = self.state.board.get(pos) {
                    draw_piece(&mut frame, center, piece_radius, color, 1.0);
                }
            }
        }

        // Draw animating pieces (fading out and shrinking)
        for anim in self.animations {
            let center = board_to_screen(anim.position, board_size, cell_size, offset_x, offset_y);
            let progress = anim.progress();
            let alpha = 1.0 - progress;
            let scale = 1.0 - (progress * 0.5);
            draw_piece_animated(&mut frame, center, piece_radius, anim.color, alpha, scale);
        }

        vec![frame.into_geometry()]
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
            let cell_size = compute_cell_size(board_size, bounds);
            let board_pixel_size = board_size as f32 * cell_size;
            let offset_x = (bounds.width - board_pixel_size) / 2.0;
            let offset_y = (bounds.height - board_pixel_size) / 2.0;

            if let Some(pos) = screen_to_board(
                cursor_position.x,
                cursor_position.y,
                board_size,
                cell_size,
                offset_x,
                offset_y,
            ) {
                // If we have a selected piece and clicked a valid destination, select the jump
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

fn draw_piece(frame: &mut Frame, center: Point, piece_radius: f32, color: PieceColor, alpha: f32) {
    draw_piece_animated(frame, center, piece_radius, color, alpha, 1.0);
}

fn draw_piece_animated(
    frame: &mut Frame,
    center: Point,
    piece_radius: f32,
    color: PieceColor,
    alpha: f32,
    scale: f32,
) {
    let stone_size = piece_radius * 2.0 * scale;
    let half_size = stone_size / 2.0;

    let path = match color {
        PieceColor::Black => BLACK_STONE_PATH,
        PieceColor::White => WHITE_STONE_PATH,
    };

    let handle = iced::widget::image::Handle::from_path(path);
    let image = Image::new(handle)
        .opacity(alpha)
        .filter_method(iced::widget::image::FilterMethod::Linear);

    let top_left = Point::new(center.x - half_size, center.y - half_size);
    frame.draw_image(Rectangle::new(top_left, Size::new(stone_size, stone_size)), image);
}
