// SPDX-License-Identifier: Apache-2.0
//! Controlled switches with immediate native button actions and paint-only thumb motion.
use super::Appearance;
use crate::{
    animation::{
        Element, Renderer,
        primitives::{Easing, QUICK, Tween},
    },
    motion::Policy,
    theme::color,
    visual::{Fill, Rgba, Shape, SurfaceStyle},
};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Renderer as _, Shell, Size, Theme, Vector, Widget,
    layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_widget::{Space, button};
use std::time::Instant;
pub struct ToggleSwitch<'a, M> {
    appearance: Appearance,
    policy: Policy,
    value: bool,
    enabled: bool,
    action: Option<Box<dyn Fn(bool) -> M + 'a>>,
    id: Option<iced_core::widget::Id>,
}
impl<'a, M: Clone + 'a> ToggleSwitch<'a, M> {
    pub fn new(appearance: Appearance, policy: Policy, value: bool) -> Self {
        Self {
            appearance,
            policy,
            value,
            enabled: true,
            action: None,
            id: None,
        }
    }
    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn on_toggle(mut self, action: impl Fn(bool) -> M + 'a) -> Self {
        self.action = Some(Box::new(action));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let enabled = self.enabled && self.action.is_some();
        let action = if enabled {
            self.action.map(|f| f(!self.value))
        } else {
            None
        };
        let button = button(Space::new().width(44).height(24))
            .width(44)
            .height(24)
            .padding(0)
            .style(|_, _| button::Style::default())
            .on_press_maybe(action);
        let inner = Element::new(SwitchWidget {
            appearance: self.appearance,
            policy: self.policy,
            value: self.value,
            enabled,
            button: button.into(),
        });
        let mut control = super::KeyboardControl::new(self.appearance, inner).enabled(enabled);
        if let Some(id) = self.id {
            control = control.id(id);
        }
        control.into()
    }
}
impl<'a, M: Clone + 'a> From<ToggleSwitch<'a, M>> for Element<'a, M> {
    fn from(s: ToggleSwitch<'a, M>) -> Self {
        s.build()
    }
}
struct SwitchWidget<'a, M> {
    appearance: Appearance,
    policy: Policy,
    value: bool,
    enabled: bool,
    button: Element<'a, M>,
}
struct State {
    position: Tween,
    now: Instant,
}
fn paint(r: &mut Renderer, bounds: Rectangle, surface: SurfaceStyle, fill: iced_core::Color) {
    let mut bg = Some(fill.into());
    let mut border = iced_core::Border {
        radius: 12.0.into(),
        ..Default::default()
    };
    let mut shadow = iced_core::Shadow::default();
    surface.apply(&mut bg, &mut border, &mut shadow);
    for layer in surface.layers().iter().skip(1).flatten() {
        r.fill_quad(
            renderer::Quad {
                bounds,
                border,
                shadow: layer.native(surface.opacity()),
                ..Default::default()
            },
            iced_core::Color::TRANSPARENT,
        );
    }
    r.fill_quad(
        renderer::Quad {
            bounds,
            border,
            shadow,
            ..Default::default()
        },
        bg.unwrap(),
    );
    super::surface::paint_inset(r, bounds, surface);
}
impl<M> Widget<M, Theme, Renderer> for SwitchWidget<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        let now = Instant::now();
        tree::State::new(State {
            position: Tween::new(if self.value { 1. } else { 0. }, now),
            now,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.button)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(std::slice::from_ref(&self.button));
        let state = t.state.downcast_mut::<State>();
        state.now = Instant::now();
        state.position.retarget(
            if self.value { 1. } else { 0. },
            state.now,
            QUICK,
            Easing::EaseOut,
            self.policy,
        );
    }
    fn size(&self) -> Size<Length> {
        Size::new(44.into(), 24.into())
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.button
            .as_widget_mut()
            .layout(&mut t.children[0], r, limits)
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let tokens = self.appearance.tokens(theme);
        let state = t.state.downcast_ref::<State>();
        let status = if !self.enabled {
            button::Status::Disabled
        } else if cursor.is_over(l.bounds()) {
            button::Status::Hovered
        } else {
            button::Status::Active
        };
        let mut track = (if self.value {
            tokens.visual.switch_on
        } else {
            tokens.visual.switch_off
        })
        .map(|s| s.button(status))
        .unwrap_or(SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(
                if self.value {
                    tokens.accent
                } else {
                    tokens.border
                },
                1.,
            ))),
            shape: Some(Shape::Pill),
            opacity: Some(1.),
            ..Default::default()
        });
        if !self.enabled {
            track.opacity = Some(track.opacity() * tokens.opacity_disabled as f32);
        }
        paint(
            r,
            l.bounds(),
            track,
            color(if self.value {
                tokens.accent
            } else {
                tokens.border
            }),
        );
        let progress = state.position.value(state.now, self.policy);
        let thumb = Rectangle {
            x: l.bounds().x + 3. + 20. * progress,
            y: l.bounds().y + 3.,
            width: 18.,
            height: 18.,
        };
        let mut surface = tokens.visual.switch_thumb.unwrap_or_default();
        if !self.enabled {
            surface.opacity = Some(surface.opacity() * tokens.opacity_disabled as f32);
        }
        paint(
            r,
            thumb,
            surface,
            tokens.contrast(if self.value {
                tokens.accent
            } else {
                tokens.border
            }),
        );
    }
    fn update(
        &mut self,
        t: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        r: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.button.as_widget_mut().update(
            &mut t.children[0],
            event,
            l,
            cursor,
            r,
            clipboard,
            shell,
            viewport,
        );
        let state = t.state.downcast_mut::<State>();
        if let Event::Window(iced_core::window::Event::RedrawRequested(now)) = event {
            state.now = *now;
        }
        state.position.request_frame(state.now, self.policy, shell);
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, op: &mut dyn Operation) {
        self.button
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, op);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.button
            .as_widget()
            .mouse_interaction(&t.children[0], l, cursor, viewport, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        viewport: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.button
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, viewport, tr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn switch(value: bool, enabled: bool, policy: Policy) -> Element<'static, bool> {
        ToggleSwitch::new(
            Appearance::fixed(
                iced_core::Font::DEFAULT,
                crate::palette::palette(true, false),
            ),
            policy,
            value,
        )
        .enabled(enabled)
        .on_toggle(|v| v)
        .build()
    }
    #[test]
    fn actions_are_immediate_and_disabled_switches_do_not_publish() {
        for enabled in [true, false] {
            let mut widget = switch(false, enabled, Policy::DISABLED);
            let mut tree = Tree::new(&widget);
            let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
            let bounds = Rectangle::with_size(Size::new(44., 24.));
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, bounds.size()),
            );
            let mut messages = vec![];
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                widget.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(iced_core::Point::new(20., 12.)),
                    &renderer,
                    &mut iced_core::clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &bounds,
                );
            }
            assert_eq!(messages, if enabled { vec![true] } else { vec![] });
        }
    }
    #[test]
    fn motion_retargets_and_settles_when_disabled() {
        let on = Policy {
            system_enabled: true,
        };
        let widget = switch(false, true, on);
        let mut tree = Tree::new(&widget);
        let next = switch(true, true, on);
        tree.diff(next.as_widget());
        let state = tree.children[0].state.downcast_ref::<State>();
        assert!(state.position.value(state.now, on) < 1.);
        assert_eq!(state.position.value(state.now + QUICK, on), 1.);
        {
            let policy = Policy::DISABLED;
            let next = switch(false, true, policy);
            tree.diff(next.as_widget());
            let state = tree.children[0].state.downcast_ref::<State>();
            assert_eq!(state.position.value(state.now, policy), 0.);
        }
    }
}
