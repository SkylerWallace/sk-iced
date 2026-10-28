// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Transparent drag marker; child actions remain immediate.

use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
pub struct DragHandle<'a, M, T> {
    content: Element<'a, M>,
    marker: T,
    hideable: bool,
    interaction: mouse::Interaction,
}
impl<M, T: Clone + 'static> Widget<M, Theme, Renderer> for DragHandle<'_, M, T> {
    fn tag(&self) -> iced_core::widget::tree::Tag {
        iced_core::widget::tree::Tag::of::<DragVisibility>()
    }
    fn state(&self) -> iced_core::widget::tree::State {
        iced_core::widget::tree::State::new(DragVisibility::default())
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
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l)
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
        if self.hideable && t.state.downcast_ref::<DragVisibility>().hidden {
            return;
        }
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
        let mut marker = self.marker.clone();
        o.custom(None, l.bounds(), &mut marker);
        if self.hideable {
            o.custom(None, l.bounds(), t.state.downcast_mut::<DragVisibility>());
        }
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
        let child = self
            .content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r);
        if child == mouse::Interaction::default() && c.is_over(l.bounds()) {
            self.interaction
        } else {
            child
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

#[derive(Debug, Default)]
pub struct DragVisibility {
    pub hidden: bool,
}
impl<'a, M: 'a, T: Clone + 'static> DragHandle<'a, M, T> {
    pub fn new(content: impl Into<Element<'a, M>>, marker: T) -> Self {
        Self {
            content: content.into(),
            marker,
            hideable: false,
            interaction: mouse::Interaction::default(),
        }
    }
    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = interaction;
        self
    }
    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a, T: Clone + 'static> From<DragHandle<'a, M, T>> for Element<'a, M> {
    fn from(handle: DragHandle<'a, M, T>) -> Self {
        handle.build()
    }
}
