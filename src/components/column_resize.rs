// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Local column-divider capture, bounded sizing and app-owned resize actions.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer,
    widget::{Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme};

pub struct ColumnResize<'a, M> {
    appearance: Appearance,
    width: f32,
    minimum: f32,
    maximum: f32,
    scale: f32,
    direction: f32,
    enabled: bool,
    on_resize: Option<Box<dyn Fn(f32) -> M + 'a>>,
}
impl<'a, M: 'a> ColumnResize<'a, M> {
    pub fn new(appearance: Appearance, width: f32) -> Self {
        Self {
            appearance,
            width,
            minimum: 32.0,
            maximum: f32::MAX,
            scale: 1.0,
            direction: 1.0,
            enabled: true,
            on_resize: None,
        }
    }
    /// Widths and callbacks are logical layout units. Scale only affects handle art.
    pub fn limits(mut self, minimum: f32, maximum: f32) -> Self {
        self.minimum = minimum.max(0.0);
        self.maximum = maximum.max(self.minimum);
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.01);
        self
    }
    pub fn leading(mut self, leading: bool) -> Self {
        self.direction = if leading { -1.0 } else { 1.0 };
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn on_resize(mut self, action: impl Fn(f32) -> M + 'a) -> Self {
        self.on_resize = Some(Box::new(action));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<ColumnResize<'a, M>> for Element<'a, M> {
    fn from(value: ColumnResize<'a, M>) -> Self {
        value.build()
    }
}
#[derive(Default)]
struct Drag {
    start: Option<(f32, f32)>,
}
impl<M> Widget<M, Theme, Renderer> for ColumnResize<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Drag>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Drag::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(6.0 * self.scale),
            Length::Fixed(26.0 * self.scale),
        )
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(6.0 * self.scale, 26.0 * self.scale, Size::ZERO))
    }
    fn draw(
        &self,
        tree: &Tree,
        r: &mut Renderer,
        th: &Theme,
        _: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        _: &Rectangle,
    ) {
        let t = self.appearance.tokens(th);
        let active = self.enabled
            && self.on_resize.is_some()
            && (tree.state.downcast_ref::<Drag>().start.is_some() || c.is_over(l.bounds()));
        r.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: l.bounds().center_x() - 0.5,
                    y: l.bounds().y + 5.0 * self.scale,
                    width: 1.0,
                    height: 16.0 * self.scale,
                },
                ..Default::default()
            },
            color(if active { t.accent } else { t.border }),
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
        if !self.enabled || self.on_resize.is_none() {
            state.start = None;
            return;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(l.bounds()) =>
            {
                state.start = cursor.position().map(|p| (p.x, self.width));
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.start.is_some() => {
                // Scrollable ancestors translate the cursor into content coordinates.
                let Some(position) = cursor.position() else {
                    shell.capture_event();
                    return;
                };
                let (start, width) = state.start.unwrap();
                if let Some(action) = &self.on_resize {
                    shell.publish(action(
                        (width + (position.x - start) * self.direction)
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
            && self.on_resize.is_some()
            && (tree.state.downcast_ref::<Drag>().start.is_some() || cursor.is_over(l.bounds()))
        {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::default()
        }
    }
}
