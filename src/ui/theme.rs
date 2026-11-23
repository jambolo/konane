use iced::font::{Family, Weight};
use iced::theme::Palette;
use iced::widget::canvas::{self, Path, Stroke};
use iced::widget::{button, container, overlay, pick_list, radio, scrollable, text_input};
use iced::{Background, Border, Color, Font, Point, Rectangle, Renderer, Shadow, Theme, Vector, mouse};

// Brand palette, documented in docs/branding/visual.md
pub const BASALT_900: Color = Color::from_rgb8(0x11, 0x12, 0x13);
pub const BASALT_800: Color = Color::from_rgb8(0x1b, 0x1c, 0x1e);
pub const BASALT_700: Color = Color::from_rgb8(0x27, 0x28, 0x2b);
pub const BASALT_600: Color = Color::from_rgb8(0x38, 0x3a, 0x3e);
pub const CORAL_500: Color = Color::from_rgb8(0x9c, 0x91, 0x8b);
pub const CORAL_300: Color = Color::from_rgb8(0xd3, 0xca, 0xc0);
pub const CORAL_100: Color = Color::from_rgb8(0xf1, 0xee, 0xeb);
pub const TURMERIC: Color = Color::from_rgb8(0xe2, 0xa5, 0x2b);
pub const SEA: Color = Color::from_rgb8(0x3f, 0xb5, 0xac);
pub const MOON: Color = Color::from_rgb8(0xc9, 0xdb, 0xe6);
pub const EMBER: Color = Color::from_rgb8(0xc4, 0x58, 0x3f);
pub const EMBER_300: Color = Color::from_rgb8(0xe8, 0x91, 0x78);

const TURMERIC_HOVER: Color = Color::from_rgb8(0xec, 0xb7, 0x4c);
const TURMERIC_PRESSED: Color = Color::from_rgb8(0xc9, 0x90, 0x1f);

// Fonts are embedded so the app looks the same on every machine
pub const FONT_FILES: [&[u8]; 4] = [
    include_bytes!("../../data/fonts/SourceSans3-Regular.ttf"),
    include_bytes!("../../data/fonts/SourceSans3-SemiBold.ttf"),
    include_bytes!("../../data/fonts/Lora-Medium.ttf"),
    include_bytes!("../../data/fonts/Lora-SemiBold.ttf"),
];

pub const BODY: Font = Font::with_name("Source Sans 3");
pub const LABEL: Font = Font {
    family: Family::Name("Source Sans 3"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};
pub const HEADING: Font = Font {
    family: Family::Name("Lora"),
    weight: Weight::Medium,
    ..Font::DEFAULT
};
pub const DISPLAY: Font = Font {
    family: Family::Name("Lora"),
    weight: Weight::Semibold,
    ..Font::DEFAULT
};

pub const RADIUS_CONTROL: f32 = 6.0;
pub const RADIUS_PANEL: f32 = 12.0;

pub fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

pub fn theme() -> Theme {
    Theme::custom(
        "Kōnane",
        Palette {
            background: BASALT_800,
            text: CORAL_100,
            primary: TURMERIC,
            success: SEA,
            warning: TURMERIC,
            danger: EMBER,
        },
    )
}

/// The one action the player should take
pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color) = match status {
        button::Status::Active => (TURMERIC, BASALT_800),
        button::Status::Hovered => (TURMERIC_HOVER, BASALT_800),
        button::Status::Pressed => (TURMERIC_PRESSED, BASALT_800),
        button::Status::Disabled => (BASALT_600, CORAL_500),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            radius: RADIUS_CONTROL.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

/// Every other action
pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let (background, text_color, border_color) = match status {
        button::Status::Active => (BASALT_700, CORAL_100, CORAL_500),
        button::Status::Hovered => (BASALT_600, CORAL_100, CORAL_300),
        button::Status::Pressed => (BASALT_900, CORAL_100, CORAL_300),
        button::Status::Disabled => (BASALT_800, with_alpha(CORAL_500, 0.6), BASALT_600),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..button::Style::default()
    }
}

/// A text-only action
pub fn link_button(_theme: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Active => CORAL_300,
        button::Status::Hovered | button::Status::Pressed => CORAL_100,
        button::Status::Disabled => CORAL_500,
    };
    button::Style {
        background: None,
        text_color,
        ..button::Style::default()
    }
}

pub fn radio_style(_theme: &Theme, status: radio::Status) -> radio::Style {
    let (is_selected, hovered) = match status {
        radio::Status::Active { is_selected } => (is_selected, false),
        radio::Status::Hovered { is_selected } => (is_selected, true),
    };
    radio::Style {
        background: Background::Color(BASALT_900),
        dot_color: TURMERIC,
        border_width: if is_selected { 2.0 } else { 1.0 },
        border_color: if is_selected {
            TURMERIC
        } else if hovered {
            CORAL_300
        } else {
            CORAL_500
        },
        text_color: Some(if is_selected || hovered { CORAL_100 } else { CORAL_300 }),
    }
}

pub fn pick_list_style(_theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let border_color = match status {
        pick_list::Status::Active => CORAL_500,
        pick_list::Status::Hovered => CORAL_300,
        pick_list::Status::Opened { .. } => TURMERIC,
    };
    pick_list::Style {
        text_color: CORAL_100,
        placeholder_color: CORAL_500,
        handle_color: CORAL_300,
        background: Background::Color(BASALT_900),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
    }
}

pub fn menu_style(_theme: &Theme) -> overlay::menu::Style {
    overlay::menu::Style {
        background: Background::Color(BASALT_700),
        border: Border {
            color: CORAL_500,
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        text_color: CORAL_100,
        selected_text_color: BASALT_800,
        selected_background: Background::Color(TURMERIC),
        shadow: panel_shadow(),
    }
}

pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => CORAL_500,
        text_input::Status::Hovered => CORAL_300,
        text_input::Status::Focused { .. } => TURMERIC,
        text_input::Status::Disabled => BASALT_600,
    };
    text_input::Style {
        background: Background::Color(BASALT_900),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        icon: CORAL_300,
        placeholder: CORAL_500,
        value: CORAL_100,
        selection: with_alpha(TURMERIC, 0.35),
    }
}

pub fn scrollable_style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let rail = |mut rail: scrollable::Rail| {
        rail.background = None;
        rail.scroller.background = Background::Color(BASALT_600);
        rail.scroller.border.radius = 4.0.into();
        rail
    };
    style.vertical_rail = rail(style.vertical_rail);
    style.horizontal_rail = rail(style.horizontal_rail);
    style
}

fn panel_shadow() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        offset: Vector::new(0.0, 6.0),
        blur_radius: 24.0,
    }
}

/// A raised surface: dialogs, the setup card
pub fn panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BASALT_700)),
        text_color: Some(CORAL_100),
        border: Border {
            color: BASALT_600,
            width: 1.0,
            radius: RADIUS_PANEL.into(),
        },
        shadow: panel_shadow(),
        ..container::Style::default()
    }
}

/// A flat surface that sits beside the board
pub fn side_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BASALT_700)),
        text_color: Some(CORAL_100),
        border: Border {
            color: BASALT_600,
            width: 1.0,
            radius: RADIUS_PANEL.into(),
        },
        ..container::Style::default()
    }
}

/// Dims whatever is behind a dialog
pub fn scrim(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(with_alpha(BASALT_900, 0.6))),
        ..container::Style::default()
    }
}

/// The board's opening position, repeated at low contrast behind a screen
pub struct Pattern;

impl<Message> canvas::Program<Message> for Pattern {
    type State = canvas::Cache;

    fn draw(
        &self,
        cache: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        const PITCH: f32 = 36.0;
        const RADIUS: f32 = 11.0;

        vec![cache.draw(renderer, bounds.size(), |frame| {
            let cols = (bounds.width / PITCH).ceil() as usize + 1;
            let rows = (bounds.height / PITCH).ceil() as usize + 1;
            // Centered so the pattern stays symmetric as the window resizes
            let origin_x = (bounds.width - cols as f32 * PITCH) / 2.0 + PITCH / 2.0;
            let origin_y = (bounds.height - rows as f32 * PITCH) / 2.0 + PITCH / 2.0;
            for row in 0..rows {
                for col in 0..cols {
                    let center = Point::new(origin_x + col as f32 * PITCH, origin_y + row as f32 * PITCH);
                    // Fades out toward the middle, where the screen's text sits
                    let distance = (center.x - bounds.width / 2.0).abs();
                    let strength = ((distance - 210.0) / 200.0).clamp(0.0, 1.0);
                    if strength == 0.0 {
                        continue;
                    }
                    let color = if (row + col) % 2 == 0 {
                        with_alpha(BASALT_900, 0.8 * strength)
                    } else {
                        with_alpha(CORAL_100, 0.05 * strength)
                    };
                    frame.fill(&Path::circle(center, RADIUS), color);
                }
            }
        })]
    }
}

/// The brand mark: four lua after Black's opening removal
pub struct Mark;

impl<Message> canvas::Program<Message> for Mark {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        // Drawn on a 64-unit grid, as in docs/branding/assets/mark.svg
        let unit = bounds.width.min(bounds.height) / 64.0;
        let at = |x: f32, y: f32| Point::new(x * unit, y * unit);
        frame.fill(&Path::circle(at(19.0, 19.0), 10.0 * unit), CORAL_100);
        frame.fill(&Path::circle(at(45.0, 45.0), 10.0 * unit), CORAL_100);
        let black = Path::circle(at(19.0, 45.0), 9.0 * unit);
        frame.fill(&black, Color::from_rgb8(0x08, 0x09, 0x0a));
        frame.stroke(
            &black,
            Stroke::default()
                .with_color(Color::from_rgb8(0x7a, 0x7d, 0x83))
                .with_width(2.0 * unit),
        );
        frame.stroke(
            &Path::circle(at(45.0, 19.0), 8.5 * unit),
            Stroke::default().with_color(TURMERIC).with_width(3.0 * unit),
        );
        vec![frame.into_geometry()]
    }
}
