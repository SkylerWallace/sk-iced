// SPDX-License-Identifier: Apache-2.0
//! Focus operations and keyboard activation without a runtime dependency.
use super::Appearance;
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Renderer as _, Shell, Size, Theme, Vector, Widget,
    keyboard::{self, Key, key::Named},
    layout, mouse, overlay, renderer,
    widget::{Id, Operation, Tree, operation::Focusable, tree},
};

#[derive(Default)]
struct Focus {
    focused: bool,
    visible: bool,
}
impl Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
        self.visible = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
        self.visible = false;
    }
}
type KeyHandler<'a, M> = Box<dyn Fn(Named) -> Option<M> + 'a>;
/// Adds one focus stop to a non-focusable native control. Space/Enter run its
/// existing pointer action; disabled controls are excluded from focus operations.
/// Do not wrap a text editor or another focusable control.
pub struct KeyboardControl<'a, M> {
    content: Element<'a, M>,
    focus_style: crate::visual::SurfaceStyle,
    focus_inner: Option<iced_core::Color>,
    resolver: Option<fn(&Theme) -> crate::theme::Tokens>,
    enabled: bool,
    id: Option<Id>,
    keys: Option<KeyHandler<'a, M>>,
    activate: bool,
    close_on_key: bool,
    native_arrows: bool,
    selection_keys: bool,
}
fn focus_material(
    tokens: crate::theme::Tokens,
) -> (crate::visual::SurfaceStyle, Option<iced_core::Color>) {
    match tokens.visual.focus_ring {
        Some(style) => (style, None),
        None => (
            crate::visual::SurfaceStyle {
                border_width: Some(2.),
                border_color: Some(crate::visual::Rgba::new(tokens.accent, 1.)),
                shape: Some(crate::visual::Shape::Rounded([4.; 4])),
                ..Default::default()
            },
            Some(crate::theme::color(tokens.accent_text)),
        ),
    }
}
impl<'a, M: 'a> KeyboardControl<'a, M> {
    pub fn new(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        let ((focus_style, focus_inner), resolver) = match appearance.palette {
            super::Palette::Fixed(tokens) => (focus_material(tokens), None),
            super::Palette::Themed(resolve) => ((Default::default(), None), Some(resolve)),
        };
        Self {
            content: content.into(),
            focus_style,
            focus_inner,
            resolver,
            enabled: true,
            id: None,
            keys: None,
            activate: true,
            close_on_key: false,
            native_arrows: false,
            selection_keys: false,
        }
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    /// Additional keys, such as arrows for a controlled selection group.
    pub fn on_key(mut self, handler: impl Fn(Named) -> Option<M> + 'a) -> Self {
        self.keys = Some(Box::new(handler));
        self
    }
    pub fn activation(mut self, value: bool) -> Self {
        self.activate = value;
        self
    }
    pub(crate) fn selection_keys(mut self) -> Self {
        self.selection_keys = true;
        self
    }
    pub(crate) fn native_arrows(mut self) -> Self {
        self.native_arrows = true;
        self
    }
    pub(crate) fn close_on_key(mut self) -> Self {
        self.close_on_key = true;
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<KeyboardControl<'a, M>> for Element<'a, M> {
    fn from(v: KeyboardControl<'a, M>) -> Self {
        v.build()
    }
}
impl<M> Widget<M, Theme, Renderer> for KeyboardControl<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Focus>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Focus::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.content]);
        if !self.enabled {
            t.state.downcast_mut::<Focus>().unfocus();
        }
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
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v);
        let state = t.state.downcast_ref::<Focus>();
        if self.enabled && state.focused && state.visible {
            let (material, inner_color) = self
                .resolver
                .map(|resolve| focus_material(resolve(th)))
                .unwrap_or((self.focus_style, self.focus_inner));
            let mut bg = None;
            let mut border = iced_core::Border::default();
            let mut shadow = iced_core::Shadow::default();
            material.apply(&mut bg, &mut border, &mut shadow);
            r.fill_quad(
                renderer::Quad {
                    bounds: l.bounds(),
                    border,
                    shadow,
                    ..Default::default()
                },
                bg.unwrap_or(iced_core::Color::TRANSPARENT.into()),
            );
            if let Some(color) = inner_color {
                let bounds = l.bounds();
                if bounds.width > 4. && bounds.height > 4. {
                    r.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: bounds.x + 2.,
                                y: bounds.y + 2.,
                                width: bounds.width - 4.,
                                height: bounds.height - 4.,
                            },
                            border: iced_core::Border {
                                color,
                                width: 1.,
                                radius: 2.0.into(),
                            },
                            ..Default::default()
                        },
                        iced_core::Color::TRANSPARENT,
                    );
                }
            }
        }
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
        if s.is_event_captured() && matches!(e, Event::Keyboard(_)) {
            return;
        }
        let state = t.state.downcast_mut::<Focus>();
        if matches!(e, Event::Window(iced_core::window::Event::Unfocused)) {
            state.unfocus();
        }
        if matches!(
            e,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            let focused = self.enabled && c.is_over(l.bounds()) && c.is_over(*v);
            if focused != state.focused || state.visible {
                s.request_redraw();
            }
            state.focused = focused;
            state.visible = false;
        }
        if self.enabled
            && state.focused
            && self.close_on_key
            && matches!(
                e,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Tab),
                    ..
                })
            )
        {
            let mut messages = Vec::new();
            let mut child_shell = Shell::new(&mut messages);
            self.content.as_widget_mut().update(
                &mut t.children[0],
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                l,
                mouse::Cursor::Unavailable,
                r,
                cb,
                &mut child_shell,
                v,
            );
            s.request_redraw_at(child_shell.redraw_request());
            if child_shell.is_layout_invalid() {
                s.invalidate_layout();
            }
            for message in messages {
                s.publish(message);
            }
        }
        if self.enabled
            && state.focused
            && self.native_arrows
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modifiers,
                ..
            }) = e
            && matches!(
                key,
                Named::ArrowUp | Named::ArrowDown | Named::ArrowLeft | Named::ArrowRight
            )
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
        {
            state.visible = true;
            let mut event = e.clone();
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: native,
                modified_key,
                ..
            }) = &mut event
            {
                *native = Key::Named(if matches!(key, Named::ArrowUp | Named::ArrowRight) {
                    Named::ArrowUp
                } else {
                    Named::ArrowDown
                });
                *modified_key = native.clone();
            }
            self.content.as_widget_mut().update(
                &mut t.children[0],
                &event,
                l,
                mouse::Cursor::Available(l.bounds().center()),
                r,
                cb,
                s,
                v,
            );
            s.capture_event();
            s.request_redraw();
            return;
        }
        if self.enabled
            && state.focused
            && let Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modifiers,
                repeat,
                ..
            }) = e
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
            && !modifiers.shift()
        {
            let action = self.keys.as_ref().and_then(|f| f(*key));
            let activation_key = matches!(key, Named::Space | Named::Enter);
            let activate = self.activate && activation_key;
            let navigate = self.selection_keys
                && matches!(
                    key,
                    Named::ArrowUp
                        | Named::ArrowDown
                        | Named::ArrowLeft
                        | Named::ArrowRight
                        | Named::Home
                        | Named::End
                );
            if action.is_some() || activate || navigate {
                state.visible = true;
                // Repeat is useful for arrows, but must not repeat destructive actions.
                if !*repeat || !activation_key {
                    let cursor = if action.is_some() {
                        mouse::Cursor::Unavailable
                    } else {
                        mouse::Cursor::Available(l.bounds().center())
                    };
                    if activate || self.close_on_key {
                        for event in [
                            mouse::Event::ButtonPressed(mouse::Button::Left),
                            mouse::Event::ButtonReleased(mouse::Button::Left),
                        ] {
                            let mut messages = Vec::new();
                            let mut child_shell = Shell::new(&mut messages);
                            self.content.as_widget_mut().update(
                                &mut t.children[0],
                                &Event::Mouse(event),
                                l,
                                cursor,
                                r,
                                cb,
                                &mut child_shell,
                                v,
                            );
                            s.merge(child_shell, std::convert::identity);
                        }
                    }
                    if let Some(message) = action {
                        s.publish(message);
                    }
                }
                s.capture_event();
                s.request_redraw();
                return;
            }
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if self.enabled {
            o.focusable(
                self.id.as_ref(),
                l.bounds(),
                t.state.downcast_mut::<Focus>(),
            );
        }
        // This wrapper owns the control's single focus stop.
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

#[derive(Default)]
struct ScopeState {
    initialized: bool,
    active: bool,
}
/// Opt-in Tab/Shift+Tab traversal for a view or popup. Mount consistently to keep
/// child state. Existing captured keys and modified shortcuts are left alone.
/// Focus order follows widget order, wraps within this scope, and skips disabled
/// shared controls. Use a separate scope for modal popup content.
#[doc = include_str!("../../docs/keyboard.md")]
pub struct FocusScope<'a, M> {
    content: Element<'a, M>,
    active: bool,
    initial: Option<Id>,
}
impl<'a, M: 'a> FocusScope<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            active: true,
            initial: None,
        }
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    /// Focus this ID when the scope first becomes active. Restore the trigger ID
    /// using Iced's focus operation when dismissing an app-owned popup.
    pub fn autofocus(mut self, id: impl Into<Id>) -> Self {
        self.initial = Some(id.into());
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<FocusScope<'a, M>> for Element<'a, M> {
    fn from(v: FocusScope<'a, M>) -> Self {
        v.build()
    }
}
#[derive(Default)]
struct Count {
    total: usize,
    focused: Option<usize>,
}
impl Operation for Count {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self)
    }
    fn focusable(&mut self, _: Option<&Id>, _: Rectangle, s: &mut dyn Focusable) {
        if s.is_focused() {
            self.focused = Some(self.total);
        }
        self.total += 1;
    }
}
struct Select {
    index: usize,
    current: usize,
}
impl Operation for Select {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self)
    }
    fn focusable(&mut self, _: Option<&Id>, _: Rectangle, s: &mut dyn Focusable) {
        if self.current == self.index {
            s.focus()
        } else {
            s.unfocus()
        }
        self.current += 1;
    }
}

#[derive(Clone, Copy)]
struct ScrollGeometry {
    index: usize,
    translation: Vector,
}
#[derive(Default)]
struct LocateFocus {
    next: usize,
    pending: Option<ScrollGeometry>,
    parent: Option<ScrollGeometry>,
    request: Option<(ScrollGeometry, Rectangle)>,
}
impl Operation for LocateFocus {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        let previous = self.parent;
        if let Some(pending) = self.pending.take() {
            self.parent = Some(pending);
        }
        f(self);
        self.parent = previous;
    }
    fn scrollable(
        &mut self,
        _: Option<&Id>,
        _bounds: Rectangle,
        _content: Rectangle,
        translation: Vector,
        _: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        self.pending = Some(ScrollGeometry {
            index: self.next,
            translation,
        });
        self.next += 1;
    }
    fn focusable(&mut self, _: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.request = self.parent.map(|parent| (parent, bounds));
        }
    }
}
struct RevealFocus {
    next: usize,
    target: ScrollGeometry,
    focused: Rectangle,
}
impl Operation for RevealFocus {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self)
    }
    fn scrollable(
        &mut self,
        _: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        _: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        if self.next == self.target.index {
            let visible = self.focused - self.target.translation;
            let delta = |start: f32, end: f32, minimum: f32, maximum: f32| {
                if start < minimum {
                    start - minimum
                } else if end > maximum {
                    end - maximum
                } else {
                    0.
                }
            };
            let x = delta(
                visible.x,
                visible.x + visible.width,
                bounds.x,
                bounds.x + bounds.width,
            );
            let y = delta(
                visible.y,
                visible.y + visible.height,
                bounds.y,
                bounds.y + bounds.height,
            );
            state.scroll_by(
                iced_core::widget::operation::scrollable::AbsoluteOffset { x, y },
                bounds,
                content,
            );
        }
        self.next += 1;
    }
}

impl<M> Widget<M, Theme, Renderer> for FocusScope<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScopeState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(ScopeState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.content]);
        let state = t.state.downcast_mut::<ScopeState>();
        if self.active != state.active {
            state.initialized = false;
        }
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l);
        let state = t.state.downcast_mut::<ScopeState>();
        if !state.initialized {
            if self.active {
                if let Some(id) = &self.initial {
                    self.content.as_widget_mut().operate(
                        &mut t.children[0],
                        Layout::new(&node),
                        r,
                        &mut iced_core::widget::operation::focusable::focus::<()>(id.clone()),
                    );
                }
            } else {
                self.content.as_widget_mut().operate(
                    &mut t.children[0],
                    Layout::new(&node),
                    r,
                    &mut iced_core::widget::operation::focusable::unfocus::<()>(),
                );
            }
            state.initialized = true;
            state.active = self.active;
        }
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
        self.content
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
        if !self.active && matches!(e, Event::Keyboard(_)) {
            return;
        }
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
        if !self.active || s.is_event_captured() {
            return;
        }
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Tab),
            modifiers,
            ..
        }) = e
            && !modifiers.control()
            && !modifiers.alt()
            && !modifiers.logo()
        {
            let mut count = Count::default();
            self.content
                .as_widget_mut()
                .operate(&mut t.children[0], l, r, &mut count);
            if count.total > 0 {
                let index = match (count.focused, modifiers.shift()) {
                    (Some(i), true) => (i + count.total - 1) % count.total,
                    (Some(i), false) => (i + 1) % count.total,
                    (None, true) => count.total - 1,
                    (None, false) => 0,
                };
                self.content.as_widget_mut().operate(
                    &mut t.children[0],
                    l,
                    r,
                    &mut Select { index, current: 0 },
                );
                let mut locate = LocateFocus::default();
                self.content
                    .as_widget_mut()
                    .operate(&mut t.children[0], l, r, &mut locate);
                if let Some((target, focused)) = locate.request {
                    self.content.as_widget_mut().operate(
                        &mut t.children[0],
                        l,
                        r,
                        &mut RevealFocus {
                            next: 0,
                            target,
                            focused,
                        },
                    );
                    s.invalidate_layout();
                }
                s.capture_event();
                s.request_redraw();
            }
        }
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        if self.active {
            self.content
                .as_widget_mut()
                .operate(&mut t.children[0], l, r, o)
        }
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
