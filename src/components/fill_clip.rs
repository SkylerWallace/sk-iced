// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Paint-only progress text clipping; content keeps its full-width layout.
use crate::animation::{Element, Renderer};
use iced_core::{
    Layout, Length, Rectangle, Renderer as _, Size, Theme, Widget, layout, mouse, renderer,
    widget::Tree,
};
pub struct FillClip<'a, M> {
    content: Element<'a, M>,
    percent: f32,
}
impl<M> Widget<M, Theme, Renderer> for FillClip<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, tree: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], r, limits);
        layout::Node::with_children(child.size(), vec![child])
    }
    fn draw(
        &self,
        tree: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = l.bounds();
        let fill = Rectangle {
            width: bounds.width * self.percent.clamp(0.0, 100.0) / 100.0,
            ..bounds
        };
        if let Some(clip) = fill.intersection(viewport) {
            r.with_layer(clip, |r| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    r,
                    theme,
                    style,
                    l.children().next().unwrap(),
                    cursor,
                    &clip,
                )
            });
        }
    }
}

impl<'a, M: 'a> FillClip<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>, percent: f32) -> Self {
        Self {
            content: content.into(),
            percent,
        }
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<FillClip<'a, M>> for Element<'a, M> {
    fn from(clip: FillClip<'a, M>) -> Self {
        clip.build()
    }
}
