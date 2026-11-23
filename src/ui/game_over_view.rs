use std::time::{Duration, Instant};

use iced::widget::canvas::{self, Action, Canvas, Event, Frame, Geometry, Path};
use iced::widget::{Space, button, column, container, opaque, row, stack, text, text_input};
use iced::{Alignment, Element, Length, Point, Rectangle, Renderer, Theme, mouse, window};

use crate::game::{MoveHistory, PieceColor};
use crate::ui::theme;

const MOONRISE: Duration = Duration::from_millis(1000);

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
    pub move_history: MoveHistory,
    pub board_size: usize,
    pub show_export_modal: bool,
    pub export_path: String,
    pub export_format: Option<ExportFormat>,
    pub export_error: Option<String>,
    // When the result appeared, which times the moonrise
    shown_at: Instant,
}

impl GameOverView {
    pub fn new(winner: PieceColor, move_history: MoveHistory, board_size: usize) -> Self {
        Self {
            winner,
            move_history,
            board_size,
            show_export_modal: false,
            export_path: String::new(),
            export_format: None,
            export_error: None,
            shown_at: Instant::now(),
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
            moves: &'a MoveHistory,
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
        let loser = self.winner.opposite();
        let moon = Canvas::new(Moonrise { started: self.shown_at })
            .width(Length::Fill)
            .height(Length::Fixed(124.0));
        let winner_text = text(format!("{} wins", self.winner)).font(theme::DISPLAY).size(32);
        let reason_text = text(format!("{} has no jump left.", loser)).size(16).color(theme::CORAL_300);
        let moves_text = text(format!("{} moves", self.move_history.len()))
            .size(13)
            .color(theme::CORAL_500);

        let dismiss_button = button(
            text("New game")
                .font(theme::LABEL)
                .size(17)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        )
        .padding([10, 20])
        .width(Length::Fill)
        .style(theme::primary_button)
        .on_press(GameOverMessage::Dismiss);

        let save_button = |label: &'static str, format: ExportFormat| {
            button(
                text(label)
                    .font(theme::LABEL)
                    .size(15)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .padding([8, 12])
            .width(Length::Fill)
            .style(theme::secondary_button)
            .on_press(GameOverMessage::ShowExportModal(format))
        };
        let save_row = row![
            save_button("Save this game", ExportFormat::Json),
            save_button("Save the move list", ExportFormat::Text)
        ]
        .spacing(10);

        let content = column![
            moon,
            column![winner_text, reason_text, moves_text]
                .spacing(4)
                .align_x(Alignment::Center),
            Space::new().height(8.0),
            column![dismiss_button, save_row].spacing(10),
        ]
        .spacing(10)
        .align_x(Alignment::Center);

        let popup = container(content).width(Length::Fixed(400.0)).padding(26).style(theme::panel);

        // Scrim that keeps clicks from reaching the board underneath
        let main_view = opaque(
            container(popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(theme::scrim),
        );

        if self.show_export_modal {
            stack![main_view, self.export_modal_view()].into()
        } else {
            main_view
        }
    }

    fn export_modal_view(&self) -> Element<'_, GameOverMessage> {
        let (heading, example) = match self.export_format {
            Some(ExportFormat::Json) => ("Save this game", "konane-game.json"),
            Some(ExportFormat::Text) | None => ("Save the move list", "konane-game.txt"),
        };
        let title = text(heading).font(theme::HEADING).size(22);
        let placeholder = format!("File name, for example {}", example);

        let path_input = text_input(&placeholder, &self.export_path)
            .on_input(GameOverMessage::ExportPathChanged)
            .on_submit(GameOverMessage::ConfirmExport)
            .font(theme::BODY)
            .size(16)
            .padding(10)
            .style(theme::text_input_style);

        let export_btn = button(text("Save").font(theme::LABEL).size(15))
            .padding([8, 18])
            .style(theme::primary_button)
            .on_press(GameOverMessage::ConfirmExport);

        let cancel_btn = button(text("Cancel").font(theme::LABEL).size(15))
            .padding([8, 18])
            .style(theme::secondary_button)
            .on_press(GameOverMessage::CancelExport);

        let buttons = row![Space::new().width(Length::Fill), cancel_btn, export_btn].spacing(10);

        let mut modal_content = column![title, path_input].spacing(16);

        if let Some(error) = &self.export_error {
            modal_content = modal_content.push(text(error.as_str()).size(15).color(theme::EMBER_300));
        }

        let inner_popup = container(modal_content.push(buttons))
            .width(Length::Fixed(420.0))
            .padding(26)
            .style(theme::panel);

        opaque(
            container(inner_popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(theme::scrim),
        )
    }
}

/// A full moon that rises when the result appears
struct Moonrise {
    started: Instant,
}

impl Moonrise {
    fn progress(&self) -> f32 {
        (self.started.elapsed().as_secs_f32() / MOONRISE.as_secs_f32()).min(1.0)
    }
}

impl canvas::Program<GameOverMessage> for Moonrise {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<GameOverMessage>> {
        // Keep redrawing until the moon is up
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            return (self.progress() < 1.0).then(Action::request_redraw);
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        const RADIUS: f32 = 26.0;

        // Ease out, so the moon slows as it settles
        let eased = 1.0 - (1.0 - self.progress()).powi(3);
        let rest_y = bounds.height / 2.0;
        let start_y = bounds.height + RADIUS;
        let center = Point::new(bounds.width / 2.0, start_y + (rest_y - start_y) * eased);

        let mut frame = Frame::new(renderer, bounds.size());
        // Overlapping faint discs build a soft glow
        for ring in (1..=10).rev() {
            let glow = Path::circle(center, RADIUS + ring as f32 * 3.0);
            frame.fill(&glow, theme::with_alpha(theme::MOON, 0.022 * eased));
        }
        frame.fill(&Path::circle(center, RADIUS), theme::MOON);
        vec![frame.into_geometry()]
    }
}
