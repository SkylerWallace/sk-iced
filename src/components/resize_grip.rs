// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! A vertical resize grip using screen-space drag deltas; app owns dimensions.
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer,
    widget::{Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme};
pub struct ResizeGrip<'a, M> {
    appearance: super::Appearance,
    minimum: f32,
    enabled: bool,
    action: Option<Box<dyn Fn(f32) -> M + 'a>>,
    height: f32,
    maximum: f32,
    horizontal: bool,
    thickness: f32,
    indicator: bool,
}
#[derive(Default)]
struct Drag {
    start: Option<(f32, f32)>,
}
impl<M> Widget<M, Theme, Renderer> for ResizeGrip<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Drag>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Drag::default())
    }
    fn diff(&self, tree: &mut Tree) {
        if !self.enabled || self.action.is_none() {
            tree.state.downcast_mut::<Drag>().start = None;
        }
    }
    fn size(&self) -> Size<Length> {
        if self.horizontal {
            Size::new(Length::Fixed(self.thickness), Length::Fill)
        } else {
            Size::new(Length::Fill, Length::Fixed(self.thickness))
        }
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(if self.horizontal {
            limits.resolve(self.thickness, Length::Fill, Size::ZERO)
        } else {
            limits.resolve(Length::Fill, self.thickness, Size::ZERO)
        })
    }
    fn draw(
        &self,
        tree: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let t = self.appearance.tokens(theme);
        let active =
            tree.state.downcast_ref::<Drag>().start.is_some() || cursor.is_over(l.bounds());
        if self.indicator && t.visual.grip.fill.is_some() && self.enabled && self.action.is_some() {
            let width = if self.horizontal {
                3.
            } else {
                l.bounds().width.min(96.)
            };
            let height = if self.horizontal {
                l.bounds().height.min(96.)
            } else {
                3.
            };
            r.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: l.bounds().center_x() - width / 2.,
                        y: l.bounds().center_y() - height / 2.,
                        width,
                        height,
                    },
                    border: iced_core::Border {
                        radius: t
                            .visual
                            .grip
                            .shape
                            .map(|s| s.radius())
                            .unwrap_or(2.0.into()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                if active {
                    color(t.accent).into()
                } else {
                    t.visual
                        .grip
                        .fill
                        .unwrap_or(crate::visual::Fill::Solid(crate::visual::Rgba::new(
                            t.secondary,
                            0.45,
                        )))
                        .background(t.visual.grip.opacity())
                },
            );
            return;
        }
        if !active || !self.enabled || self.action.is_none() {
            return;
        }
        r.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: if self.horizontal {
                        l.bounds().center_x() - 0.5
                    } else {
                        l.bounds().x
                    },
                    y: if self.horizontal {
                        l.bounds().y
                    } else {
                        l.bounds().center_y() - 0.5
                    },
                    width: if self.horizontal {
                        1.
                    } else {
                        l.bounds().width
                    },
                    height: if self.horizontal {
                        l.bounds().height
                    } else {
                        1.
                    },
                },
                ..Default::default()
            },
            color(t.accent),
        );
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<Drag>();
        if !self.enabled || self.action.is_none() {
            state.start = None;
            return;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(l.bounds()) =>
            {
                state.start = cursor
                    .position()
                    .map(|p| (if self.horizontal { p.x } else { p.y }, self.height));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.start.is_some() => {
                let (y, height) = state.start.unwrap();
                if let Some(action) = &self.action {
                    shell.publish(action(
                        (height
                            + if self.horizontal {
                                position.x - y
                            } else {
                                y - position.y
                            })
                        .clamp(self.minimum, self.maximum.max(self.minimum)),
                    ));
                }
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.start.take().is_some() =>
            {
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(iced_core::window::Event::Unfocused) => {
                state.start = None;
                shell.request_redraw();
            }
            _ => {}
        }
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if self.enabled
            && self.action.is_some()
            && (tree.state.downcast_ref::<Drag>().start.is_some() || cursor.is_over(l.bounds()))
        {
            if self.horizontal {
                mouse::Interaction::ResizingHorizontally
            } else {
                mouse::Interaction::ResizingVertically
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, M: 'a> ResizeGrip<'a, M> {
    pub fn new(appearance: super::Appearance, height: f32) -> Self {
        Self {
            appearance,
            height,
            minimum: 0.0,
            maximum: f32::MAX,
            horizontal: false,
            thickness: 6.,
            indicator: true,
            enabled: true,
            action: None,
        }
    }
    /// Resize a left/right split using horizontal screen-space drag deltas.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(0.);
        self
    }
    pub fn indicator(mut self, visible: bool) -> Self {
        self.indicator = visible;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }
    pub fn limits(mut self, minimum: f32, maximum: f32) -> Self {
        self.minimum = minimum.max(0.0);
        self.maximum = maximum.max(self.minimum);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn on_resize(mut self, action: impl Fn(f32) -> M + 'a) -> Self {
        self.action = Some(Box::new(action));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<ResizeGrip<'a, M>> for Element<'a, M> {
    fn from(grip: ResizeGrip<'a, M>) -> Self {
        grip.build()
    }
}

#[cfg(test)]
mod axis_tests {
    use super::*;
    #[test]
    fn horizontal_grip_tracks_screen_delta_and_clamps() {
        let appearance = super::super::Appearance::fixed(
            iced_core::Font::DEFAULT,
            crate::palette::palette(true, false),
        );
        let mut widget = ResizeGrip::new(appearance, 210.)
            .horizontal()
            .limits(160., 400.)
            .on_resize(|v| v);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let mut tree = Tree::new(&widget as &dyn Widget<f32, Theme, Renderer>);
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(6., 500.)),
        );
        assert_eq!(node.size(), Size::new(6., 500.));
        let mut messages = vec![];
        for (event, point) in [
            (
                mouse::Event::ButtonPressed(mouse::Button::Left),
                iced_core::Point::new(3., 50.),
            ),
            (
                mouse::Event::CursorMoved {
                    position: iced_core::Point::new(53., 50.),
                },
                iced_core::Point::new(53., 50.),
            ),
            (
                mouse::Event::CursorMoved {
                    position: iced_core::Point::new(900., 50.),
                },
                iced_core::Point::new(900., 50.),
            ),
            (
                mouse::Event::ButtonReleased(mouse::Button::Left),
                iced_core::Point::new(900., 50.),
            ),
        ] {
            widget.update(
                &mut tree,
                &Event::Mouse(event),
                Layout::new(&node),
                mouse::Cursor::Available(point),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &node.bounds(),
            );
        }
        assert_eq!(messages, vec![260., 400.]);
        assert!(tree.state.downcast_ref::<Drag>().start.is_none());
    }
}
