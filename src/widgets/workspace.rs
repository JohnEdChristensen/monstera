//! This example showcases a simple native custom widget that draws a circle.
use glam::Vec3;
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::mouse::Event::{ButtonPressed, ButtonReleased, CursorMoved, WheelScrolled};
use iced::mouse::ScrollDelta;
use iced::touch::Event::{FingerLifted, FingerLost, FingerMoved, FingerPressed};
use iced::widget::canvas::Cache;
use iced::{event, keyboard, mouse, Color, Point, Theme, Vector};
use iced::{Element, Event};
use iced::{Length, Rectangle, Size};

use crate::curve::Curve;

/// A workspace is a an infinite canvas that can be zoomed, panned,
/// and contains widgets that can be placed anywhere in 3d (stacking in Z)
pub struct Workspace<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::graphics::geometry::Renderer,
{
    camera: &'a Vec3,
    elements: Vec<(Point, Element<'a, Message, Theme, Renderer>)>,
    primitives: &'a Vec<Curve>,
    cache: &'a Cache<Renderer>,
    pan: Option<Box<dyn Fn(Vector) -> Message + 'a>>,
    zoom: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    on_press: Option<Message>,
    on_move: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_release: Option<Message>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct InnerState {
    modifiers: keyboard::Modifiers,
    positions: Vec<Point>,
}

impl<'a, Message, Theme, Renderer> Workspace<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: iced::advanced::graphics::geometry::Renderer,
{
    pub fn new(
        camera: &'a Vec3,
        primitives: &'a Vec<Curve>,
        elements: Vec<(Point, Element<'a, Message, Theme, Renderer>)>,
        cache: &'a Cache<Renderer>,
    ) -> Self {
        Self {
            camera,
            primitives,
            elements,
            cache,
            pan: None,
            zoom: None,
            on_press: None,
            on_move: None,
            on_release: None,
        }
    }

    pub fn pan(mut self, pan: impl Fn(Vector) -> Message + 'a) -> Self {
        self.pan = Some(Box::new(pan));
        self
    }

    pub fn zoom(mut self, zoom: impl Fn(f32) -> Message + 'a) -> Self {
        self.zoom = Some(Box::new(zoom));
        self
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    pub fn on_move(mut self, on_move: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_move = Some(Box::new(on_move));
        self
    }

    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }
}

/// Implement Widet
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Workspace<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: iced::advanced::graphics::geometry::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InnerState>()
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn children(&self) -> Vec<Tree> {
        self.elements
            .iter()
            .map(|(_, content)| Tree::new(content.as_widget()))
            .collect()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::with_children(
            //// Fill the screen
            limits.resolve(Length::Fill, Length::Fill, Size::new(50., 50.)),
            ///// Layout child elements
            self.elements
                .iter()
                .zip(&mut tree.children)
                .map(|(e, t)| e.1.as_widget().layout(t, renderer, limits).move_to(e.0))
                .collect(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        workspace_layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        //// Saved curves
        let geo = self
            .cache
            .draw(renderer, workspace_layout.bounds().size() * 2., |frame| {
                println!("drawing!");
                frame.translate(Vector::new(-self.camera.x, -self.camera.y));

                //// Foreground
                self.primitives.iter().for_each(|v| v.draw(frame));
            });

        renderer.draw_geometry(geo);

        let padding = 0.0;

        //// Render Children in a layer that is bounded to the size of the workspace
        renderer.with_layer(workspace_layout.bounds().shrink(padding), |renderer| {
            let elements = self.elements.iter().zip(&tree.children);
            for ((e, tree), c_layout) in elements.zip(workspace_layout.children()) {
                e.1.as_widget()
                    .draw(tree, renderer, theme, style, c_layout, cursor, viewport);
            }
        });
    }

    //// Move children based on input events
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let event_status = event::Status::Ignored;

        ////Pass event down to children
        let event_status = self
            .elements
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((element, tree), layout)| {
                element.1.as_widget_mut().on_event(
                    tree,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event_status, event::Status::merge);

        match event_status {
            event::Status::Ignored => match (event.clone(), cursor.position()) {
                (
                    Event::Mouse(ButtonPressed(mouse::Button::Left))
                    | Event::Touch(FingerPressed { .. }),
                    Some(_),
                ) => {
                    if let Some(on_press) = self.on_press.clone() {
                        shell.publish(on_press);
                    }
                    event::Status::Captured
                }
                (
                    Event::Mouse(ButtonReleased(mouse::Button::Left))
                    | Event::Touch(FingerLifted { .. })
                    | Event::Touch(FingerLost { .. }),
                    _,
                ) => {
                    if let Some(on_release) = self.on_release.clone() {
                        shell.publish(on_release);
                    }
                    event::Status::Captured
                }

                (
                    Event::Mouse(CursorMoved { .. }) | Event::Touch(FingerMoved { .. }),
                    Some(cursor_position),
                ) => {
                    if let Some(on_move) = &self.on_move {
                        shell.publish(on_move(cursor_position));
                    }
                    event::Status::Captured
                }
                (Event::Mouse(WheelScrolled { delta }), _) => {
                    if let Some(pan) = &self.pan {
                        match delta {
                            ScrollDelta::Lines { x, y } => shell.publish(pan(Vector::new(x, y))),
                            ScrollDelta::Pixels { x, y } => {
                                shell.publish(pan(Vector::new(x * 5., y * 5.)))
                            }
                        }
                    }
                    event::Status::Captured
                }
                _ => event::Status::Ignored,
            },
            _ => event::Status::Ignored,
        }
    }

    //fn mouse_interaction(
    //    &self,
    //    tree: &Tree,
    //    layout: Layout<'_>,
    //    cursor: mouse::Cursor,
    //    _viewport: &Rectangle,
    //    _renderer: &Renderer,
    //) -> mouse::Interaction {
    //    //let action = tree.state.downcast_ref::<InnerState>().action;
    //
    //    match action {
    //        Action::Dragging { .. } => mouse::Interaction::Grabbing,
    //        Action::Idle => {
    //            if layout.children().any(|l| {
    //                cursor
    //                    .position_over(
    //                        l.bounds()
    //                            .intersection(&layout.bounds())
    //                            .unwrap_or(Rectangle::new((0., 0.).into(), (0., 0.).into())),
    //                    )
    //                    .is_some()
    //            }) {
    //                //TODO: get mouse status of children?
    //                mouse::Interaction::Grab
    //            } else {
    //                mouse::Interaction::default()
    //            }
    //        }
    //    }
    //}
}

/// Convert to an element
impl<'a, Message, Theme, Renderer> From<Workspace<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
    Renderer: 'a + iced::advanced::graphics::geometry::Renderer,
{
    fn from(
        workspace: Workspace<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Self::new(workspace)
    }
}

// Convenience function

/// Create a new `Workspace`
pub fn workspace<'a, Message, Theme, Renderer>(
    camera: &'a glam::Vec3,
    primitives: &'a Vec<Curve>,
    elements: Vec<(Point, Element<'a, Message, Theme, Renderer>)>,
    cache: &'a Cache<Renderer>,
) -> Workspace<'a, Message, Theme, Renderer>
where
    Theme: 'a + Catalog,
    Renderer: iced::advanced::graphics::geometry::Renderer,
{
    Workspace::new(camera, primitives, elements, cache)
}

/// Very rough styling implementation
/// The appearance of a workspace.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Color,

    pub foreground: Color,
}

pub trait Catalog: Sized {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.palette();

    let background = palette.background;
    let foreground = palette.primary;

    Style {
        background,
        foreground,
    }
}
