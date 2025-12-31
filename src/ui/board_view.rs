use std::time::Instant;

use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Event, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::game::rules::Jump;
use crate::game::{Cell, GamePhase, GameState, PieceColor, Position, Rules};

const CELL_SIZE: f32 = 50.0;
const PIECE_RADIUS: f32 = 20.0;
const HOLE_RADIUS: f32 = 22.0;
const SHADOW_OFFSET: f32 = 3.0;
const ANIMATION_DURATION_MS: u64 = 300;

#[derive(Debug, Clone)]
pub enum BoardMessage {
    CellClicked(Position),
    JumpSelected(Jump),
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

/// Convert board position to screen coordinates
/// Row 0 is at the BOTTOM of the screen, row N-1 is at the TOP
fn board_to_screen(pos: Position, board_size: usize, offset_x: f32, offset_y: f32) -> Point {
    // Flip the row: screen_row = (board_size - 1) - board_row
    let screen_row = (board_size - 1) - pos.row;
    Point::new(
        offset_x + pos.col as f32 * CELL_SIZE + CELL_SIZE / 2.0,
        offset_y + screen_row as f32 * CELL_SIZE + CELL_SIZE / 2.0,
    )
}

/// Convert screen coordinates to board position
fn screen_to_board(
    cursor_x: f32,
    cursor_y: f32,
    board_size: usize,
    offset_x: f32,
    offset_y: f32,
) -> Option<Position> {
    let col = ((cursor_x - offset_x) / CELL_SIZE).floor() as isize;
    let screen_row = ((cursor_y - offset_y) / CELL_SIZE).floor() as isize;

    if screen_row >= 0 && screen_row < board_size as isize && col >= 0 && col < board_size as isize
    {
        // Convert screen row back to board row
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

        let board_size = self.state.board.size();
        let board_pixel_size = board_size as f32 * CELL_SIZE;

        // Center the board
        let offset_x = (bounds.width - board_pixel_size) / 2.0;
        let offset_y = (bounds.height - board_pixel_size) / 2.0;

        // Draw board background (lava rock color)
        let board_bg = Path::rectangle(
            Point::new(offset_x, offset_y),
            Size::new(board_pixel_size, board_pixel_size),
        );
        frame.fill(&board_bg, Color::from_rgb(0.2, 0.18, 0.15));

        // Draw grid lines (carved grooves)
        let groove_color = Color::from_rgb(0.15, 0.13, 0.1);
        for i in 0..=board_size {
            let pos = i as f32 * CELL_SIZE;
            // Horizontal lines
            frame.stroke(
                &Path::line(
                    Point::new(offset_x, offset_y + pos),
                    Point::new(offset_x + board_pixel_size, offset_y + pos),
                ),
                Stroke::default().with_color(groove_color).with_width(1.0),
            );
            // Vertical lines
            frame.stroke(
                &Path::line(
                    Point::new(offset_x + pos, offset_y),
                    Point::new(offset_x + pos, offset_y + board_pixel_size),
                ),
                Stroke::default().with_color(groove_color).with_width(1.0),
            );
        }

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
                let center = board_to_screen(pos, board_size, offset_x, offset_y);

                // Draw hole (indentation)
                let hole = Path::circle(center, HOLE_RADIUS);
                frame.fill(&hole, Color::from_rgb(0.12, 0.1, 0.08));

                // Highlight valid removal positions
                if valid_removals.contains(&pos) {
                    let highlight = Path::circle(center, HOLE_RADIUS + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                // Highlight movable pieces
                if movable_pieces.contains(&pos) && selected_pos.is_none() {
                    let highlight = Path::circle(center, HOLE_RADIUS + 2.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(0.3, 0.7, 1.0))
                            .with_width(2.0),
                    );
                }

                // Highlight selected piece
                if selected_pos == Some(pos) {
                    let highlight = Path::circle(center, HOLE_RADIUS + 3.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(Color::from_rgb(1.0, 0.8, 0.0))
                            .with_width(3.0),
                    );
                }

                // Highlight valid destinations
                if valid_destinations.contains(&pos) {
                    let highlight = Path::circle(center, HOLE_RADIUS);
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
                    draw_piece(&mut frame, center, color, 1.0);
                }
            }
        }

        // Draw animating pieces (fading out and shrinking)
        for anim in self.animations {
            let center = board_to_screen(anim.position, board_size, offset_x, offset_y);
            let progress = anim.progress();
            let alpha = 1.0 - progress;
            let scale = 1.0 - (progress * 0.5); // Shrink to 50% size
            draw_piece_animated(&mut frame, center, anim.color, alpha, scale);
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
            let board_pixel_size = board_size as f32 * CELL_SIZE;
            let offset_x = (bounds.width - board_pixel_size) / 2.0;
            let offset_y = (bounds.height - board_pixel_size) / 2.0;

            if let Some(pos) = screen_to_board(
                cursor_position.x,
                cursor_position.y,
                board_size,
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

fn draw_piece(frame: &mut Frame, center: Point, color: PieceColor, alpha: f32) {
    draw_piece_animated(frame, center, color, alpha, 1.0);
}

fn draw_piece_animated(
    frame: &mut Frame,
    center: Point,
    color: PieceColor,
    alpha: f32,
    scale: f32,
) {
    let radius = PIECE_RADIUS * scale;

    // Shadow
    let shadow_offset = SHADOW_OFFSET * scale;
    let shadow_center = Point::new(center.x + shadow_offset, center.y + shadow_offset);
    let shadow = Path::circle(shadow_center, radius);
    frame.fill(&shadow, Color::from_rgba(0.0, 0.0, 0.0, 0.4 * alpha));

    // Piece
    let piece = Path::circle(center, radius);
    let piece_color = match color {
        PieceColor::Black => Color::from_rgba(0.1, 0.1, 0.1, alpha),
        PieceColor::White => Color::from_rgba(0.95, 0.93, 0.88, alpha),
    };
    frame.fill(&piece, piece_color);

    // Highlight on piece
    let highlight_offset = 5.0 * scale;
    let highlight_center = Point::new(center.x - highlight_offset, center.y - highlight_offset);
    let highlight = Path::circle(highlight_center, 5.0 * scale);
    let highlight_color = match color {
        PieceColor::Black => Color::from_rgba(1.0, 1.0, 1.0, 0.15 * alpha),
        PieceColor::White => Color::from_rgba(1.0, 1.0, 1.0, 0.5 * alpha),
    };
    frame.fill(&highlight, highlight_color);
}
