use iced::{
    widget::canvas::{Frame, Path, Stroke},
    Color, Point,
};

type RawCurve = Vec<Point>;

#[derive(Debug)]
pub struct Curve {
    raw: RawCurve,
    path: Path,
    color: Color,
    width: f32,
}

impl Default for Curve {
    fn default() -> Self {
        Curve::new(vec![])
    }
}

impl Curve {
    pub fn new(raw_curve: Vec<Point>) -> Self {
        Curve {
            path: Self::build_path(&raw_curve),
            raw: raw_curve,
            color: Color::WHITE,
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

    pub fn draw(&self, frame: &mut Frame) {
        frame.stroke(
            &self.path,
            Stroke::default()
                .with_color(self.color)
                .with_width(self.width),
        )
    }

    pub fn create_reduced(&self, factor: usize) -> Self {
        Self::new(self.raw.chunks(factor).map(|chunk| chunk[0]).collect())
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
        Element, Renderer,
    };

    use super::*;

    #[derive(Default, Debug)]
    pub struct State {
        curve: Curve,
        step: usize,
    }
    pub enum Message {}

    impl State {
        pub fn view(&self) -> Element<Message> {
            column![canvas(self), row![text(format!("steps: {}", self.step))]].into()
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
