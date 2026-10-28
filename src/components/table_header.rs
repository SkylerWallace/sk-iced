// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Header gestures stay local until release: sorting, reordering and a context menu.
use super::Appearance;
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Point, Rectangle, Size, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HeaderResize {
    #[default]
    None,
    Leading(f32),
    Trailing(f32),
}
impl HeaderResize {
    fn contains(self, p: Point, bounds: Rectangle) -> bool {
        match self {
            Self::None => false,
            Self::Leading(w) => p.x < bounds.x + w,
            Self::Trailing(w) => p.x >= bounds.x + bounds.width - w,
        }
    }
}
/// Sort/reorder/context gestures around a native row. Column IDs, sort values
/// and persisted visibility/order stay in the app; callbacks use visible indices.
pub struct TableHeader<'a, M> {
    content: Element<'a, M>,
    appearance: Appearance,
    columns: Vec<HeaderResize>,
    enabled: bool,
    on_sort: Option<Box<dyn Fn(usize) -> M + 'a>>,
    on_move: Option<Box<dyn Fn(usize, usize) -> M + 'a>>,
    on_context: Option<Box<dyn Fn(Point) -> M + 'a>>,
}
impl<'a, M: 'a> TableHeader<'a, M> {
    pub fn new(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            appearance,
            content: content.into(),
            columns: Vec::new(),
            enabled: true,
            on_sort: None,
            on_move: None,
            on_context: None,
        }
    }
    pub fn columns(mut self, columns: impl IntoIterator<Item = HeaderResize>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn on_sort(mut self, action: impl Fn(usize) -> M + 'a) -> Self {
        self.on_sort = Some(Box::new(action));
        self
    }
    pub fn on_move(mut self, action: impl Fn(usize, usize) -> M + 'a) -> Self {
        self.on_move = Some(Box::new(action));
        self
    }
    pub fn on_context(mut self, action: impl Fn(Point) -> M + 'a) -> Self {
        self.on_context = Some(Box::new(action));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<TableHeader<'a, M>> for Element<'a, M> {
    fn from(value: TableHeader<'a, M>) -> Self {
        value.build()
    }
}
#[derive(Default)]
struct State {
    start: Option<(usize, Point)>,
    target: Option<usize>,
    dragging: bool,
}
impl<M> Widget<M, Theme, Renderer> for TableHeader<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.content]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut t.children[0], r, limits)
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        th: &Theme,
        s: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v);
        let state = t.state.downcast_ref::<State>();
        if state.dragging
            && let Some(target) = state.target
            && let Some(cell) = l.children().nth(target)
        {
            let tokens = self.appearance.tokens(th);
            if let Some(clip) = l.bounds().intersection(v) {
                r.with_layer(clip, |r| {
                    r.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: if state.start.is_some_and(|(from, _)| from < target) {
                                    cell.bounds().x + cell.bounds().width - 2.
                                } else {
                                    cell.bounds().x
                                },
                                y: l.bounds().y,
                                width: 2.,
                                height: l.bounds().height,
                            },
                            ..Default::default()
                        },
                        crate::theme::color(tokens.accent),
                    )
                });
            }
        }
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, M>,
        v: &Rectangle,
    ) {
        let state = t.state.downcast_mut::<State>();
        if !self.enabled {
            *state = State::default();
            return;
        }
        let hit = c.position().filter(|p| v.contains(*p)).and_then(|p| {
            l.children()
                .enumerate()
                .find(|(_, cell)| cell.bounds().contains(p))
                .map(|(i, cell)| (i, p, cell.bounds()))
        });
        match e {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
                if self.on_context.is_some()
                    && c.is_over(l.bounds())
                    && c.position().is_some_and(|p| v.contains(p)) =>
            {
                if let Some(action) = &self.on_context {
                    s.publish(action(c.position().unwrap()));
                }
                s.capture_event();
                return;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if hit.is_some() => {
                let (i, p, bounds) = hit.unwrap();
                let divider = self
                    .columns
                    .get(i)
                    .copied()
                    .unwrap_or_default()
                    .contains(p, bounds);
                if !divider && (self.on_sort.is_some() || self.on_move.is_some()) {
                    state.start = Some((i, p));
                    state.target = Some(i);
                    state.dragging = false;
                    s.capture_event();
                    return;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.start.is_some() => {
                let (_, start) = state.start.unwrap();
                state.dragging |= self.on_move.is_some() && start.distance(*position) > 5.;
                state.target = hit.map(|h| h.0);
                s.capture_event();
                s.request_redraw();
                return;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.start.is_some() =>
            {
                let (from, _) = state.start.take().unwrap();
                if let Some((to, _, _)) = hit {
                    if state.dragging {
                        if from != to
                            && let Some(action) = &self.on_move
                        {
                            s.publish(action(from, to));
                        }
                    } else if from == to
                        && let Some(action) = &self.on_sort
                    {
                        s.publish(action(from));
                    }
                }
                state.target = None;
                state.dragging = false;
                s.capture_event();
                s.request_redraw();
                return;
            }
            Event::Window(iced_core::window::Event::Unfocused) => {
                *state = State::default();
                s.request_redraw();
            }
            Event::Keyboard(iced_core::keyboard::Event::KeyPressed {
                key: iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape),
                ..
            }) if state.start.is_some() => {
                *state = State::default();
                s.capture_event();
                s.request_redraw();
                return;
            }
            _ => {}
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        if t.state.downcast_ref::<State>().dragging {
            mouse::Interaction::Grabbing
        } else {
            let child = self
                .content
                .as_widget()
                .mouse_interaction(&t.children[0], l, c, v, r);
            if child != mouse::Interaction::None {
                child
            } else if c.is_over(l.bounds()) {
                mouse::Interaction::Pointer
            } else {
                child
            }
        }
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, tr)
    }
}
