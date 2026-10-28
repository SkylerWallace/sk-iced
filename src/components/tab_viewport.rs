// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Reveal a changed active tab locally, without scroll messages or title measurement.
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
#[cfg(test)]
pub fn view<'a, M: 'a>(
    content: Element<'a, M>,
    identity: u64,
    start: f32,
    end: f32,
) -> Element<'a, M> {
    Element::new(TabViewport {
        content,
        identity,
        start,
        end,
    })
}
pub struct TabViewport<'a, M> {
    content: Element<'a, M>,
    identity: u64,
    start: f32,
    end: f32,
}

impl<'a, M: 'a> TabViewport<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>, identity: u64, start: f32, end: f32) -> Self {
        Self {
            content: content.into(),
            identity,
            start,
            end,
        }
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<TabViewport<'a, M>> for Element<'a, M> {
    fn from(view: TabViewport<'a, M>) -> Self {
        view.build()
    }
}
#[derive(Default)]
struct State {
    last: Option<(u64, u32, u32, u32)>,
}
struct ScrollIntoView {
    start: f32,
    end: f32,
    done: bool,
}
impl Operation for ScrollIntoView {
    fn scrollable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        if self.done {
            return;
        }
        self.done = true;
        let current = translation.x;
        let offset = if self.start < current {
            self.start
        } else if self.end > current + bounds.width {
            self.end - bounds.width
        } else {
            current
        };
        let offset = offset.clamp(0.0, (content.width - bounds.width).max(0.0));
        if (offset - current).abs() > 0.01 {
            state.scroll_to(iced_core::widget::operation::scrollable::AbsoluteOffset {
                x: Some(offset),
                y: None,
            });
        }
    }
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        if !self.done {
            f(self);
        }
    }
}
impl<M> Widget<M, Theme, Renderer> for TabViewport<'_, M> {
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
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, limits);
        let key = (
            self.identity,
            self.start.to_bits(),
            self.end.to_bits(),
            node.bounds().width.to_bits(),
        );
        let state = t.state.downcast_mut::<State>();
        if state.last != Some(key) {
            self.content.as_widget_mut().operate(
                &mut t.children[0],
                Layout::new(&node),
                r,
                &mut ScrollIntoView {
                    start: self.start,
                    end: self.end,
                    done: false,
                },
            );
            state.last = Some(key);
        }
        node
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
        self.content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r)
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

#[cfg(test)]
mod tests {
    use super::*;
    use iced_core::Point;
    use iced_widget::{Space, row, scrollable};
    fn strip(identity: u64, index: usize) -> Element<'static, ()> {
        let content = row((0..30).map(|_| Space::new().width(72).height(30).into())).spacing(2);
        view(
            scrollable(content)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::hidden(),
                ))
                .width(Length::Fill)
                .into(),
            identity,
            index as f32 * 74.0,
            index as f32 * 74.0 + 72.0,
        )
    }
    struct Probe {
        offset: f32,
        width: f32,
    }
    impl Operation for Probe {
        fn scrollable(
            &mut self,
            _: Option<&iced_core::widget::Id>,
            bounds: Rectangle,
            _: Rectangle,
            translation: Vector,
            _: &mut dyn iced_core::widget::operation::Scrollable,
        ) {
            self.offset = translation.x;
            self.width = bounds.width;
        }
        fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
            f(self);
        }
    }
    fn offset(
        element: &mut Element<'_, ()>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &Renderer,
    ) -> Probe {
        let mut probe = Probe {
            offset: 0.0,
            width: 0.0,
        };
        element
            .as_widget_mut()
            .operate(tree, Layout::new(node), renderer, &mut probe);
        probe
    }
    #[test]
    fn active_tab_is_revealed_on_selection_and_resize_without_fighting_manual_scroll() {
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let mut element = strip(16, 15);
        let mut tree = Tree::new(&element);
        let mut node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(220.0, 30.0), Size::new(220.0, 30.0)),
        );
        let p = offset(&mut element, &mut tree, &node, &renderer);
        assert!(p.offset <= 15.0 * 74.0 && p.offset + p.width >= 15.0 * 74.0 + 72.0);
        element = strip(1, 0);
        tree.diff(element.as_widget());
        node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(220.0, 30.0), Size::new(220.0, 30.0)),
        );
        assert_eq!(
            offset(&mut element, &mut tree, &node, &renderer).offset,
            0.0
        );
        element = strip(30, 29);
        tree.diff(element.as_widget());
        node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(320.0, 30.0), Size::new(320.0, 30.0)),
        );
        let before = offset(&mut element, &mut tree, &node, &renderer).offset;
        node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(160.0, 30.0), Size::new(160.0, 30.0)),
        );
        let p = offset(&mut element, &mut tree, &node, &renderer);
        assert!(p.offset > before && p.offset + p.width >= 29.0 * 74.0 + 72.0);
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 4.0, y: 0.0 },
            }),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(60.0, 15.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &node.bounds(),
        );
        assert!(!shell.is_layout_invalid());
        assert!(messages.is_empty());
        let manual = offset(&mut element, &mut tree, &node, &renderer).offset;
        assert!(manual < p.offset);
        element = strip(30, 29);
        tree.diff(element.as_widget());
        node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::new(160.0, 30.0), Size::new(160.0, 30.0)),
        );
        assert_eq!(
            offset(&mut element, &mut tree, &node, &renderer).offset,
            manual
        );
    }
}
