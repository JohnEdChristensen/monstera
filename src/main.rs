use chilly_canvas::curve::{curve_demo, Curve};
use chilly_canvas::widgets::workspace;
use glam::Vec3;
use iced::mouse::ScrollDelta;
use iced::widget::canvas::Cache;
use iced::widget::{canvas, column, mouse_area};
use iced::{mouse, Length, Vector};
use iced::{Color, Rectangle, Renderer, Theme};
use iced::{Element, Point};

const BG_COLOR: Color = Color {
    r: 0.1,
    g: 0.15,
    b: 0.15,
    a: 1.0,
};

#[allow(dead_code)]
const NEUTRAL_COLOR: Color = Color {
    r: 0.25,
    g: 0.25,
    b: 0.25,
    a: 1.0,
};

pub fn main() -> iced::Result {
    iced::application(":)", update, view)
        .antialiasing(true)
        .run()
}

#[derive(Debug)]
enum Tool {
    #[allow(dead_code)]
    Pan,
    Draw(Curve),
}

#[derive(Debug, Clone)]
enum Message {
    Scroll(ScrollDelta),
    Move(Point),
    Click,
    MouseUp,
    DemoMessage,
}

// First, we define the data we need for drawing
#[derive(Debug)]
struct World {
    curves: Vec<Curve>,
    camera: Vec3,
    tool: Tool,
    cache: Cache,
    curve_demo: curve_demo::State,
}

impl Default for World {
    fn default() -> Self {
        World {
            camera: Vec3::new(0., 0., 300.),
            tool: Tool::Pan,
            curves: vec![],
            cache: Cache::new(),
            curve_demo: curve_demo::State::default(),
        }
    }
}

fn update(world: &mut World, message: Message) {
    match message {
        Message::Scroll(scroll_delta) => {
            world.cache.clear();
            match scroll_delta {
                ScrollDelta::Pixels { x, y } => world.camera += Vec3::new(-x, -y, 0.),
                ScrollDelta::Lines { x, y } => world.camera += Vec3::new(-x * 5., -y * 5., 0.),
            }
        }
        //// Start draw curve
        Message::Click => world.tool = Tool::Draw(Curve::new(vec![])),

        //// Building curve
        #[allow(clippy::single_match)]
        Message::Move(point) => match &mut world.tool {
            Tool::Draw(curve) => {
                curve.push(point + Vector::new(world.camera.x, world.camera.y));
                world.cache.clear();
            }
            _ => {}
        },

        //// Finish curve
        #[allow(clippy::single_match)]
        Message::MouseUp => match &mut world.tool {
            Tool::Draw(curve) => {
                world.curves.push(curve.create_reduced(3));
                world.cache.clear();
                world.tool = Tool::Pan
            }
            _ => (),
        },
        Message::DemoMessage => {}
    }
}

fn view(world: &World) -> Element<Message> {
    //mouse_area(text("hi!")).into()
    //text("hi!").into()
    column!(
        mouse_area(canvas(world).width(Length::Fill).height(Length::Fill))
            .on_scroll(Message::Scroll)
            .on_press(Message::Click)
            .on_release(Message::MouseUp)
            .on_move(Message::Move),
        //workspace::workspace(vec![(
        //    (0., 0.).into(),
        //    world.curve_demo.view().map(|_| Message::DemoMessage)
        //)])
    )
    .into()
}

impl<Message> canvas::Program<Message> for World {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        vec![self.cache.draw(renderer, bounds.size(), |frame| {
            println!("redraw!");
            let camera = self.camera;

            //let mut frame = canvas::Frame::new(renderer, bounds.size());

            ////Coordinates
            //let width = frame.width();
            //let height = frame.height();
            frame.translate(Vector::new(-camera.x, -camera.y));

            //// Background
            frame.fill_rectangle(
                (-100_000., -100_000.).into(),
                (10_000_000., 10_000_000.).into(),
                BG_COLOR,
            );

            //// Foreground

            //// Saved curves
            self.curves.iter().for_each(|v| v.draw(frame));

            //// Current curve
            if let Tool::Draw(curve) = &self.tool {
                curve.draw(frame)
            };
        })]
    }
}
