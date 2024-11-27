use crate::curve::Curve;
use crate::widgets::workspace;
use glam::Vec3;
use iced::widget::canvas::Cache;
use iced::widget::{
    button, column, container, horizontal_space, radio, row, stack, vertical_space,
};
use iced::{Alignment, Length, Vector};
use iced::{Color, Element, Point, Theme};
use style::color_button;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Tool {
    Line,
    Pen(bool),
    Erase(bool),
}

/// unit version of Tool, used for initialize Tool and displaying summarized version of Tool
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SelectedTool {
    Pen,
    Line,
    Erase,
}

impl From<SelectedTool> for Tool {
    fn from(val: SelectedTool) -> Self {
        match val {
            SelectedTool::Line => Tool::Line,
            SelectedTool::Pen => Tool::Pen(false),
            SelectedTool::Erase => Tool::Erase(false),
        }
    }
}
impl From<Tool> for SelectedTool {
    fn from(val: Tool) -> Self {
        match val {
            Tool::Line => SelectedTool::Line,
            Tool::Pen(_) => SelectedTool::Pen,
            Tool::Erase(_) => SelectedTool::Erase,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Pan(Vector),
    Zoom(f32),
    Move(Point),
    MouseUp(Point),
    MouseDown(Point),
    DemoMessage,
    SetTool(SelectedTool),
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
            tool: Tool::Pen(false),
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
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Pan(delta) => {
                self.camera += Vec3::new(-delta.x * 1.5, -delta.y * 1.5, 0.);
                self.cache.clear();
            }
            Message::Zoom(delta) => {
                self.camera += Vec3::new(0., 0., delta);
                self.cache.clear();
            }

            //// Building curve
            #[allow(clippy::single_match)]
            Message::Move(point) => match &mut self.tool {
                Tool::Pen(true) => {
                    self.curves
                        .last_mut()
                        .unwrap()
                        .push(point + Vector::new(self.camera.x, self.camera.y));
                    self.cache.clear();
                }
                _ => {}
            },

            //// Finish curve
            #[allow(clippy::single_match)]
            Message::MouseUp(_point) => match &mut self.tool {
                Tool::Pen(true) => {
                    if let Some(curve) = self.curves.last_mut() {
                        *curve = curve.create_reduced(3)
                    }
                    self.cache.clear();
                    self.tool = Tool::Pen(false)
                }
                Tool::Erase(true) => self.tool = Tool::Erase(false),
                _ => {}
            },
            #[allow(clippy::single_match)]
            Message::MouseDown(_point) => match &mut self.tool {
                Tool::Pen(false) => {
                    self.tool = Tool::Pen(true);
                    self.curves.push(Curve::new(vec![], self.active_color));
                }

                Tool::Erase(false) => self.tool = Tool::Erase(true),
                _ => (),
            },
            Message::SetTool(tool) => {
                //#[allow(clippy::single_match)]
                //match tool {
                //    Tool::Pen(false) =>                     _ => {}
                //}
                self.tool = tool.into();
            }
            Message::SetColor(color) => self.active_color = color,
            Message::Clear => {
                self.curves = vec![];
                self.cache.clear();
            }
            Message::DemoMessage => {}
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
        .on_press(Message::MouseDown)
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

        let pen = radio(
            "Pen",
            SelectedTool::Pen,
            Some(self.tool.into()),
            Message::SetTool,
        );
        let erase = radio(
            "Erase",
            SelectedTool::Erase,
            Some(self.tool.into()),
            Message::SetTool,
        );

        let tools = column!(
            row!(pen, erase).spacing(10.),
            button("Clear").on_press(Message::Clear)
        )
        .spacing(10.)
        .padding(10.);
        //.wrap();

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
