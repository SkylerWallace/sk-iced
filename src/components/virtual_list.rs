// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Fixed-height viewport. Wheel and thumb state stay local to the widget;
//! A bounded row cache preserves shaped text while scrolling. Translation and
//! cache refills happen locally; scrolling never invalidates application layout.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::Length::Fill;
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer,
    widget::{Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme};
use iced_widget::Column;

pub const ROW_HEIGHT: f32 = 21.0;
const RAIL: f32 = 16.0;

#[cfg(test)]
pub fn virtual_list<'a, M: 'a>(
    len: usize,
    identity: u64,
    reveal: Option<(u64, usize)>,
    row: impl Fn(usize) -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    Element::new(VirtualList {
        len,
        identity,
        row_height: ROW_HEIGHT,
        row: Box::new(row),
        content: Column::new(),
        cached: 0..0,
        content_node: layout::Node::default(),
        cached_width: 0.0,
        appearance: Appearance::themed(iced_core::Font::DEFAULT, crate::palette::from_theme),
        event_hook: None,
        on_double_click: None,
        reveal,
    })
}
#[cfg(test)]
pub fn file_list<'a, M: 'a>(
    len: usize,
    identity: u64,
    row: impl Fn(usize) -> Element<'a, M> + 'a,
    on_double: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    file_list_sized(len, identity, ROW_HEIGHT, row, on_double)
}
#[cfg(test)]
pub fn file_list_sized<'a, M: 'a>(
    len: usize,
    identity: u64,
    row_height: f32,
    row: impl Fn(usize) -> Element<'a, M> + 'a,
    on_double: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    file_list_revealed(len, identity, row_height, None, row, on_double)
}
#[cfg(test)]
pub fn file_list_revealed<'a, M: 'a>(
    len: usize,
    identity: u64,
    row_height: f32,
    reveal: Option<(u64, usize)>,
    row: impl Fn(usize) -> Element<'a, M> + 'a,
    on_double: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    Element::new(VirtualList {
        len,
        identity,
        row_height: row_height.max(1.),
        row: Box::new(row),
        content: Column::new(),
        cached: 0..0,
        content_node: layout::Node::default(),
        cached_width: 0.0,
        appearance: Appearance::themed(iced_core::Font::DEFAULT, crate::palette::from_theme),
        event_hook: None,
        on_double_click: Some(Box::new(on_double)),
        reveal,
    })
}
pub struct VirtualList<'a, M> {
    appearance: Appearance,
    event_hook: Option<fn(&Event)>,
    len: usize,
    identity: u64,
    row_height: f32,
    row: Box<dyn Fn(usize) -> Element<'a, M> + 'a>,
    content: Column<'a, M>,
    cached: std::ops::Range<usize>,
    content_node: layout::Node,
    cached_width: f32,
    on_double_click: Option<Box<dyn Fn(usize) -> M + 'a>>,
    reveal: Option<(u64, usize)>,
}
#[derive(Default)]
struct State {
    identity: u64,
    row_height: f32,
    last_reveal: Option<u64>,
    offset: f32,
    shift: bool,
    grab: Option<f32>,
    first: usize,
    positions: std::collections::HashMap<u64, f32>,
    last_click: Option<(usize, std::time::Instant)>,
}

impl<'a, M: 'a> VirtualList<'a, M> {
    /// A bounded fixed-height viewport. identity is a stable view key; change it
    /// to switch datasets and restore their individual offsets. Rows remain
    /// app-owned; use stable item IDs in actions rather than cached row indices.
    pub fn new(
        appearance: Appearance,
        len: usize,
        identity: u64,
        row: impl Fn(usize) -> Element<'a, M> + 'a,
    ) -> Self {
        Self {
            appearance,
            event_hook: None,
            len,
            identity,
            row_height: ROW_HEIGHT,
            row: Box::new(row),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.0,
            on_double_click: None,
            reveal: None,
        }
    }
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(1.0);
        self
    }
    /// A request serial and visible index. Reusing a serial leaves manual scrolling alone.
    pub fn reveal(mut self, request: Option<(u64, usize)>) -> Self {
        self.reveal = request;
        self
    }
    pub fn on_double_click(mut self, action: impl Fn(usize) -> M + 'a) -> Self {
        self.on_double_click = Some(Box::new(action));
        self
    }
    /// Optional app-owned instrumentation; never publishes scroll messages.
    pub fn event_hook(mut self, hook: fn(&Event)) -> Self {
        self.event_hook = Some(hook);
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<VirtualList<'a, M>> for Element<'a, M> {
    fn from(value: VirtualList<'a, M>) -> Self {
        value.build()
    }
}
impl<M> VirtualList<'_, M> {
    fn prepare(&mut self, tree: &mut Tree, renderer: &Renderer, size: Size) {
        let state = tree.state.downcast_mut::<State>();
        if state.identity != self.identity {
            if state.positions.len() > 128 {
                state.positions.clear();
            }
            state.positions.insert(state.identity, state.offset);
            state.identity = self.identity;
            state.offset = state.positions.get(&self.identity).copied().unwrap_or(0.0);
            state.grab = None;
            state.last_click = None;
        }
        if state.row_height != self.row_height {
            if state.row_height > 0. {
                let ratio = self.row_height / state.row_height;
                state.offset *= ratio;
                for offset in state.positions.values_mut() {
                    *offset *= ratio;
                }
            }
            state.row_height = self.row_height;
            state.grab = None;
            self.cached_width = -1.;
            self.cached = 0..0;
        }
        if let Some((key, index)) = self.reveal
            && state.last_reveal != Some(key)
        {
            let top = index as f32 * self.row_height;
            let bottom = top + self.row_height;
            if top < state.offset {
                state.offset = top;
            } else if bottom > state.offset + size.height {
                state.offset = bottom - size.height;
            }
            state.last_reveal = Some(key);
        }
        state.offset = state
            .offset
            .clamp(0.0, overflow_sized(self.len, size.height, self.row_height));
        let visible = visible_sized(self.len, size.height, state.offset, self.row_height);
        let width = (size.width - RAIL).max(0.0);
        if self.cached.start <= visible.start
            && self.cached.end >= visible.end
            && !tree.children.is_empty()
        {
            if self.cached_width != width {
                self.cached_width = width;
                self.content_node = self.content.layout(
                    &mut tree.children[0],
                    renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
                );
            }
            return;
        }
        let first = visible.start.saturating_sub(32) / 32 * 32;
        let end = (first + visible.len() + 96).min(self.len);
        self.content = Column::with_children((first..end).map(|i| (self.row)(i))).width(Fill);
        if state.first != first {
            tree.children.clear();
        }
        state.first = first;
        self.cached = first..end;
        self.cached_width = width;
        if tree.children.is_empty() {
            tree.children
                .push(Tree::new(&self.content as &dyn Widget<M, Theme, Renderer>));
        } else {
            tree.children[0].diff(&self.content as &dyn Widget<M, Theme, Renderer>);
        }
        self.content_node = self.content.layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
        );
    }
    fn translation(&self, tree: &Tree, bounds: Rectangle) -> iced_core::Vector {
        let state = tree.state.downcast_ref::<State>();
        iced_core::Vector::new(
            bounds.x,
            bounds.y + self.cached.start as f32 * self.row_height - state.offset,
        )
    }
}
fn overflow_sized(len: usize, height: f32, row_height: f32) -> f32 {
    (len as f32 * row_height - height).max(0.0)
}
fn visible_sized(len: usize, height: f32, offset: f32, row_height: f32) -> std::ops::Range<usize> {
    let first = (offset / row_height).floor() as usize;
    first.min(len)..(first + (height / row_height).ceil() as usize + 1).min(len)
}
fn thumb_sized(bounds: Rectangle, len: usize, offset: f32, row_height: f32) -> Rectangle {
    let height = (bounds.height * bounds.height / (len as f32 * row_height).max(1.0))
        .max(28.0)
        .min(bounds.height);
    let travel = (bounds.height - height).max(0.0);
    Rectangle {
        x: bounds.x + bounds.width - 11.0,
        y: bounds.y + travel * offset / overflow_sized(len, bounds.height, row_height).max(1.0),
        width: 6.0,
        height,
    }
}
impl<M> Widget<M, Theme, Renderer> for VirtualList<'_, M> {
    fn size(&self) -> Size<Length> {
        Size::new(Fill, Fill)
    }
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn diff(&self, _tree: &mut Tree) {} // Visible children are reconciled during layout.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Fill, Fill, Size::ZERO);
        self.prepare(tree, renderer, size);
        layout::Node::new(size)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation,
    ) {
        let translation = self.translation(tree, layout.bounds());
        self.content.operate(
            &mut tree.children[0],
            Layout::with_offset(translation, &self.content_node),
            renderer,
            operation,
        );
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(clip) = layout.bounds().intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            let content_clip = Rectangle {
                width: (clip.width - RAIL).max(0.0),
                ..clip
            };
            renderer.with_layer(content_clip, |renderer| {
                self.content.draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    Layout::with_offset(
                        self.translation(tree, layout.bounds()),
                        &self.content_node,
                    ),
                    cursor,
                    &content_clip,
                );
            });
            if overflow_sized(self.len, clip.height, self.row_height) > 0.0 {
                let state = tree.state.downcast_ref::<State>();
                let t = self.appearance.tokens(theme);
                let bounds = clip;
                super::RailVisual {
                    track: Rectangle {
                        x: bounds.x + bounds.width - 10.0,
                        width: 4.0,
                        ..bounds
                    },
                    thumb: Some(thumb_sized(bounds, self.len, state.offset, self.row_height)),
                    track_color: color(t.border),
                    thumb_color: color(if state.grab.is_some() {
                        t.accent
                    } else {
                        t.secondary
                    }),
                    track_radius: 0.0,
                    thumb_radius: 3.0,
                }
                .draw_styled(renderer, t.visual.scrollbar);
            }
        });
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let Some(bounds) = layout.bounds().intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_mut::<State>();
        let max = overflow_sized(self.len, bounds.height, self.row_height);
        let mut scrolled = false;
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) && let Some(position) = cursor
            .position_over(bounds)
            .filter(|p| p.x < bounds.x + bounds.width - RAIL)
        {
            let index = ((position.y - bounds.y + state.offset) / self.row_height).floor() as usize;
            if index < self.len {
                let now = std::time::Instant::now();
                let double = state.last_click.is_some_and(|(last, at)| {
                    last == index && now.duration_since(at) < std::time::Duration::from_millis(500)
                });
                state.last_click = if double { None } else { Some((index, now)) };
                if double && let Some(action) = &self.on_double_click {
                    shell.publish(action(index));
                    shell.capture_event();
                    return;
                }
            }
        }
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.is_over(bounds)
                    && max > 0.0
                    && !state.shift
                    && match delta {
                        mouse::ScrollDelta::Lines { y, .. }
                        | mouse::ScrollDelta::Pixels { y, .. } => *y != 0.0,
                    } =>
            {
                let y = match *delta {
                    mouse::ScrollDelta::Lines { y, .. } => y * self.row_height * 3.0,
                    mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                state.offset = (state.offset - y).clamp(0.0, max);
                scrolled = true;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds) && max > 0.0 =>
            {
                if let Some(p) = cursor
                    .position()
                    .filter(|p| p.x >= bounds.x + bounds.width - RAIL)
                {
                    let thumb = thumb_sized(bounds, self.len, state.offset, self.row_height);
                    let grab = if thumb.contains(p) {
                        p.y - thumb.y
                    } else {
                        thumb.height / 2.0
                    };
                    state.grab = Some(grab);
                    state.offset =
                        ((p.y - bounds.y - grab) / (bounds.height - thumb.height).max(1.0) * max)
                            .clamp(0.0, max);
                    scrolled = true;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.grab.is_some() => {
                let thumb = thumb_sized(bounds, self.len, state.offset, self.row_height);
                state.offset = ((position.y - bounds.y - state.grab.unwrap())
                    / (bounds.height - thumb.height).max(1.0)
                    * max)
                    .clamp(0.0, max);
                scrolled = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.grab.take().is_some() =>
            {
                scrolled = true;
            }
            Event::Keyboard(iced_core::keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift = modifiers.shift()
            }
            Event::Window(iced_core::window::Event::Unfocused) => {
                state.shift = false;
                state.grab = None;
            }
            _ => {}
        }
        if scrolled {
            if let Some(hook) = self.event_hook {
                hook(event);
            }
            shell.capture_event();
            self.prepare(tree, renderer, layout.bounds().size());
            shell.request_redraw();
            return;
        }
        if let Some(clip) = bounds.intersection(viewport) {
            let offset = self.translation(tree, layout.bounds());
            // Cached rows extend past this viewport. Never let their invisible
            // buttons consume clicks intended for the adjacent file pane.
            let cursor = if cursor.is_over(clip) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            };
            self.content.update(
                &mut tree.children[0],
                event,
                Layout::with_offset(offset, &self.content_node),
                cursor,
                renderer,
                clipboard,
                shell,
                &clip,
            );
        }
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<State>().grab.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let visible = layout.bounds().intersection(viewport).unwrap_or_default();
        if cursor
            .position()
            .is_some_and(|p| visible.contains(p) && p.x >= visible.x + visible.width - RAIL)
            && overflow_sized(self.len, visible.height, self.row_height) > 0.0
        {
            return mouse::Interaction::Pointer;
        }
        self.content.mouse_interaction(
            &tree.children[0],
            Layout::with_offset(self.translation(tree, layout.bounds()), &self.content_node),
            if cursor.is_over(layout.bounds()) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            },
            viewport,
            renderer,
        )
    }
}

#[cfg(test)]
fn overflow(len: usize, height: f32) -> f32 {
    overflow_sized(len, height, ROW_HEIGHT)
}
#[cfg(test)]
fn visible(len: usize, height: f32, offset: f32) -> std::ops::Range<usize> {
    visible_sized(len, height, offset, ROW_HEIGHT)
}
#[cfg(test)]
fn thumb(bounds: Rectangle, len: usize, offset: f32) -> Rectangle {
    thumb_sized(bounds, len, offset, ROW_HEIGHT)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn width_only_resize_reuses_rows_and_reflows_the_cached_layout() {
        let built = std::cell::Cell::new(0);
        let mut list = VirtualList::<()> {
            len: 10_000,
            identity: 1,
            row_height: ROW_HEIGHT,
            row: Box::new(|_| {
                built.set(built.get() + 1);
                iced_widget::Space::new().height(ROW_HEIGHT).into()
            }),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, |t| {
                crate::palette::from_theme(t)
            }),
            event_hook: None,
            on_double_click: None,
            reveal: None,
        };
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let mut tree = Tree::new(&list as &dyn Widget<(), Theme, Renderer>);
        let initial = Size::new(500., 300.);
        let _ = list.layout(&mut tree, &renderer, &layout::Limits::new(initial, initial));
        let initial_built = built.get();
        assert!(initial_built > 0 && initial_built < 200);
        for width in [700., 400., 800., 500.] {
            let size = Size::new(width, 300.);
            let _ = list.layout(&mut tree, &renderer, &layout::Limits::new(size, size));
            assert_eq!(
                built.get(),
                initial_built,
                "resizing must not rebuild cached rows"
            );
            assert_eq!(list.content_node.size().width, width - RAIL);
        }
    }
    use iced_core::Point;

    #[test]
    fn nested_horizontal_scroll_reuses_rows_and_keeps_vertical_thumb_at_the_visible_edge() {
        use std::cell::Cell;
        let built = Cell::new(0usize);
        let list = file_list(
            100_000,
            1,
            |_| {
                built.set(built.get() + 1);
                iced_widget::Space::new().height(ROW_HEIGHT).into()
            },
            |_| (),
        );
        let mut element: Element<'_, ()> = iced_widget::scrollable(
            iced_widget::column![iced_widget::Space::new().height(26), list]
                .width(900)
                .height(Fill),
        )
        .direction(iced_widget::scrollable::Direction::Horizontal(
            iced_widget::scrollable::Scrollbar::new()
                .width(6)
                .scroller_width(6)
                .spacing(4),
        ))
        .width(320)
        .height(300)
        .into();
        let mut tree = Tree::new(&element);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let size = Size::new(320.0, 300.0);
        let node =
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let initial = built.get();
        fn list_state(tree: &Tree) -> &State {
            if tree.tag == tree::Tag::of::<State>() {
                return tree.state.downcast_ref::<State>();
            }
            tree.children.iter().find_map(find).unwrap()
        }
        fn find(tree: &Tree) -> Option<&State> {
            if tree.tag == tree::Tag::of::<State>() {
                Some(tree.state.downcast_ref::<State>())
            } else {
                tree.children.iter().find_map(find)
            }
        }
        struct Probe(f32);
        impl iced_core::widget::Operation for Probe {
            fn scrollable(
                &mut self,
                _: Option<&iced_core::widget::Id>,
                _: Rectangle,
                _: Rectangle,
                translation: iced_core::Vector,
                _: &mut dyn iced_core::widget::operation::Scrollable,
            ) {
                self.0 = self.0.max(translation.x);
            }
            fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn iced_core::widget::Operation)) {
                f(self);
            }
        }
        let mut messages = vec![];
        for _ in 0..4 {
            let mut shell = Shell::new(&mut messages);
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: -1.0, y: 0.0 },
                }),
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(100.0, 100.0)),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
                &node.bounds(),
            );
            assert!(!shell.is_layout_invalid());
        }
        assert!(messages.is_empty());
        assert_eq!(built.get(), initial);
        assert_eq!(list_state(&tree).offset, 0.0);
        let mut probe = Probe(0.0);
        element
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
        assert!(probe.0 > 0.0);
        let before_shift = probe.0;
        for event in [
            Event::Keyboard(iced_core::keyboard::Event::ModifiersChanged(
                iced_core::keyboard::Modifiers::SHIFT,
            )),
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
            }),
            Event::Keyboard(iced_core::keyboard::Event::ModifiersChanged(
                iced_core::keyboard::Modifiers::empty(),
            )),
        ] {
            let mut shell = Shell::new(&mut messages);
            element.as_widget_mut().update(
                &mut tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(100.0, 100.0)),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
                &node.bounds(),
            );
            assert!(!shell.is_layout_invalid());
        }
        let mut after_shift = Probe(0.0);
        element
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut after_shift);
        assert!(after_shift.0 > before_shift);
        assert_eq!(list_state(&tree).offset, 0.0);
        assert_eq!(built.get(), initial);
        let mut send = |event, point| {
            element.as_widget_mut().update(
                &mut tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(point),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &node.bounds(),
            );
        };
        send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Point::new(314.0, 60.0),
        );
        send(
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(314.0, 200.0),
            }),
            Point::new(314.0, 200.0),
        );
        send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            Point::new(314.0, 200.0),
        );
        assert!(list_state(&tree).offset > 1000.0);
    }
    #[test]
    fn double_click_publishes_one_open_for_the_visible_row() {
        let mut list = VirtualList::<u8> {
            len: 10,
            identity: 1,
            row_height: ROW_HEIGHT,
            row: Box::new(|_| {
                iced_widget::button(iced_widget::Space::new().height(ROW_HEIGHT))
                    .height(ROW_HEIGHT)
                    .width(Fill)
                    .on_press(1)
                    .into()
            }),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.0,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, |t| {
                crate::palette::from_theme(t)
            }),
            event_hook: None,
            on_double_click: Some(Box::new(|_| 2)),
            reveal: None,
        };
        let mut tree = Tree::new(&list as &dyn Widget<u8, Theme, Renderer>);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let node = list.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(500.0, 200.0), Size::new(500.0, 200.0)),
        );
        let mut clipboard = iced_core::clipboard::Null;
        let mut messages = vec![];
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            let mut shell = Shell::new(&mut messages);
            list.update(
                &mut tree,
                &Event::Mouse(event),
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(100.0, 10.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &node.bounds(),
            );
        }
        assert_eq!(messages, vec![1, 2]);
    }
    #[test]
    fn invisible_cached_rows_cannot_consume_adjacent_pane_clicks() {
        let mut list = VirtualList::<()> {
            len: 1000,
            identity: 1,
            row_height: ROW_HEIGHT,
            row: Box::new(|_| {
                iced_widget::button(iced_widget::Space::new().width(Fill).height(ROW_HEIGHT))
                    .height(ROW_HEIGHT)
                    .on_press(())
                    .into()
            }),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.0,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, |t| {
                crate::palette::from_theme(t)
            }),
            event_hook: None,
            on_double_click: None,
            reveal: None,
        };
        let mut tree = Tree::new(&list as &dyn Widget<(), Theme, Renderer>);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let node = list.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(500.0, 560.0), Size::new(500.0, 560.0)),
        );
        let mut clipboard = iced_core::clipboard::Null;
        for position in [Point::new(100.0, 600.0), Point::new(100.0, 50.0)] {
            let mut messages = vec![];
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                let mut shell = Shell::new(&mut messages);
                list.update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(position),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &node.bounds(),
                );
            }
            assert_eq!(messages.len(), usize::from(position.y < 560.0));
        }
    }
    #[test]
    fn wheel_events_reuse_rows_without_app_messages_or_global_layout() {
        use std::{cell::Cell, rc::Rc};
        let built = Rc::new(Cell::new(0));
        let count = built.clone();
        let mut list = VirtualList::<()> {
            len: 100_000,
            identity: 1,
            row_height: ROW_HEIGHT,
            row: Box::new(move |_| {
                count.set(count.get() + 1);
                iced_widget::Space::new().height(ROW_HEIGHT).into()
            }),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.0,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, |t| {
                crate::palette::from_theme(t)
            }),
            event_hook: None,
            on_double_click: None,
            reveal: None,
        };
        let mut tree = Tree::new(&list as &dyn Widget<(), Theme, Renderer>);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let node = list.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(500.0, 560.0), Size::new(500.0, 560.0)),
        );
        let initial = built.get();
        assert!(initial < 140);
        let mut clipboard = iced_core::clipboard::Null;
        for _ in 0..10 {
            let mut messages = vec![];
            let mut shell = Shell::new(&mut messages);
            list.update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
                }),
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(100.0, 100.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &node.bounds(),
            );
            assert!(!shell.is_layout_invalid());
            assert!(shell.is_empty());
        }
        assert_eq!(built.get(), initial);
        assert_eq!(tree.state.downcast_ref::<State>().offset, ROW_HEIGHT * 30.0);
    }
    #[test]
    fn large_lists_only_construct_a_viewport() {
        assert_eq!(
            visible(100_000, ROW_HEIGHT * 20.0, ROW_HEIGHT * 1000.0),
            1000..1021
        );
        assert_eq!(visible(0, 560.0, 0.0), 0..0);
        let max = overflow(100_000, 560.0);
        let range = visible(100_000, 560.0, max);
        assert_eq!(range.end, 100_000);
        assert!(range.len() <= (560.0 / ROW_HEIGHT).ceil() as usize + 1);
    }
    #[test]
    fn thumb_tracks_the_full_range() {
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(300.0, 560.0));
        assert_eq!(thumb(bounds, 1000, 0.0).y, 0.0);
        let bottom = thumb(bounds, 1000, overflow(1000, bounds.height));
        assert_eq!(bottom.y + bottom.height, bounds.height);
    }
    #[test]
    fn reveal_scrolls_once_and_leaves_subsequent_manual_scroll_untouched() {
        let mut list = VirtualList::<()> {
            len: 1000,
            identity: 1,
            row_height: ROW_HEIGHT,
            row: Box::new(|_| iced_widget::Space::new().height(ROW_HEIGHT).into()),
            content: Column::new(),
            cached: 0..0,
            content_node: layout::Node::default(),
            cached_width: 0.,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, |t| {
                crate::palette::from_theme(t)
            }),
            event_hook: None,
            on_double_click: None,
            reveal: Some((7, 800)),
        };
        let mut tree = Tree::new(&list as &dyn Widget<(), Theme, Renderer>);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let size = Size::new(500., 200.);
        let node = list.layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let before = tree.state.downcast_ref::<State>().offset;
        assert!(before > 800.0 * ROW_HEIGHT - 200.0);
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        list.update(
            &mut tree,
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0., y: -1. },
            }),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(50., 50.)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &node.bounds(),
        );
        assert_eq!(
            tree.state.downcast_ref::<State>().offset,
            before + 3.0 * ROW_HEIGHT
        );
        list.reveal = Some((8, 4));
        list.prepare(&mut tree, &renderer, size);
        assert_eq!(tree.state.downcast_ref::<State>().offset, 4.0 * ROW_HEIGHT);
    }
    #[test]
    fn density_changes_preserve_top_item_and_double_click_targets() {
        for percent in [75, 125, 150, 175, 200] {
            let make = |height| {
                file_list_sized(
                    100_000,
                    123,
                    height,
                    move |_| iced_widget::Space::new().height(height).into(),
                    |row| row,
                )
            };
            let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
            let size = Size::new(500., 280.);
            let mut view = make(ROW_HEIGHT);
            let mut tree = Tree::new(&view);
            let node =
                view.as_widget_mut()
                    .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
            let mut messages = Vec::new();
            let point = iced_core::Point::new(50., 50.);
            view.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0., y: -10. },
                }),
                Layout::new(&node),
                mouse::Cursor::Available(point),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &node.bounds(),
            );
            assert_eq!(tree.state.downcast_ref::<State>().offset, 30. * ROW_HEIGHT);
            let height = ROW_HEIGHT * percent as f32 / 100.0;
            let mut changed = make(height);
            tree.diff(changed.as_widget());
            let node = changed.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(size, size),
            );
            assert_eq!(tree.state.downcast_ref::<State>().offset, 30. * height);
            let point = iced_core::Point::new(50., height * 2. + 2.);
            for _ in 0..2 {
                changed.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    Layout::new(&node),
                    mouse::Cursor::Available(point),
                    &renderer,
                    &mut iced_core::clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &node.bounds(),
                );
            }
            assert_eq!(messages, vec![32]);
            let end = overflow_sized(100_000, size.height, height);
            let thumb = thumb_sized(node.bounds(), 100_000, end, height);
            assert!((thumb.y + thumb.height - size.height).abs() < 0.01);
            assert!(visible_sized(100_000, size.height, end, height).len() < 32);
        }
    }
}
