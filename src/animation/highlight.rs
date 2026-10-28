//! Absolute-time paint-only highlights; no application paths or operation state.
use crate::animation::{Element, Renderer};
use crate::motion::Policy;
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
use std::time::{Duration, Instant};
pub const DURATION: Duration = Duration::from_millis(400);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pulse {
    started: Instant,
}
impl Pulse {
    pub fn started(self) -> Instant {
        self.started
    }
    pub fn new(policy: Policy) -> Option<Self> {
        policy.enabled().then(|| Self {
            started: Instant::now(),
        })
    }
    pub fn amount(self, now: Instant, policy: Policy) -> f32 {
        if !policy.enabled() {
            return 0.0;
        }
        let t = now.saturating_duration_since(self.started).as_secs_f32() / DURATION.as_secs_f32();
        (1.0 - t).clamp(0.0, 1.0).powi(2)
    }
}
#[derive(Clone, Copy)]
pub enum Style {
    Item,
    Destination,
}
pub fn wrap<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    pulse: Option<Pulse>,
    policy: Policy,
    style: Style,
) -> Element<'a, M> {
    Element::new(Highlight {
        content: content.into(),
        pulse,
        policy,
        style,
    })
}
struct Highlight<'a, M> {
    content: Element<'a, M>,
    pulse: Option<Pulse>,
    policy: Policy,
    style: Style,
}
struct State {
    now: Instant,
    suppressed: Option<Pulse>,
}
impl<M> Widget<M, Theme, Renderer> for Highlight<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            now: Instant::now(),
            suppressed: None,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
        if !self.policy.enabled() {
            tree.state.downcast_mut::<State>().suppressed = self.pulse;
        }
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let state = tree.state.downcast_mut::<State>();
        if let Event::Window(iced_core::window::Event::RedrawRequested(now)) = event {
            state.now = state.now.max(*now);
        }
        if !self.policy.enabled() {
            state.suppressed = self.pulse;
        }
        if self
            .pulse
            .is_some_and(|p| Some(p) != state.suppressed && p.amount(state.now, self.policy) > 0.0)
            && layout.bounds().intersection(viewport).is_some()
        {
            shell.request_redraw();
        }
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        let state = tree.state.downcast_ref::<State>();
        let amount = self
            .pulse
            .filter(|p| Some(*p) != state.suppressed)
            .map_or(0.0, |p| p.amount(state.now, self.policy));
        if amount <= 0.0 {
            return;
        }
        if let Some(clip) = layout.bounds().intersection(viewport) {
            let accent = theme.palette().primary;
            let destination = matches!(self.style, Style::Destination);
            renderer.with_layer(clip, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: layout.bounds(),
                        border: iced_core::Border {
                            color: accent.scale_alpha(if destination {
                                amount * 0.75
                            } else {
                                0.0
                            }),
                            width: if destination { 2.0 } else { 0.0 },
                            radius: 3.0.into(),
                        },
                        ..Default::default()
                    },
                    accent.scale_alpha(amount * if destination { 0.06 } else { 0.22 }),
                )
            });
        }
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_core::renderer::Headless as _;
    const ON: Policy = Policy {
        system_enabled: true,
    };
    const OFF: Policy = Policy::DISABLED;
    fn widget(pulse: Option<Pulse>, policy: Policy) -> Element<'static, u8> {
        wrap(
            iced_widget::button(iced_widget::Space::new())
                .width(80)
                .height(24)
                .on_press(7)
                .style(|_, _| iced_widget::button::Style::default()),
            pulse,
            policy,
            Style::Item,
        )
    }
    #[test]
    fn actions_are_immediate_pixels_are_clipped_and_frames_stop() {
        let pulse = Pulse::new(ON).unwrap();
        let mut widget = widget(Some(pulse), ON);
        let mut tree = Tree::new(&widget);
        let mut renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let node = widget.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(100.0, 30.0)),
        );
        let mut messages = vec![];
        let mut send = |event: Event, tree: &mut Tree, widget: &mut Element<'_, u8>| {
            let mut shell = Shell::new(&mut messages);
            widget.as_widget_mut().update(
                tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(iced_core::Point::new(20.0, 12.0)),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
                &node.bounds(),
            );
            assert!(!shell.is_layout_invalid());
            shell.redraw_request()
        };
        send(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            &mut tree,
            &mut widget,
        );
        send(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            &mut tree,
            &mut widget,
        );
        assert_eq!(
            send(
                Event::Window(iced_core::window::Event::RedrawRequested(pulse.started)),
                &mut tree,
                &mut widget
            ),
            iced_core::window::RedrawRequest::NextFrame
        );
        assert_eq!(
            send(
                Event::Window(iced_core::window::Event::RedrawRequested(
                    pulse.started + DURATION
                )),
                &mut tree,
                &mut widget
            ),
            iced_core::window::RedrawRequest::Wait
        );
        assert_eq!(messages, vec![7]);
        tree.state.downcast_mut::<State>().now = pulse.started;
        let bounds = Rectangle::with_size(Size::new(100.0, 30.0));
        let clip = Rectangle::new(iced_core::Point::new(10.0, 0.0), Size::new(20.0, 24.0));
        renderer.reset(bounds);
        widget.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &clip,
        );
        let pixels = renderer.screenshot(Size::new(100, 30), 1.0, iced_core::Color::BLACK);
        assert_eq!(
            &pixels[(12 * 100 + 5) * 4..(12 * 100 + 5) * 4 + 3],
            &[0, 0, 0]
        );
        assert_ne!(
            &pixels[(12 * 100 + 20) * 4..(12 * 100 + 20) * 4 + 3],
            &[0, 0, 0]
        );
    }
    #[test]
    fn disabled_policy_never_replays_and_recycled_widgets_use_absolute_time() {
        let pulse = Pulse::new(ON).unwrap();
        assert_eq!(pulse.amount(pulse.started, OFF), 0.0);
        assert_eq!(pulse.amount(pulse.started, Policy::DISABLED), 0.0);
        let mut widget = widget(Some(pulse), ON);
        let mut tree = Tree::new(&widget);
        widget = super::tests::widget(Some(pulse), OFF);
        tree.diff(&widget);
        widget = super::tests::widget(Some(pulse), ON);
        tree.diff(&widget);
        assert_eq!(tree.state.downcast_ref::<State>().suppressed, Some(pulse));
        let old = Pulse {
            started: Instant::now() - DURATION,
        };
        assert_eq!(old.amount(Instant::now(), ON), 0.0);
    }
}
