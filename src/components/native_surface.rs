// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Shadow decoration that delegates the native tree tag and state unchanged.
use super::Appearance;
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Renderer as _, Shell, Size, Theme, Vector, Widget,
    layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Input,
    Checkbox,
}
pub(crate) struct NativeSurface<'a, M> {
    content: Element<'a, M>,
    appearance: Appearance,
    kind: Kind,
    enabled: bool,
}
impl<M> Widget<M, Theme, Renderer> for NativeSurface<'_, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, t: &mut Tree) {
        self.content.as_widget().diff(t);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(t, r, l)
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
        let (states, bounds, focused) = match self.kind {
            Kind::Input => {
                type InputState = iced_widget::text_input::State<
                    <Renderer as iced_core::text::Renderer>::Paragraph,
                >;
                (
                    tokens.visual.input,
                    l.bounds(),
                    t.state.downcast_ref::<InputState>().is_focused(),
                )
            }
            Kind::Checkbox => (
                tokens.visual.checkbox,
                l.children()
                    .next()
                    .map(|l| l.bounds())
                    .unwrap_or(l.bounds()),
                false,
            ),
        };
        let surface = (if !self.enabled {
            states.disabled
        } else if focused {
            states.focused
        } else if c.is_over(l.bounds()) {
            states.hovered
        } else {
            None
        })
        .unwrap_or_default()
        .inherit(states.normal);

        for layer in surface.layers().iter().flatten() {
            r.fill_quad(
                renderer::Quad {
                    bounds,
                    border: iced_core::Border {
                        radius: surface.shape.map(|s| s.radius()).unwrap_or(2.0.into()),
                        ..Default::default()
                    },
                    shadow: layer.native(surface.opacity()),
                    ..Default::default()
                },
                iced_core::Color::TRANSPARENT,
            );
        }

        self.content.as_widget().draw(t, r, th, s, l, c, v);
        super::surface::paint_inset(r, bounds, surface);
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
        self.content.as_widget_mut().update(t, e, l, c, r, cb, s, v);
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content.as_widget_mut().operate(t, l, r, o);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(t, l, c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(t, l, r, v, tr)
    }
}

pub(crate) fn decorate<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    appearance: Appearance,
    kind: Kind,
    enabled: bool,
) -> Element<'a, M> {
    Element::new(NativeSurface {
        content: content.into(),
        appearance,
        kind,
        enabled,
    })
}
