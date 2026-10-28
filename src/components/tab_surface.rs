// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Rounded tab surface with a separate close hit area; does not own tab state.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    theme::blend,
};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
pub struct TabSurface<'a, M> {
    content: Element<'a, M>,
    selected: bool,
    appearance: Appearance,
}
impl<M> Widget<M, Theme, Renderer> for TabSurface<'_, M> {
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
        let tokens = self.appearance.tokens(th);
        let surface = if self.selected {
            tokens
                .visual
                .selected_tab
                .unwrap_or_default()
                .inherit(tokens.visual.tab)
        } else {
            tokens.visual.tab
        };
        let mut style = iced_widget::container::Style::default();
        surface.apply(&mut style.background, &mut style.border, &mut style.shadow);
        r.fill_quad(
            renderer::Quad {
                bounds: l.bounds(),
                border: iced_core::Border {
                    radius: surface.shape.map(|s| s.radius()).unwrap_or(
                        iced_core::border::Radius {
                            top_left: 12.0,
                            top_right: 12.0,
                            bottom_left: 0.0,
                            bottom_right: 0.0,
                        },
                    ),
                    color: style.border.color,
                    width: style.border.width,
                },
                shadow: style.shadow,
                ..Default::default()
            },
            style.background.unwrap_or_else(|| {
                blend(
                    if self.selected {
                        tokens.surface
                    } else {
                        tokens.navigation_background
                    },
                    tokens.text,
                    if c.is_over(Rectangle {
                        width: (l.bounds().width - 28.0).max(0.0),
                        ..l.bounds()
                    }) {
                        tokens.opacity_hover
                    } else {
                        0.0
                    },
                )
                .into()
            }),
        );
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
        if matches!(
            e,
            Event::Mouse(mouse::Event::CursorMoved { .. }) | Event::Mouse(mouse::Event::CursorLeft)
        ) {
            s.request_redraw();
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

impl<'a, M: 'a> TabSurface<'a, M> {
    pub fn new(appearance: Appearance, content: impl Into<Element<'a, M>>, selected: bool) -> Self {
        Self {
            appearance,
            content: content.into(),
            selected,
        }
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<TabSurface<'a, M>> for Element<'a, M> {
    fn from(tab: TabSurface<'a, M>) -> Self {
        tab.build()
    }
}
