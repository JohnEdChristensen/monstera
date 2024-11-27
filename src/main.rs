use chilly_canvas::world::World;
use iced::{theme::palette, Theme};
//use chilly_canvas::{ wayland::WaylandConnection};
//use wayland_client::Connection;

pub fn main() -> iced::Result {
    //let conn = Connection::connect_to_env().unwrap();
    //
    //let mut event_queue = conn.new_event_queue();
    //let qhandle = event_queue.handle();
    //
    //let display = conn.display();
    //display.get_registry(&qhandle, ());
    //
    //let mut state = WaylandConnection::new();
    //println!("Starting the example window app, press <ESC> to quit.");
    //
    //while state.running {
    //    event_queue.blocking_dispatch(&mut state).unwrap();
    //}

    //// Iced
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
