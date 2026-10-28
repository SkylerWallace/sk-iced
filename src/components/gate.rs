// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Optional input gating. Mount the gate for both enabled states to retain child state.

use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
pub struct InteractionGate<'a, M> {
    content: Element<'a, M>,
    enabled: bool,
}
#[derive(Default)]
struct GateState {
    disabled: bool,
}
impl<M> Widget<M, Theme, Renderer> for InteractionGate<'_, M> {
    fn tag(&self) -> iced_core::widget::tree::Tag {
        iced_core::widget::tree::Tag::of::<GateState>()
    }
    fn state(&self) -> iced_core::widget::tree::State {
        iced_core::widget::tree::State::new(GateState::default())
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
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l);
        let state = t.state.downcast_mut::<GateState>();
        if !self.enabled && !state.disabled {
            let mut ignored = Vec::new();
            let mut shell = Shell::new(&mut ignored);
            for event in [
                Event::Window(iced_core::window::Event::Unfocused),
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            ] {
                self.content.as_widget_mut().update(
                    &mut t.children[0],
                    &event,
                    Layout::new(&node),
                    mouse::Cursor::Unavailable,
                    r,
                    &mut iced_core::clipboard::Null,
                    &mut shell,
                    &node.bounds(),
                );
            }
        }
        t.state.downcast_mut::<GateState>().disabled = !self.enabled;
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
        self.content.as_widget().draw(
            &t.children[0],
            r,
            th,
            s,
            l,
            if self.enabled {
                c
            } else {
                mouse::Cursor::Unavailable
            },
            v,
        );
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
        if self.enabled {
            self.content
                .as_widget_mut()
                .update(&mut t.children[0], e, l, c, r, cb, s, v);
        }
    }

    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if !self.enabled {
            return;
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
        if self.enabled {
            self.content
                .as_widget()
                .mouse_interaction(&t.children[0], l, c, v, r)
        } else {
            mouse::Interaction::default()
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
        if !self.enabled {
            return None;
        }
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, tr)
    }
}

impl<'a, M: 'a> InteractionGate<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>, enabled: bool) -> Self {
        Self {
            content: content.into(),
            enabled,
        }
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<InteractionGate<'a, M>> for Element<'a, M> {
    fn from(gate: InteractionGate<'a, M>) -> Self {
        gate.build()
    }
}
