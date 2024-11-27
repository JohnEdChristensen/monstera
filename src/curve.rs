use iced::{
    advanced::graphics::geometry,
    widget::canvas::{Frame, Path, Stroke},
    Color, Point,
};

type RawCurve = Vec<Point>;

#[derive(Debug, Clone)]
pub struct Curve {
    raw: RawCurve,
    path: Path,
    color: Color,
    width: f32,
}

impl Default for Curve {
    fn default() -> Self {
        Curve::new(vec![], Color::WHITE)
    }
}

impl Curve {
    pub fn new(raw_curve: Vec<Point>, color: Color) -> Self {
        Curve {
            path: Self::build_path(&raw_curve),
            raw: raw_curve,
            color,
            width: 2.0,
        }
    }

    fn build_path(raw: &RawCurve) -> Path {
        Path::new(|builder| {
            raw.iter().enumerate().for_each(|(i, &p)| match i {
                0 => builder.move_to(p),
                _ => builder.line_to(p),
            })
        })
    }

    pub fn draw<Renderer>(&self, frame: &mut Frame<Renderer>)
    where
        Renderer: geometry::Renderer,
    {
        frame.stroke(
            &self.path,
            Stroke::default()
                .with_color(self.color)
                .with_width(self.width),
        )
    }

    pub fn create_reduced(&self, factor: usize) -> Self {
        Self::new(
            self.raw.chunks(factor).map(|chunk| chunk[0]).collect(),
            self.color,
        )
    }

    pub fn push(&mut self, point: Point) {
        self.raw.push(point);
        self.path = Self::build_path(&self.raw)
    }
}

pub mod curve_demo {

    use iced::{
        mouse,
        widget::{canvas, column, row, text},
        Element, Renderer, Vector,
    };

    use super::*;

    #[derive(Default, Debug)]
    enum Action {
        #[default]
        Idle,
        Drag(Option<Vector>),
    }

    #[derive(Default, Debug)]
    pub struct State {
        position: Point,
        action: Action,

        curve: Curve,
        step: usize,
    }

    #[derive(Clone)]
    pub enum Message {
        Click,
        Move(Point),
        Release,
    }

    impl State {
        pub fn view(&self) -> (Point, Element<Message>) {
            (
                self.position,
                iced::widget::mouse_area(column![
                    canvas(self),
                    row![text(format!("steps: {}", self.step))]
                ])
                .on_press(Message::Click)
                .on_move(Message::Move)
                .on_release(Message::Release)
                .into(),
            )
        }
        pub fn update(&mut self, message: Message) {
            match message {
                Message::Click => self.action = Action::Drag(None),
                Message::Move(point) => match self.action {
                    Action::Idle => (),
                    Action::Drag(None) => self.action = Action::Drag(Some(self.position - point)),
                    Action::Drag(Some(offset)) => self.position = point - offset,
                },
                Message::Release => self.action = Action::Idle,
            }
        }
    }

    impl<Message> canvas::Program<Message> for State {
        type State = ();

        fn draw(
            &self,
            _state: &Self::State,
            renderer: &Renderer,
            _theme: &iced::Theme,
            bounds: iced::Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<canvas::Geometry<Renderer>> {
            let mut frame = canvas::Frame::new(renderer, bounds.size());

            self.curve.draw(&mut frame);
            vec![frame.into_geometry()]
        }
    }
}
