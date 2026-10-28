// SPDX-License-Identifier: Apache-2.0
//! Anchored or pointer-positioned overlays with immediate dismissal and paint-only
//! closing transitions. Lifecycle redraws reach the base; only input is isolated.
use crate::{
    animation::{Element, Renderer, reveal},
    motion::Policy,
};
use iced_core::{
    Clipboard, Event, Layout, Length, Point, Rectangle, Shell, Size, Theme, Vector, Widget, layout,
    mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
type KeyHandler<'a, M> = Box<dyn Fn(&iced_core::keyboard::Key) -> Option<M> + 'a>;
struct PopupConfig<'a, M> {
    point: Option<Point>,
    centered: bool,
    max_width: f32,
    max_height: f32,
    fit_above: bool,
    fit_viewport: bool,
    align_end: bool,
    gap: f32,
    pointer_offset: f32,
    floating: bool,
    clip_input: bool,
    capture_outside: bool,
    dismiss_any_button: bool,
    scrim: Option<iced_core::Color>,
    dismiss: Option<M>,
    on_key: Option<KeyHandler<'a, M>>,
}
pub struct Popup<'a, M> {
    layer: Layer<'a, M>,
}
impl<'a, M: Clone + 'a> Popup<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    /// Closed by default. Set open and supply the app's motion policy and dismiss
    /// action. Defaults to end-aligned placement beside the anchor widget.
    pub fn new(base: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            layer: Layer {
                base: base.into(),
                popup: content.into(),
                open: false,
                policy: Policy::DISABLED,
                config: PopupConfig {
                    point: None,
                    centered: false,
                    max_width: 432.0,
                    max_height: f32::MAX,
                    fit_above: false,
                    fit_viewport: false,
                    align_end: true,
                    gap: 6.0,
                    pointer_offset: 0.0,
                    floating: false,
                    clip_input: true,
                    capture_outside: false,
                    dismiss_any_button: false,
                    scrim: None,
                    dismiss: None,
                    on_key: None,
                },
            },
        }
    }
    pub fn scrim(mut self, value: iced_core::Color) -> Self {
        self.layer.config.scrim = Some(value);
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.layer.open = value;
        self
    }
    pub fn policy(mut self, value: Policy) -> Self {
        self.layer.policy = value;
        self
    }
    pub fn at(mut self, value: Point) -> Self {
        self.layer.config.point = Some(value);
        self
    }
    pub fn centered(mut self, value: bool) -> Self {
        self.layer.config.centered = value;
        self
    }
    pub fn max_width(mut self, value: f32) -> Self {
        self.layer.config.max_width = value;
        self
    }
    pub fn max_height(mut self, value: f32) -> Self {
        self.layer.config.max_height = value.max(1.0);
        self
    }
    pub fn fit_viewport(mut self, value: bool) -> Self {
        self.layer.config.fit_viewport = value;
        self
    }
    pub fn fit_above(mut self, value: bool) -> Self {
        self.layer.config.fit_above = value;
        self
    }
    pub fn align_end(mut self, value: bool) -> Self {
        self.layer.config.align_end = value;
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.layer.config.gap = value;
        self
    }
    pub fn pointer_offset(mut self, value: f32) -> Self {
        self.layer.config.pointer_offset = value;
        self
    }
    pub fn floating(mut self, value: bool) -> Self {
        self.layer.config.floating = value;
        self
    }
    pub fn clip_input(mut self, value: bool) -> Self {
        self.layer.config.clip_input = value;
        self
    }
    pub fn capture_outside(mut self, value: bool) -> Self {
        self.layer.config.capture_outside = value;
        self
    }
    pub fn dismiss_any_button(mut self, value: bool) -> Self {
        self.layer.config.dismiss_any_button = value;
        self
    }
    pub fn on_dismiss(mut self, value: M) -> Self {
        self.layer.config.dismiss = Some(value);
        self
    }
    pub fn on_key(mut self, value: impl Fn(&iced_core::keyboard::Key) -> Option<M> + 'a) -> Self {
        self.layer.config.on_key = Some(Box::new(value));
        self
    }
}
impl<'a, M: Clone + 'a> From<Popup<'a, M>> for Element<'a, M> {
    fn from(spec: Popup<'a, M>) -> Self {
        Element::new(spec.layer)
    }
}
struct Layer<'a, M> {
    base: Element<'a, M>,
    popup: Element<'a, M>,
    open: bool,
    policy: Policy,
    config: PopupConfig<'a, M>,
}
impl<M> Widget<M, Theme, Renderer> for Layer<'_, M>
where
    M: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<reveal::State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(reveal::State::new(self.open, std::time::Instant::now()))
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.popup)]
    }
    fn diff(&self, t: &mut Tree) {
        t.state.downcast_mut::<reveal::State>().sync(
            self.open,
            self.policy,
            std::time::Instant::now(),
        );
        t.diff_children(&[&self.base, &self.popup]);
    }
    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        self.base.as_widget_mut().layout(&mut t.children[0], r, l)
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
        self.base
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v)
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
        let now = match e {
            Event::Window(iced_core::window::Event::RedrawRequested(now)) => *now,
            _ => std::time::Instant::now(),
        };
        t.state
            .downcast_mut::<reveal::State>()
            .tick(now, self.policy, s);
        self.base
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v)
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.base
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o)
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.base
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
        if t.state.downcast_ref::<reveal::State>().present(self.policy) {
            Some(overlay::Element::new(Box::new(PopupOverlay {
                content: &mut self.popup,
                tree: &mut t.children[1],
                anchor: l.bounds() + tr,
                state: t.state.downcast_mut::<reveal::State>(),
                open: self.open,
                policy: self.policy,
                config: &self.config,
                viewport: Size::ZERO,
            })))
        } else {
            self.base
                .as_widget_mut()
                .overlay(&mut t.children[0], l, r, v, tr)
        }
    }
}
struct PopupOverlay<'a, 'b, M> {
    content: &'a mut Element<'b, M>,
    tree: &'a mut Tree,
    anchor: Rectangle,
    state: &'a mut reveal::State,
    open: bool,
    policy: Policy,
    config: &'a PopupConfig<'b, M>,
    viewport: Size,
}
impl<M> PopupOverlay<'_, '_, M> {
    fn clip(&self, layout: Layout<'_>) -> Option<Rectangle> {
        let edge = if layout.bounds().y < self.anchor.y {
            reveal::Edge::Bottom
        } else {
            reveal::Edge::Top
        };
        self.state.clip(layout.bounds(), edge, self.policy)
    }
}
impl<M> overlay::Overlay<M, Theme, Renderer> for PopupOverlay<'_, '_, M>
where
    M: Clone,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.viewport = bounds;
        let config = self.config;
        let below = bounds.height
            - self.anchor.y
            - self.anchor.height
            - 6.0
            - (bounds.width - self.anchor.x - self.anchor.width).max(8.0);
        let available = if config.point.is_some() || config.centered || config.fit_viewport {
            (bounds.height - 16.0).max(1.0)
        } else if config.fit_above {
            (bounds.height - self.anchor.y - self.anchor.height - 14.0).max(self.anchor.y - 14.0)
        } else {
            below
        };
        let max = Size::new(
            (bounds.width - 16.0).clamp(1.0, config.max_width.max(1.0)),
            available
                .clamp(1.0, (bounds.height - 16.0).max(1.0))
                .min(config.max_height),
        );
        let node = self.content.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, max),
        );
        let size = node.size();
        let (x, y) = if config.centered {
            (
                (bounds.width - size.width) / 2.0,
                (bounds.height - size.height) / 2.0,
            )
        } else if let Some(point) = config.point {
            (
                point.x + config.pointer_offset,
                point.y + config.pointer_offset,
            )
        } else {
            (
                if config.align_end {
                    self.anchor.x + self.anchor.width - size.width
                } else {
                    self.anchor.x
                },
                if self.anchor.y + self.anchor.height + size.height <= bounds.height - 8.0 {
                    self.anchor.y + self.anchor.height + config.gap
                } else {
                    self.anchor.y - size.height - 4.0
                },
            )
        };
        node.move_to(Point::new(
            x.clamp(8.0, (bounds.width - size.width - 8.0).max(8.0)),
            y.clamp(8.0, (bounds.height - size.height - 8.0).max(8.0)),
        ))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        use iced_core::Renderer as _;
        if let Some(scrim) = self.config.scrim {
            renderer.fill_quad(
                iced_core::renderer::Quad {
                    bounds: Rectangle::with_size(self.viewport),
                    ..Default::default()
                },
                scrim,
            );
        }
        let Some(clip) = self.clip(layout) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout,
                if self.open
                    && !self.config.floating
                    && (!self.config.clip_input || cursor.is_over(clip))
                {
                    cursor
                } else {
                    mouse::Cursor::Unavailable
                },
                &clip,
            )
        });
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
    ) {
        let now = match event {
            Event::Window(iced_core::window::Event::RedrawRequested(now)) => *now,
            _ => std::time::Instant::now(),
        };
        self.state.tick(now, self.policy, shell);
        if self.state.take_cancel_input() {
            reveal::cancel_input(self.content, self.tree, layout, renderer, clipboard);
        }
        if !self.open {
            if let Some(clip) = self.clip(layout) {
                reveal::update_visual(
                    self.content,
                    self.tree,
                    event,
                    layout,
                    renderer,
                    clipboard,
                    &clip,
                );
            }
            return;
        }
        if self.config.floating {
            return;
        }
        if let Event::Keyboard(iced_core::keyboard::Event::KeyPressed { key, .. }) = event {
            let action = self
                .config
                .on_key
                .as_ref()
                .and_then(|handler| handler(key))
                .or_else(|| {
                    if *key
                        == iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape)
                    {
                        self.config.dismiss.clone()
                    } else {
                        None
                    }
                });
            if let Some(action) = action {
                shell.publish(action);
                shell.capture_event();
                return;
            }
        }
        let outside_press = matches!(event, Event::Mouse(mouse::Event::ButtonPressed(button))
            if self.config.dismiss_any_button || *button == mouse::Button::Left);
        if outside_press
            && !cursor.is_over(layout.bounds())
            && let Some(action) = &self.config.dismiss
        {
            shell.publish(action.clone());
            shell.capture_event();
            return;
        }

        let visible_cursor = if !self.config.clip_input
            || self.clip(layout).is_some_and(|clip| cursor.is_over(clip))
        {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            visible_cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
        // Capturing redraws prevents the underlying tree from initializing the
        // status of rebuilt buttons, which then paint with Disabled as a fallback.
        // Only input is intercepted; redraw/window lifecycle events must propagate.
        if (self.config.capture_outside || cursor.is_over(layout.bounds()))
            && matches!(
                event,
                Event::Mouse(_) | Event::Touch(_) | Event::Keyboard(_)
            )
        {
            shell.capture_event();
        }
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.config.floating
            || !self.open
            || (self.config.clip_input
                && !self.clip(layout).is_some_and(|clip| cursor.is_over(clip)))
        {
            return mouse::Interaction::default();
        }
        let interaction = self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        );
        if self.config.capture_outside && cursor.is_over(layout.bounds()) {
            interaction.max(mouse::Interaction::Idle)
        } else {
            interaction
        }
    }
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, op: &mut dyn Operation) {
        if !self.open {
            return;
        }
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, op);
    }
    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, M, Theme, Renderer>> {
        if !self.open {
            return None;
        }
        self.content.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}

#[cfg(test)]
mod reveal_lifecycle_tests {
    use super::*;
    const ON: Policy = Policy {
        system_enabled: true,
    };

    #[test]
    fn closing_popup_is_paint_only_and_stops_requesting_frames() {
        let mut content: Element<'_, u8> = iced_widget::button("Test")
            .width(120)
            .height(80)
            .on_press(7u8)
            .into();
        let mut tree = Tree::new(&content);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let start = std::time::Instant::now();
        let mut state = reveal::State::new(true, start);
        state.sync(false, ON, start);
        let config = super::Popup::new(iced_widget::Space::new(), iced_widget::Space::new())
            .on_dismiss(9u8)
            .layer
            .config;
        let mut popup = PopupOverlay {
            config: &config,
            viewport: Size::ZERO,
            content: &mut content,
            tree: &mut tree,
            anchor: Rectangle::new(Point::new(160.0, 10.0), Size::new(28.0, 28.0)),
            state: &mut state,
            open: false,
            policy: ON,
        };
        let node = overlay::Overlay::layout(&mut popup, &renderer, Size::new(220.0, 180.0));
        let cursor = mouse::Cursor::Available(node.bounds().center());
        for input in [
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            let mut messages = vec![];
            let mut shell = Shell::new(&mut messages);
            overlay::Overlay::update(
                &mut popup,
                &input,
                Layout::new(&node),
                cursor,
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
            );
            assert!(!shell.is_event_captured());
            assert!(messages.is_empty());
        }
        assert_eq!(
            overlay::Overlay::mouse_interaction(&popup, Layout::new(&node), cursor, &renderer),
            mouse::Interaction::None
        );
        assert!(overlay::Overlay::overlay(&mut popup, Layout::new(&node), &renderer).is_none());
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &Event::Window(iced_core::window::Event::RedrawRequested(
                start + crate::animation::primitives::QUICK,
            )),
            Layout::new(&node),
            cursor,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert_eq!(
            shell.redraw_request(),
            iced_core::window::RedrawRequest::Wait
        );
        assert!(popup.clip(Layout::new(&node)).is_none());
    }

    #[test]
    fn opening_popup_dismisses_before_reveal_finishes_and_reveals_from_its_anchor() {
        let mut content: Element<'_, u8> = iced_widget::container(iced_widget::Space::new())
            .width(120)
            .height(100)
            .into();
        let mut tree = Tree::new(&content);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let start = std::time::Instant::now();
        let mut state = reveal::State::new(false, start);
        state.sync(true, ON, start);
        let config = super::Popup::new(iced_widget::Space::new(), iced_widget::Space::new())
            .on_dismiss(9u8)
            .layer
            .config;
        let mut popup = PopupOverlay {
            config: &config,
            viewport: Size::ZERO,
            content: &mut content,
            tree: &mut tree,
            anchor: Rectangle::new(Point::new(160.0, 172.0), Size::new(28.0, 28.0)),
            state: &mut state,
            open: true,
            policy: ON,
        };
        let node = overlay::Overlay::layout(&mut popup, &renderer, Size::new(220.0, 180.0));
        let mut messages = vec![];
        overlay::Overlay::update(
            &mut popup,
            &Event::Window(iced_core::window::Event::RedrawRequested(
                start + crate::animation::primitives::STANDARD / 4,
            )),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
        );
        let clip = popup.clip(Layout::new(&node)).unwrap();
        assert!(clip.height > 0.0 && clip.height < node.size().height);
        assert_eq!(clip.y + clip.height, node.bounds().y + node.bounds().height);
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(1.0, 1.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(shell.is_event_captured());
        assert!(matches!(messages.as_slice(), [9u8]));
    }

    #[test]
    fn end_aligned_popup_anchors_dismisses_and_forwards_redraws() {
        let mut content: Element<'_, u8> = iced_widget::container(iced_widget::Space::new())
            .width(376)
            .height(200)
            .into();
        let mut tree = Tree::new(&content);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.333.into());
        let mut state = reveal::State::new(true, std::time::Instant::now());
        let config = super::Popup::new(iced_widget::Space::new(), iced_widget::Space::new())
            .on_dismiss(9u8)
            .layer
            .config;
        let mut popup = PopupOverlay {
            config: &config,
            viewport: Size::ZERO,
            content: &mut content,
            tree: &mut tree,
            anchor: Rectangle {
                x: 1080.0,
                y: 0.0,
                width: 110.0,
                height: 32.0,
            },
            state: &mut state,
            open: true,
            policy: Policy::DISABLED,
        };
        let node = overlay::Overlay::layout(&mut popup, &renderer, Size::new(1200.0, 900.0));
        let bounds = node.bounds();
        assert_eq!(bounds.x + bounds.width, 1190.0);
        assert_eq!(bounds.y, 38.0);
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &Event::Window(iced_core::window::Event::RedrawRequested(
                std::time::Instant::now(),
            )),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(!shell.is_event_captured());
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(shell.is_event_captured());
        assert!(matches!(messages.as_slice(), [9u8]));
    }
}

#[cfg(test)]
mod input_routing_tests {
    use super::*;
    use iced_widget::space;
    #[test]
    fn popup_keeps_base_redraws_but_captures_clicks_and_dismissal() {
        use iced_widget::{button, column, container};
        use std::{cell::Cell, rc::Rc};
        let enabled = Rc::new(Cell::new(None));
        let disabled = Rc::new(Cell::new(None));
        let observe = |state: Rc<Cell<Option<button::Status>>>| {
            move |_: &Theme, status| {
                state.set(Some(status));
                button::Style::default()
            }
        };
        let mut base: Element<'_, u8> = column![
            button(space())
                .width(300)
                .height(120)
                .on_press(7u8)
                .style(observe(enabled.clone())),
            button(space())
                .width(300)
                .height(120)
                .style(observe(disabled.clone())),
        ]
        .into();
        let mut content: Element<'_, u8> = container(space()).width(180).height(140).into();
        let mut base_tree = Tree::new(&base);
        let mut popup_tree = Tree::new(&content);
        let mut renderer = Renderer::new(iced_core::Font::DEFAULT, 14.0.into());
        let bounds = Rectangle::with_size(Size::new(300.0, 240.0));
        let base_node = base.as_widget_mut().layout(
            &mut base_tree,
            &renderer,
            &layout::Limits::new(bounds.size(), bounds.size()),
        );
        let config = super::Popup::new(iced_widget::Space::new(), iced_widget::Space::new())
            .max_width(376.0)
            .fit_above(true)
            .clip_input(false)
            .on_dismiss(9u8)
            .layer
            .config;
        let mut state = reveal::State::new(true, std::time::Instant::now());
        let mut popup = PopupOverlay {
            content: &mut content,
            tree: &mut popup_tree,
            anchor: Rectangle {
                x: 268.0,
                y: 8.0,
                width: 32.0,
                height: 32.0,
            },
            open: true,
            state: &mut state,
            policy: Policy::DISABLED,
            config: &config,
            viewport: Size::ZERO,
        };
        let popup_node = overlay::Overlay::layout(&mut popup, &renderer, bounds.size());
        let popup_layout = Layout::new(&popup_node);
        let cursor = mouse::Cursor::Available(Point::new(180.0, 80.0));
        assert!(cursor.is_over(popup_layout.bounds()));
        let redraw = Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        ));
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &redraw,
            popup_layout,
            cursor,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(
            !shell.is_event_captured(),
            "The base tree must receive redraws"
        );
        base.as_widget_mut().update(
            &mut base_tree,
            &redraw,
            Layout::new(&base_node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &bounds,
        );
        base.as_widget().draw(
            &base_tree,
            &mut renderer,
            &Theme::Dark,
            &renderer::Style::default(),
            Layout::new(&base_node),
            mouse::Cursor::Unavailable,
            &bounds,
        );
        assert_eq!(enabled.get(), Some(button::Status::Active));
        assert_eq!(disabled.get(), Some(button::Status::Disabled));
        for event in [
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            let mut shell = Shell::new(&mut messages);
            overlay::Overlay::update(
                &mut popup,
                &event,
                popup_layout,
                cursor,
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
            );
            assert!(
                shell.is_event_captured(),
                "Clicks inside the popup must not reach Run"
            );
        }
        assert!(messages.is_empty());
        let mut shell = Shell::new(&mut messages);
        overlay::Overlay::update(
            &mut popup,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            popup_layout,
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(
            shell.is_event_captured(),
            "Dismissal must not click through"
        );
        assert!(matches!(messages.as_slice(), [9u8]));
    }
}
