//! Short, interruptible reveals. Logical open/closed state takes effect at once;
//! closing content only paints and never receives input, focus, or overlays.
use crate::animation::primitives::{self as motion, Easing, Policy, Tween};
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
use std::{cell::OnceCell, time::Instant};

#[derive(Clone, Copy)]
pub enum Edge {
    Left,
    Top,
    Bottom,
}

/// Shared by ordinary reveal widgets and anchored popup overlays.
pub struct State {
    open: bool,
    now: Instant,
    progress: Tween,
    cancel_input: bool,
}
impl State {
    pub fn new(open: bool, now: Instant) -> Self {
        Self {
            open,
            now,
            progress: Tween::new(if open { 1.0 } else { 0.0 }, now),
            cancel_input: false,
        }
    }
    pub fn sync(&mut self, open: bool, policy: Policy, now: Instant) {
        if open != self.open {
            if !open {
                self.cancel_input = true;
            }
            self.open = open;
            self.now = now.max(self.now);
            self.progress.retarget(
                if open { 1.0 } else { 0.0 },
                self.now,
                if open {
                    motion::STANDARD
                } else {
                    motion::QUICK
                },
                Easing::EaseOut,
                policy,
            );
        }
        self.progress.settle_if_disabled(policy);
    }
    pub fn tick<M>(&mut self, now: Instant, policy: Policy, shell: &mut Shell<'_, M>) {
        self.now = now.max(self.now);
        self.progress.settle_if_disabled(policy);
        self.progress.request_frame(self.now, policy, shell);
    }
    pub fn value(&self, policy: Policy) -> f32 {
        self.progress.value(self.now, policy)
    }
    pub fn present(&self, policy: Policy) -> bool {
        self.open || self.value(policy) > 0.0
    }
    pub fn take_cancel_input(&mut self) -> bool {
        std::mem::take(&mut self.cancel_input)
    }
    pub fn clip(&self, bounds: Rectangle, edge: Edge, policy: Policy) -> Option<Rectangle> {
        let progress = self.value(policy);
        if progress <= 0.0 {
            return None;
        }
        Some(match edge {
            Edge::Left => Rectangle {
                width: bounds.width * progress,
                ..bounds
            },
            Edge::Top => Rectangle {
                height: bounds.height * progress,
                ..bounds
            },
            Edge::Bottom => Rectangle {
                y: bounds.y + bounds.height * (1.0 - progress),
                height: bounds.height * progress,
                ..bounds
            },
        })
    }
}

/// Cancel gestures in the outgoing view without delivering any commands. This
/// also prevents a quick reopen from reviving a button press or resize drag.
pub fn cancel_input<M>(
    content: &mut Element<'_, M>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
) {
    for event in [
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        Event::Window(iced_core::window::Event::Unfocused),
    ] {
        let mut ignored = Vec::new();
        content.as_widget_mut().update(
            tree,
            &event,
            layout,
            mouse::Cursor::Unavailable,
            renderer,
            clipboard,
            &mut Shell::new(&mut ignored),
            &Rectangle::with_size(Size::ZERO),
        );
    }
}

/// Redraws initialize Iced button/text styling as well as animation clocks.
/// Outgoing content gets these lifecycle events, but no input or app messages.
pub fn update_visual<M>(
    content: &mut Element<'_, M>,
    tree: &mut Tree,
    event: &Event,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    viewport: &Rectangle,
) {
    if matches!(
        event,
        Event::Window(iced_core::window::Event::RedrawRequested(_))
    ) {
        let mut ignored = Vec::new();
        content.as_widget_mut().update(
            tree,
            event,
            layout,
            mouse::Cursor::Unavailable,
            renderer,
            clipboard,
            &mut Shell::new(&mut ignored),
            viewport,
        );
    }
}

/// Paint-only reveal for a full-size layer in a stack. Workspace layout stays fixed.
pub fn horizontal<'a, M: 'a>(
    open: bool,
    policy: Policy,
    build: impl Fn() -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    Element::new(Reveal {
        open,
        policy,
        collapse: false,
        edge: Edge::Left,
        content: OnceCell::new(),
        build: Box::new(build),
    })
}
/// Height reveal for dropdown content. The full child layout is clipped, never
/// rebuilt per animation frame. Only the outer height and parent layout change.
pub fn vertical<'a, M: 'a>(
    open: bool,
    policy: Policy,
    build: impl Fn() -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    Element::new(Reveal {
        open,
        policy,
        collapse: true,
        edge: Edge::Top,
        content: OnceCell::new(),
        build: Box::new(build),
    })
}

struct Reveal<'a, M> {
    open: bool,
    policy: Policy,
    collapse: bool,
    edge: Edge,
    content: OnceCell<Element<'a, M>>,
    build: Box<dyn Fn() -> Element<'a, M> + 'a>,
}
impl<'a, M> Reveal<'a, M> {
    fn content(&self) -> &Element<'a, M> {
        self.content.get_or_init(|| (self.build)())
    }
    fn clip(&self, tree: &Tree, layout: Layout<'_>, viewport: &Rectangle) -> Option<Rectangle> {
        if tree.state.downcast_ref::<State>().value(self.policy) <= 0.0 {
            return None;
        }
        let bounds = layout.bounds();
        if self.collapse {
            bounds
                .intersection(viewport)
                .filter(|b| b.height > 0.0 && b.width > 0.0)
        } else {
            tree.state
                .downcast_ref::<State>()
                .clip(bounds, self.edge, self.policy)?
                .intersection(viewport)
        }
    }
}
impl<M> Widget<M, Theme, Renderer> for Reveal<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.open, Instant::now()))
    }
    fn children(&self) -> Vec<Tree> {
        if self.open {
            vec![Tree::new(self.content())]
        } else {
            vec![]
        }
    }
    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        state.sync(self.open, self.policy, Instant::now());
        if state.present(self.policy) {
            tree.diff_children(std::slice::from_ref(self.content()));
        } else {
            tree.children.clear();
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fill,
            if self.collapse {
                Length::Shrink
            } else {
                Length::Fill
            },
        )
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let progress = tree.state.downcast_ref::<State>().value(self.policy);
        if !tree.state.downcast_ref::<State>().present(self.policy) {
            return layout::Node::new(limits.resolve(
                Length::Fill,
                if self.collapse {
                    Length::Shrink
                } else {
                    Length::Fill
                },
                Size::ZERO,
            ));
        }
        self.content();
        let child = self.content.get_mut().unwrap().as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, limits.max()),
        );
        let size = if self.collapse {
            Size::new(child.size().width, child.size().height * progress)
        } else {
            child.size()
        };
        layout::Node::with_children(size, vec![child])
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
        let state = tree.state.downcast_mut::<State>();
        let previous = state.value(self.policy);
        let now = match event {
            Event::Window(iced_core::window::Event::RedrawRequested(now)) => *now,
            _ => Instant::now(),
        };
        state.tick(now, self.policy, shell);
        if self.collapse && previous != state.value(self.policy) {
            shell.invalidate_layout();
        }
        if state.take_cancel_input()
            && let (Some(content), Some(child_layout), Some(child_tree)) = (
                self.content.get_mut(),
                layout.children().next(),
                tree.children.first_mut(),
            )
        {
            cancel_input(content, child_tree, child_layout, renderer, clipboard);
        }
        // Closing visuals must not dispatch actions or expose nested overlays.
        if !self.open {
            if !tree.state.downcast_ref::<State>().present(self.policy) {
                // Release outgoing rows and widget state without needing an app
                // message or keeping a hidden animation clock alive.
                tree.children.clear();
                self.content.take();
                return;
            }
            if tree.state.downcast_ref::<State>().present(self.policy)
                && let (Some(content), Some(child_layout), Some(child_tree)) = (
                    self.content.get_mut(),
                    layout.children().next(),
                    tree.children.first_mut(),
                )
            {
                update_visual(
                    content,
                    child_tree,
                    event,
                    child_layout,
                    renderer,
                    clipboard,
                    viewport,
                );
            }
            return;
        }
        if tree.children.is_empty() {
            return;
        }
        let Some(child_layout) = layout.children().next() else {
            return;
        };
        let clip = self.clip(tree, layout, viewport);
        let pointer_inside = clip.is_some_and(|clip| cursor.is_over(clip));
        // Forward releases/moves even outside the clip, with no cursor, so child
        // buttons and scrollbars can cancel an in-progress gesture normally.
        self.content.get_mut().unwrap().as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout,
            if pointer_inside {
                cursor
            } else {
                mouse::Cursor::Unavailable
            },
            renderer,
            clipboard,
            shell,
            &clip.unwrap_or(Rectangle::with_size(Size::ZERO)),
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
        let Some(clip) = self.clip(tree, layout, viewport) else {
            return;
        };
        let Some(child) = self.content.get() else {
            return;
        };
        let Some(child_layout) = layout.children().next() else {
            return;
        };
        if tree.children.is_empty() {
            return;
        }
        renderer.with_layer(clip, |renderer| {
            child.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                child_layout,
                if self.open && cursor.is_over(clip) {
                    cursor
                } else {
                    mouse::Cursor::Unavailable
                },
                &clip,
            )
        });
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.open
            && !tree.children.is_empty()
            && let Some(child_layout) = layout.children().next()
        {
            self.content.get_mut().unwrap().as_widget_mut().operate(
                &mut tree.children[0],
                child_layout,
                renderer,
                operation,
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
        if !self.open || tree.children.is_empty() {
            return mouse::Interaction::default();
        }
        let Some(clip) = self.clip(tree, layout, viewport) else {
            return mouse::Interaction::default();
        };
        if !cursor.is_over(clip) {
            return mouse::Interaction::default();
        }
        let Some(child_layout) = layout.children().next() else {
            return mouse::Interaction::default();
        };
        self.content.get().unwrap().as_widget().mouse_interaction(
            &tree.children[0],
            child_layout,
            cursor,
            &clip,
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
        if !self.open || tree.children.is_empty() {
            return None;
        }
        let clip = self.clip(tree, layout, viewport)?;
        self.content.get_mut()?.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            &clip,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_core::renderer::Headless as _;
    use std::{cell::Cell, rc::Rc};
    const ON: Policy = Policy {
        system_enabled: true,
    };

    fn view(
        open: bool,
        collapse: bool,
        policy: Policy,
        builds: Rc<Cell<usize>>,
    ) -> Element<'static, u8> {
        let build = move || {
            builds.set(builds.get() + 1);
            iced_widget::button(iced_widget::Space::new())
                .width(100)
                .height(80)
                .on_press(7)
                .style(|_, _| iced_widget::button::Style {
                    background: Some(iced_core::Color::WHITE.into()),
                    ..Default::default()
                })
                .into()
        };
        if collapse {
            vertical(open, policy, build)
        } else {
            horizontal(open, policy, build)
        }
    }
    fn renderer() -> Renderer {
        Renderer::new(iced_core::Font::DEFAULT, 13.0.into())
    }
    fn layout(view: &mut Element<'_, u8>, tree: &mut Tree, renderer: &Renderer) -> layout::Node {
        view.as_widget_mut().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(200.0, 120.0)),
        )
    }
    fn event(
        view: &mut Element<'_, u8>,
        tree: &mut Tree,
        renderer: &Renderer,
        node: &layout::Node,
        event: Event,
        cursor: mouse::Cursor,
    ) -> (Vec<u8>, iced_core::window::RedrawRequest, bool) {
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        view.as_widget_mut().update(
            tree,
            &event,
            Layout::new(node),
            cursor,
            renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(200.0, 120.0)),
        );
        let redraw = shell.redraw_request();
        let invalid = shell.is_layout_invalid();
        (messages, redraw, invalid)
    }
    fn redraw(now: Instant) -> Event {
        Event::Window(iced_core::window::Event::RedrawRequested(now))
    }
    fn mouse(pressed: bool) -> Event {
        Event::Mouse(if pressed {
            mouse::Event::ButtonPressed(mouse::Button::Left)
        } else {
            mouse::Event::ButtonReleased(mouse::Button::Left)
        })
    }

    #[test]
    fn closed_views_are_lazy_and_frames_do_not_rebuild_content() {
        let builds = Rc::new(Cell::new(0));
        let mut widget = view(false, true, ON, builds.clone());
        let mut tree = Tree::new(&widget);
        let renderer = renderer();
        assert_eq!(layout(&mut widget, &mut tree, &renderer).size().height, 0.0);
        assert_eq!(builds.get(), 0);
        widget = view(true, true, ON, builds.clone());
        tree.diff(&widget);
        let start = tree.state.downcast_ref::<State>().now;
        let mut node = layout(&mut widget, &mut tree, &renderer);
        assert_eq!(builds.get(), 1);
        for ms in [0, 30, 60, 90, 120, 150] {
            let (_, request, _) = event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                redraw(start + std::time::Duration::from_millis(ms)),
                mouse::Cursor::Unavailable,
            );
            node = layout(&mut widget, &mut tree, &renderer);
            assert_eq!(builds.get(), 1);
            assert_eq!(
                request,
                if ms < 150 {
                    iced_core::window::RedrawRequest::NextFrame
                } else {
                    iced_core::window::RedrawRequest::Wait
                }
            );
        }
        assert_eq!(node.size().height, 80.0);
        widget = view(false, true, ON, builds.clone());
        tree.diff(&widget);
        let now = tree.state.downcast_ref::<State>().now;
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            redraw(now + motion::QUICK),
            mouse::Cursor::Unavailable,
        );
        assert_eq!(layout(&mut widget, &mut tree, &renderer).size().height, 0.0);
        let prior = builds.get();
        widget = view(false, true, ON, builds.clone());
        tree.diff(&widget);
        assert_eq!(
            builds.get(),
            prior,
            "settled hidden content must not be built"
        );
    }

    #[test]
    fn interruption_is_continuous_and_disabled_policy_snaps() {
        let now = Instant::now();
        let mut state = State::new(false, now);
        state.sync(true, ON, now);
        let mut messages: Vec<()> = vec![];
        state.tick(
            now + motion::STANDARD / 2,
            ON,
            &mut Shell::new(&mut messages),
        );
        let partial = state.value(ON);
        assert!(partial > 0.0 && partial < 1.0);
        state.sync(false, ON, now + motion::STANDARD / 2);
        assert_eq!(state.value(ON), partial);
        state.sync(true, ON, now + motion::STANDARD / 2);
        assert_eq!(state.value(ON), partial);
        state.sync(true, Policy::DISABLED, now + motion::STANDARD / 2);
        assert_eq!(state.value(ON), 1.0);
        state.sync(false, Policy::DISABLED, now + motion::STANDARD);
        assert_eq!(state.value(ON), 0.0);
    }

    #[test]
    fn closing_content_blocks_input_and_reopen_cancels_stale_presses() {
        let builds = Rc::new(Cell::new(0));
        let mut widget = view(true, false, ON, builds.clone());
        let mut tree = Tree::new(&widget);
        let renderer = renderer();
        let node = layout(&mut widget, &mut tree, &renderer);
        let cursor = mouse::Cursor::Available(iced_core::Point::new(10.0, 10.0));
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            mouse(true),
            cursor,
        );
        assert_eq!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                mouse(false),
                cursor
            )
            .0,
            vec![7]
        );
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            mouse(true),
            cursor,
        );
        widget = view(false, false, ON, builds.clone());
        tree.diff(&widget);
        let closed = layout(&mut widget, &mut tree, &renderer);
        assert!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &closed,
                mouse(false),
                cursor
            )
            .0
            .is_empty()
        );
        assert_eq!(
            widget.as_widget().mouse_interaction(
                &tree,
                Layout::new(&closed),
                cursor,
                &node.bounds(),
                &renderer
            ),
            mouse::Interaction::None
        );
        assert!(
            widget
                .as_widget_mut()
                .overlay(
                    &mut tree,
                    Layout::new(&closed),
                    &renderer,
                    &node.bounds(),
                    Vector::ZERO
                )
                .is_none()
        );
        widget = view(true, false, ON, builds);
        tree.diff(&widget);
        let reopened = layout(&mut widget, &mut tree, &renderer);
        assert!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &reopened,
                mouse(false),
                cursor
            )
            .0
            .is_empty()
        );
    }

    #[test]
    fn closing_children_are_excluded_from_focus_operations() {
        struct Count(usize);
        impl Operation for Count {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn container(&mut self, _: Option<&iced_core::widget::Id>, _: Rectangle) {
                self.0 += 1;
            }
        }
        let builds = Rc::new(Cell::new(0));
        let mut widget = view(true, false, ON, builds.clone());
        let mut tree = Tree::new(&widget);
        let renderer = renderer();
        let node = layout(&mut widget, &mut tree, &renderer);
        let mut count = Count(0);
        widget
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut count);
        assert!(count.0 > 0);
        widget = view(false, false, ON, builds);
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        let mut count = Count(0);
        widget
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut count);
        assert_eq!(count.0, 0);
    }

    #[test]
    fn disabled_motion_is_immediate_without_animation_frames() {
        {
            let policy = Policy::DISABLED;
            let builds = Rc::new(Cell::new(0));
            let mut widget = view(false, true, policy, builds.clone());
            let mut tree = Tree::new(&widget);
            let renderer = renderer();
            layout(&mut widget, &mut tree, &renderer);
            widget = view(true, true, policy, builds.clone());
            tree.diff(&widget);
            let node = layout(&mut widget, &mut tree, &renderer);
            assert_eq!(node.size().height, 80.0);
            let (_, request, invalid) = event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                redraw(Instant::now()),
                mouse::Cursor::Unavailable,
            );
            assert_eq!(request, iced_core::window::RedrawRequest::Wait);
            assert!(!invalid);
            widget = view(false, true, policy, builds);
            tree.diff(&widget);
            assert_eq!(layout(&mut widget, &mut tree, &renderer).size().height, 0.0);
        }
    }

    #[test]
    fn partial_horizontal_reveal_clips_pixels_and_mouse_input() {
        let builds = Rc::new(Cell::new(0));
        let mut widget = view(false, false, ON, builds.clone());
        let mut tree = Tree::new(&widget);
        let mut renderer = renderer();
        layout(&mut widget, &mut tree, &renderer);
        widget = view(true, false, ON, builds);
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        let now = tree.state.downcast_ref::<State>().now;
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            redraw(now + motion::STANDARD / 4),
            mouse::Cursor::Unavailable,
        );
        let reveal_width = 100.0 * tree.state.downcast_ref::<State>().value(ON);
        let cursor = mouse::Cursor::Available(iced_core::Point::new(95.0, 10.0));
        assert!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                mouse(true),
                cursor
            )
            .0
            .is_empty()
        );
        assert!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                mouse(false),
                cursor
            )
            .0
            .is_empty()
        );
        let bounds = Rectangle::with_size(Size::new(200.0, 120.0));
        renderer.reset(bounds);
        widget.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &bounds,
        );
        let pixels = renderer.screenshot(Size::new(200, 120), 1.0, iced_core::Color::BLACK);
        assert!(
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .any(|(i, p)| i % 200 < reveal_width as usize && p[..3] != [0, 0, 0])
        );
        for y in 0..120usize {
            for x in reveal_width.ceil() as usize..200 {
                let i = (y * 200 + x) * 4;
                assert_eq!(&pixels[i..i + 3], &[0, 0, 0]);
            }
        }
    }
}
