//! Visual feedback around an existing button. The child owns input and commands;
//! this wrapper only observes accepted presses and forwards everything unchanged.
use crate::animation::primitives::{self as motion, Easing, Policy, Ripple, Tween};
use crate::animation::{Element, Renderer};
#[cfg(feature = "svg")]
use iced_core::svg;
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Point, Rectangle, Size, Theme, Vector};
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum RefreshIcon {
    None,
    #[cfg(feature = "svg")]
    Center,
    #[cfg(feature = "svg")]
    Leading,
}

pub fn wrap<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    policy: Policy,
    refresh: RefreshIcon,
    enabled: bool,
) -> Element<'a, M> {
    Element::new(Feedback {
        content: content.into(),
        policy,
        refresh,
        enabled,
    })
}

struct Feedback<'a, M> {
    content: Element<'a, M>,
    policy: Policy,
    refresh: RefreshIcon,
    enabled: bool,
}

struct State {
    now: Instant,
    ripple: Option<Ripple>,
    spin: Tween,
}
impl State {
    fn new(now: Instant) -> Self {
        Self {
            now,
            ripple: None,
            spin: Tween::new(0.0, now),
        }
    }
    fn clear(&mut self) {
        self.ripple = None;
        self.spin = Tween::new(0.0, self.now);
    }
    fn spin(&mut self, policy: Policy) {
        // Repeated clicks retarget the current turn, never queue more turns.
        let angle = self
            .spin
            .value(self.now, policy)
            .rem_euclid(std::f32::consts::TAU);
        self.spin = Tween::new(angle, self.now);
        self.spin.retarget(
            std::f32::consts::TAU,
            self.now,
            motion::FEEDBACK,
            Easing::EaseOut,
            policy,
        );
    }
}

impl<M> Widget<M, Theme, Renderer> for Feedback<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::new(Instant::now()))
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
        if !self.policy.enabled() {
            tree.state.downcast_mut::<State>().clear();
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
        // No added layout node: geometry, hit areas, and anchors stay identical.
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
        let mut messages = Vec::new();
        let available = !shell.is_event_captured();
        let mut child_shell = Shell::new(&mut messages);
        if shell.is_event_captured() {
            child_shell.capture_event();
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut child_shell,
            viewport,
        );
        let accepted = available && self.enabled && child_shell.is_event_captured();
        let activated = !child_shell.is_empty();
        // Commands are published in this very event, without waiting for motion.
        shell.merge(child_shell, std::convert::identity);
        let state = tree.state.downcast_mut::<State>();
        state.now = match event {
            Event::Window(iced_core::window::Event::RedrawRequested(now)) => *now,
            _ => Instant::now(),
        };
        if !self.policy.enabled() {
            state.clear();
            return;
        }
        let bounds = layout.bounds();
        let visible = bounds.intersection(viewport);
        let pressed = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
            Event::Touch(iced_core::touch::Event::FingerPressed { position, .. }) => {
                Some(*position)
            }
            _ => None,
        };
        if let Some(point) =
            pressed.filter(|point| accepted && visible.is_some_and(|clip| clip.contains(*point)))
        {
            state.ripple = Some(Ripple::new(
                Point::new(point.x - bounds.x, point.y - bounds.y),
                bounds.size(),
                state.now,
                self.policy,
            ));
        }
        if activated && !matches!(self.refresh, RefreshIcon::None) {
            state.spin(self.policy);
        }
        if let Some(ripple) = &state.ripple {
            if ripple.sample(state.now, self.policy).is_some() {
                ripple.request_frame(state.now, self.policy, shell);
            } else {
                state.ripple = None;
            }
        }
        state.spin.request_frame(state.now, self.policy, shell);
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
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let state = tree.state.downcast_ref::<State>();
        let text = theme.palette().text;
        #[cfg(feature = "svg")]
        let secondary = theme.extended_palette().background.strong.text;
        if let Some((radius, opacity)) = state
            .ripple
            .as_ref()
            .and_then(|r| r.sample(state.now, self.policy))
            && radius > 0.0
        {
            let origin = state.ripple.as_ref().unwrap().origin;
            renderer.with_layer(clip, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + origin.x - radius, bounds.y + origin.y - radius),
                            Size::new(radius * 2.0, radius * 2.0),
                        ),
                        border: iced_core::Border {
                            radius: radius.into(),
                            ..Default::default()
                        },
                        snap: false,
                        ..Default::default()
                    },
                    text.scale_alpha(opacity),
                )
            });
        }
        #[cfg(feature = "svg")]
        if !matches!(self.refresh, RefreshIcon::None) {
            let x = match self.refresh {
                RefreshIcon::Leading => bounds.x + 6.0,
                _ => bounds.center_x() - 7.0,
            };
            let angle = if state.spin.active(state.now, self.policy) {
                state.spin.value(state.now, self.policy)
            } else {
                0.0
            };
            svg::Renderer::draw_svg(
                renderer,
                svg::Svg::new(refresh_frame(angle)).color(
                    (if self.enabled { text } else { secondary }).scale_alpha(if self.enabled {
                        1.0
                    } else {
                        0.38
                    }),
                ),
                Rectangle::new(
                    Point::new(x, bounds.center_y() - 7.0),
                    Size::new(14.0, 14.0),
                ),
                clip,
            );
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

#[cfg(all(test, feature = "svg"))]
mod tests {
    use super::*;
    use iced_core::renderer::Headless as _;

    const ON: Policy = Policy {
        system_enabled: true,
    };
    const OFF: Policy = Policy {
        system_enabled: false,
    };

    fn button(policy: Policy, enabled: bool) -> Feedback<'static, u8> {
        Feedback {
            content: iced_widget::button(iced_widget::Space::new().width(14).height(16))
                .width(80)
                .height(28)
                .padding(0)
                .on_press_maybe(enabled.then_some(7))
                .style(|_, _| iced_widget::button::Style::default())
                .into(),
            policy,
            refresh: RefreshIcon::Center,
            enabled,
        }
    }
    fn setup(widget: &mut Feedback<'_, u8>) -> (Tree, Renderer, layout::Node) {
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let mut tree = Tree::new(widget as &dyn Widget<u8, Theme, Renderer>);
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(120.0, 40.0)),
        );
        (tree, renderer, node)
    }
    fn event(
        widget: &mut Feedback<'_, u8>,
        tree: &mut Tree,
        renderer: &Renderer,
        node: &layout::Node,
        event: Event,
        point: Point,
    ) -> (Vec<u8>, iced_core::window::RedrawRequest) {
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        widget.update(
            tree,
            &event,
            Layout::new(node),
            mouse::Cursor::Available(point),
            renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(120.0, 40.0)),
        );
        let redraw = shell.redraw_request();
        (messages, redraw)
    }
    fn press() -> Event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    }
    fn release() -> Event {
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
    }

    #[test]
    fn actions_are_immediate_and_feedback_survives_rebuilds() {
        let mut widget = button(ON, true);
        let (mut tree, renderer, node) = setup(&mut widget);
        let point = Point::new(12.0, 9.0);
        assert!(
            event(&mut widget, &mut tree, &renderer, &node, press(), point)
                .0
                .is_empty()
        );
        assert_eq!(
            tree.state
                .downcast_ref::<State>()
                .ripple
                .as_ref()
                .unwrap()
                .origin,
            point
        );
        assert_eq!(
            event(&mut widget, &mut tree, &renderer, &node, release(), point).0,
            vec![7]
        );
        let now = tree.state.downcast_ref::<State>().now;
        widget = button(ON, true);
        widget.diff(&mut tree);
        assert!(tree.state.downcast_ref::<State>().spin.active(now, ON));
        let (messages, redraw) = event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            Event::Window(iced_core::window::Event::RedrawRequested(
                now + motion::QUICK,
            )),
            point,
        );
        assert!(messages.is_empty());
        assert_eq!(redraw, iced_core::window::RedrawRequest::NextFrame);
        let (_, redraw) = event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            Event::Window(iced_core::window::Event::RedrawRequested(
                now + motion::FEEDBACK,
            )),
            point,
        );
        assert_eq!(redraw, iced_core::window::RedrawRequest::Wait);
        assert!(tree.state.downcast_ref::<State>().ripple.is_none());
    }

    #[test]
    fn disabled_buttons_and_other_mouse_buttons_do_not_animate_or_activate() {
        for enabled in [true, false] {
            let mut widget = button(ON, enabled);
            let (mut tree, renderer, node) = setup(&mut widget);
            let point = Point::new(12.0, 9.0);
            let input = if enabled {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            } else {
                press()
            };
            assert!(
                event(&mut widget, &mut tree, &renderer, &node, input, point)
                    .0
                    .is_empty()
            );
            assert!(
                event(&mut widget, &mut tree, &renderer, &node, release(), point)
                    .0
                    .is_empty()
            );
            let state = tree.state.downcast_ref::<State>();
            assert!(state.ripple.is_none());
            assert!(!state.spin.active(state.now, ON));
        }
    }

    #[test]
    fn disabled_motion_keeps_commands_and_suppresses_effects() {
        for policy in [OFF, Policy::DISABLED] {
            let mut widget = button(policy, true);
            let (mut tree, renderer, node) = setup(&mut widget);
            let point = Point::new(12.0, 9.0);
            event(&mut widget, &mut tree, &renderer, &node, press(), point);
            assert_eq!(
                event(&mut widget, &mut tree, &renderer, &node, release(), point).0,
                vec![7]
            );
            let state = tree.state.downcast_ref::<State>();
            assert!(state.ripple.is_none());
            assert!(!state.spin.active(state.now, ON));
        }
    }

    #[test]
    fn policy_change_discards_motion_and_repeated_clicks_remain_bounded() {
        let mut widget = button(ON, true);
        let (mut tree, renderer, node) = setup(&mut widget);
        let point = Point::new(12.0, 9.0);
        for _ in 0..5 {
            event(&mut widget, &mut tree, &renderer, &node, press(), point);
            assert_eq!(
                event(&mut widget, &mut tree, &renderer, &node, release(), point).0,
                vec![7]
            );
        }
        let state = tree.state.downcast_ref::<State>();
        assert!(!state.spin.active(state.now + motion::FEEDBACK, ON));
        widget.policy = OFF;
        widget.diff(&mut tree);
        widget.policy = ON;
        widget.diff(&mut tree);
        let state = tree.state.downcast_ref::<State>();
        assert!(state.ripple.is_none());
        assert!(!state.spin.active(state.now, ON));
    }

    #[test]
    fn cancelled_click_and_previously_captured_input_do_not_activate() {
        let mut widget = button(ON, true);
        let (mut tree, renderer, node) = setup(&mut widget);
        let point = Point::new(12.0, 9.0);
        event(&mut widget, &mut tree, &renderer, &node, press(), point);
        assert!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                release(),
                Point::new(100.0, 35.0)
            )
            .0
            .is_empty()
        );
        let state = tree.state.downcast_ref::<State>();
        assert!(!state.spin.active(state.now, ON));
        widget.policy = OFF;
        widget.diff(&mut tree);
        widget.policy = ON;
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        shell.capture_event();
        widget.update(
            &mut tree,
            &press(),
            Layout::new(&node),
            mouse::Cursor::Available(point),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &node.bounds(),
        );
        assert!(tree.state.downcast_ref::<State>().ripple.is_none());
        assert!(messages.is_empty());
    }

    #[test]
    fn rendered_ripple_is_clipped_and_refresh_really_rotates_on_cpu() {
        let mut widget = button(ON, true);
        let (mut tree, mut renderer, node) = setup(&mut widget);
        let bounds = Rectangle::with_size(Size::new(120.0, 40.0));
        let theme = crate::palette::palette(true, false).theme();
        let mut render = |widget: &Feedback<'_, u8>, tree: &Tree| {
            renderer.reset(bounds);
            widget.draw(
                tree,
                &mut renderer,
                &theme,
                &renderer::Style::default(),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &bounds,
            );
            renderer.screenshot(Size::new(120, 40), 1.0, iced_core::Color::BLACK)
        };
        let idle = render(&widget, &tree);
        let now = tree.state.downcast_ref::<State>().now;
        {
            let state = tree.state.downcast_mut::<State>();
            state.ripple = Some(Ripple::new(Point::new(1.0, 1.0), node.size(), now, ON));
            state.now = now + motion::QUICK;
        }
        let ripple = render(&widget, &tree);
        assert_ne!(idle, ripple);
        for y in 0..40usize {
            for x in 0..120usize {
                if x >= 80 || y >= 28 {
                    let i = (y * 120 + x) * 4;
                    assert_eq!(&idle[i..i + 4], &ripple[i..i + 4]);
                }
            }
        }
        let state = tree.state.downcast_mut::<State>();
        state.ripple = None;
        state.spin(ON);
        state.now += motion::FEEDBACK / 4;
        let rotated = render(&widget, &tree);
        assert_ne!(
            idle, rotated,
            "CPU rendering must actually change the refresh glyph"
        );
    }
}

#[cfg(feature = "svg")]
pub fn refresh_frame(angle: f32) -> svg::Handle {
    const FRAMES: usize = 32;
    static CACHE: std::sync::OnceLock<[svg::Handle; FRAMES]> = std::sync::OnceLock::new();
    let frames = CACHE.get_or_init(|| std::array::from_fn(|i| {
        svg::Handle::from_memory(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><g transform="rotate({} 12 12)" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 7a8 8 0 1 0 1 8M19 2v5h-5"/></g></svg>"#,
            i as f32 * 360.0 / FRAMES as f32
        ).into_bytes())
    }));
    let index = ((angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU * FRAMES as f32)
        .round() as usize)
        % FRAMES;
    frames[index].clone()
}

// Compatibility path: there is one shared public button builder.
pub use crate::components::Button;

#[cfg(test)]
mod builder_tests {
    use super::*;
    #[test]
    fn builder_dispatches_once_without_waiting_for_motion() {
        for policy in [
            Policy {
                system_enabled: true,
            },
            Policy::DISABLED,
        ] {
            let mut widget: Element<'_, u8> = Button::custom(
                crate::components::Appearance::fixed(
                    iced_core::Font::DEFAULT,
                    crate::palette::palette(true, false),
                ),
                policy,
                iced_widget::button(iced_widget::Space::new())
                    .width(80)
                    .height(30),
            )
            .on_press_maybe(Some(7))
            .into();
            let mut tree = Tree::new(&widget);
            let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
            let size = Size::new(80.0, 30.0);
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(size, size),
            );
            let mut messages = vec![];
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                let mut shell = Shell::new(&mut messages);
                widget.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(Point::new(10.0, 10.0)),
                    &renderer,
                    &mut iced_core::clipboard::Null,
                    &mut shell,
                    &node.bounds(),
                );
                assert!(!shell.is_layout_invalid());
            }
            assert_eq!(messages, vec![7]);
            assert_eq!(node.size(), size);
            let state = tree.state.downcast_ref::<State>();
            assert_eq!(state.ripple.is_some(), policy.enabled());
        }
    }
}
