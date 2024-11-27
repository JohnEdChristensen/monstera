use iced::{theme::palette, Theme};
use monstera::world::World;

pub fn main() -> iced::Result {
    let palette = palette::Palette {
        background: [0.1, 0.15, 0.15, 1.0].into(),
        primary: [0.4, 0.7, 0.5, 1.0].into(),
        text: [0.95, 0.9, 0.9, 1.0].into(),
        success: [0.5, 0.6, 0.8, 1.0].into(),
        danger: [0.9, 0.8, 0.6, 1.0].into(),
    };
    let theme = Theme::custom("my_theme".into(), palette);

    iced::application(":)", World::update, World::view)
        .antialiasing(true)
        .theme(move |_| theme.clone())
        .run()
}
