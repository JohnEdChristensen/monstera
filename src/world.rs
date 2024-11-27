use crate::curve::Curve;
use crate::widgets::workspace;
use glam::Vec3;
use iced::widget::canvas::Cache;
use iced::widget::{button, column, container, horizontal_space, row, stack, vertical_space};
use iced::{Alignment, Length, Vector};
use iced::{Color, Element, Point, Theme};
use style::color_button;

#[derive(Debug, Clone)]
pub enum Tool {
    Pan,
    Draw,
    Erase,
}

#[derive(Debug, Clone)]
pub enum Message {
    Pan(Vector),
    Zoom(f32),
    Move(Point),
    MouseUp,
    DemoMessage,
    SetTool(Tool),
    SetColor(Color),
    Clear,
}

#[derive(Debug)]
pub struct World {
    curves: Vec<Curve>,
    camera: Vec3,
    tool: Tool,
    cache: Cache,
    active_color: Color,
    colors: Vec<Color>, //curve_demo: curve_demo::State,
}
const L: f32 = 0.2;
const H: f32 = 0.5;

impl Default for World {
    fn default() -> Self {
        World {
            camera: Vec3::new(0., 0., 300.),
            tool: Tool::Pan,
            curves: vec![],
            cache: Cache::new(),
            colors: vec![
                Color::from_linear_rgba(0.8, 0.8, 0.8, 1.0),
                Color::from_linear_rgba(0.5, 0.5, 0.5, 1.0),
                Color::from_linear_rgba(H, L, L, 1.0),
                Color::from_linear_rgba(H - 0.1, H - 0.2, L - 0.1, 1.0),
                Color::from_linear_rgba(L, H, L, 1.0),
                Color::from_linear_rgba(L, H, H, 1.0),
                Color::from_linear_rgba(L, L, H, 1.0),
            ], //curve_demo: curve_demo::State::default(),
            active_color: Color::from_linear_rgba(0.8, 0.8, 0.8, 1.0),
        }
    }
}
impl World {
    pub fn update(world: &mut World, message: Message) {
        match message {
            Message::Pan(delta) => {
                world.camera += Vec3::new(-delta.x * 1.5, -delta.y * 1.5, 0.);
                world.cache.clear();
            }
            Message::Zoom(delta) => {
                world.camera += Vec3::new(0., 0., delta);
                world.cache.clear();
            }

            //// Building curve
            #[allow(clippy::single_match)]
            Message::Move(point) => match &mut world.tool {
                Tool::Draw => {
                    world
                        .curves
                        .last_mut()
                        .unwrap()
                        .push(point + Vector::new(world.camera.x, world.camera.y));
                    world.cache.clear();
                }
                _ => {}
            },

            //// Finish curve
            #[allow(clippy::single_match)]
            Message::MouseUp => match &mut world.tool {
                Tool::Draw => {
                    if let Some(curve) = world.curves.last_mut() {
                        *curve = curve.create_reduced(3)
                    }
                    world.cache.clear();
                    world.tool = Tool::Pan
                }
                _ => world.cache.clear(),
            },
            Message::DemoMessage => {}
            Message::SetTool(tool) => {
                #[allow(clippy::single_match)]
                match tool {
                    Tool::Draw => world.curves.push(Curve::new(vec![], world.active_color)),
                    _ => {}
                }
                world.tool = tool;
            }
            Message::SetColor(color) => world.active_color = color,
            Message::Clear => {
                world.curves = vec![];
                world.cache.clear();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        //mouse_area(text("hi!")).into()
        //text("hi!").into()
        //let demo = self.curve_demo.view();
        //
        let workspace = workspace::workspace(
            &self.camera,
            &self.curves,
            //vec![(demo.0, demo.1.map(|_| Message::DemoMessage))],
            vec![],
            &self.cache,
        )
        .pan(Message::Pan)
        .on_press(Message::SetTool(Tool::Draw))
        .on_release(Message::MouseUp)
        .on_move(Message::Move);

        let color_buttons = self.colors.iter().map(|c| {
            button("")
                .style(|_: &Theme, s| color_button(*c, s))
                .on_press(Message::SetColor(*c))
                .width(20.)
                .height(20.)
                .into()
        });

        let tools = row!(
            button("Erase").on_press(Message::SetTool(Tool::Erase)),
            button("Pen").on_press(Message::SetTool(Tool::Draw)),
            button("Clear").on_press(Message::Clear)
        )
        .spacing(10.)
        .padding(10.)
        .wrap();

        let content: Element<Message> = stack!(
            workspace,
            column!(row!(
                horizontal_space(),
                column!(
                    row(color_buttons).spacing(5.0).padding(10.),
                    container(tools).center_x(Length::Fill),
                )
                .width(Length::Shrink)
            )
            .align_y(Alignment::Center),)
            .padding(5.0),
            vertical_space()
        )
        .into();
        content
        //.explain(Color::BLACK)
    }
}

mod style {
    use iced::{
        border,
        widget::button::{Status, Style},
        Background,
    };

    /// A primary button; denoting a main action.
    pub fn color_button(color: iced::Color, status: Status) -> Style {
        let base = iced::widget::button::Style {
            background: Some(Background::Color(color)),
            text_color: iced::Color::TRANSPARENT,
            border: border::rounded(2),
            ..iced::widget::button::Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                border: base
                    .border
                    .color(iced::Color::BLACK.scale_alpha(0.1))
                    .width(0.5),
                ..base
            },
            Status::Disabled => Style {
                background: base.background.map(|b| b.scale_alpha(0.5)),
                ..base
            },
        }
    }
}
