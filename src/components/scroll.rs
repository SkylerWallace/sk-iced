// SPDX-License-Identifier: Apache-2.0
//! Native scroll areas and rail profiles. Scrolling stays immediate; no tween,
//! timer, motion subscription or per-frame application messages are introduced.
//! Native widget IDs, offsets, anchors and operations remain intact on rebuild.
//!
//! Use ScrollArea::new(appearance, content).id(stable_id) for a themed native area,
//! and vertical(Scrollbar::embedded()) or horizontal(Scrollbar::hidden()) for its
//! rail profile. build() exposes the native widget without adding a tree layer.
//! LocalScroll resolves ScrollAction requests locally and synchronizes matching
//! ScrollRail IDs. Its optional hooks keep app routing/instrumentation outside
//! this library. HorizontalScrollRail retains the inset file-table profile;
//! RailVisual shares paint geometry with specialized virtualized viewports.
//! No component here animates scroll offsets.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    theme::{Tokens, color},
};
use iced_core::Renderer as _;
use iced_core::{
    Clipboard, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Theme, Vector, Widget,
    layout, mouse, overlay, renderer,
    widget::{Id, Operation, Tree, tree},
};
use iced_widget::scrollable;

/// Native scrollbar geometry. Defaults exactly match Iced; embedded() is the
/// compact profile for embedded areas. Build before passing to a native Direction.
pub struct Scrollbar {
    inner: scrollable::Scrollbar,
}
impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}
impl Scrollbar {
    pub fn new() -> Self {
        Self {
            inner: scrollable::Scrollbar::new(),
        }
    }
    pub fn hidden() -> Self {
        Self {
            inner: scrollable::Scrollbar::hidden(),
        }
    }
    pub fn embedded() -> Self {
        Self::new().width(4).scroller_width(6).margin(3).spacing(12)
    }
    pub fn width(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn thumb_width(self, value: impl Into<Pixels>) -> Self {
        self.scroller_width(value)
    }
    pub fn scroller_width(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.scroller_width(value);
        self
    }
    pub fn margin(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.margin(value);
        self
    }
    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(value);
        self
    }
    pub fn anchor(mut self, value: scrollable::Anchor) -> Self {
        self.inner = self.inner.anchor(value);
        self
    }
    pub fn build(self) -> scrollable::Scrollbar {
        self.inner
    }
}
impl From<Scrollbar> for scrollable::Scrollbar {
    fn from(spec: Scrollbar) -> Self {
        spec.build()
    }
}

pub struct ScrollArea<'a, M> {
    inner: iced_widget::Scrollable<'a, M>,
}
impl<'a, M: 'a> ScrollArea<'a, M> {
    pub fn new(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        Self::custom(iced_widget::scrollable(content))
            .style(move |theme, status| appearance.tokens(theme).scroll()(theme, status))
    }
    /// Retain native styling and defaults for existing app profiles.
    pub fn custom(inner: iced_widget::Scrollable<'a, M>) -> Self {
        Self { inner }
    }
    pub fn id(mut self, value: impl Into<Id>) -> Self {
        self.inner = self.inner.id(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.height(value);
        self
    }
    pub fn direction(mut self, value: impl Into<scrollable::Direction>) -> Self {
        self.inner = self.inner.direction(value);
        self
    }
    pub fn horizontal(mut self, rail: Scrollbar) -> Self {
        self.inner = self
            .inner
            .direction(scrollable::Direction::Horizontal(rail.build()));
        self
    }
    pub fn vertical(mut self, rail: Scrollbar) -> Self {
        self.inner = self
            .inner
            .direction(scrollable::Direction::Vertical(rail.build()));
        self
    }
    pub fn on_scroll(mut self, value: impl Fn(scrollable::Viewport) -> M + 'a) -> Self {
        self.inner = self.inner.on_scroll(value);
        self
    }
    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(value);
        self
    }
    pub fn auto_scroll(mut self, value: bool) -> Self {
        self.inner = self.inner.auto_scroll(value);
        self
    }
    pub fn anchor_x(mut self, value: scrollable::Anchor) -> Self {
        self.inner = self.inner.anchor_x(value);
        self
    }
    pub fn anchor_y(mut self, value: scrollable::Anchor) -> Self {
        self.inner = self.inner.anchor_y(value);
        self
    }
    pub fn anchor_top(self) -> Self {
        self.anchor_y(scrollable::Anchor::Start)
    }
    pub fn anchor_bottom(self) -> Self {
        self.anchor_y(scrollable::Anchor::End)
    }
    pub fn anchor_left(self) -> Self {
        self.anchor_x(scrollable::Anchor::Start)
    }
    pub fn anchor_right(self) -> Self {
        self.anchor_x(scrollable::Anchor::End)
    }
    pub fn style(
        mut self,
        value: impl Fn(&Theme, scrollable::Status) -> scrollable::Style + 'a,
    ) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn build(self) -> iced_widget::Scrollable<'a, M> {
        self.inner
    }
}
impl<'a, M: 'a> From<ScrollArea<'a, M>> for Element<'a, M> {
    fn from(spec: ScrollArea<'a, M>) -> Self {
        spec.build().into()
    }
}
pub fn scroll_area<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> ScrollArea<'a, M> {
    ScrollArea::custom(iced_widget::scrollable(content))
}

/// Shared CPU rail painter; geometry/cursors and scrolling policy are separate.
/// Different native profiles can preserve square tracks and rounded thumbs.
pub struct RailVisual {
    pub track: Rectangle,
    pub thumb: Option<Rectangle>,
    pub track_color: iced_core::Color,
    pub thumb_color: iced_core::Color,
    pub track_radius: f32,
    pub thumb_radius: f32,
}
impl RailVisual {
    pub fn draw(&self, renderer: &mut Renderer) {
        self.draw_styled(renderer, Default::default());
    }
    pub fn draw_styled(&self, renderer: &mut Renderer, style: crate::visual::SurfaceStyle) {
        use iced_core::Renderer as _;
        renderer.fill_quad(
            renderer::Quad {
                bounds: self.track,
                border: iced_core::Border {
                    radius: self.track_radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            self.track_color,
        );
        if let Some(thumb) = self.thumb {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: thumb,
                    border: iced_core::Border {
                        radius: style
                            .shape
                            .map(|s| s.radius())
                            .unwrap_or(self.thumb_radius.into()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                style
                    .fill
                    .map(|f| f.background(style.opacity()))
                    .unwrap_or(self.thumb_color.into()),
            );
        }
    }
}
/// Messages consumed inside a local scroll area instead of rebuilding the app.
pub enum ScrollAction {
    Forward,
    Discard,
    Vertical(f32),
}
/// Optional event preprocessing, for app-owned profiling/replay instrumentation.
/// The default is empty and allocates no event buffer.
pub struct ScrollReplay<M> {
    pub events: Vec<(Event, mouse::Cursor)>,
    pub messages: Vec<M>,
    pub redraw: bool,
    pub skip_input: bool,
}
impl<M> Default for ScrollReplay<M> {
    fn default() -> Self {
        Self {
            events: vec![],
            messages: vec![],
            redraw: false,
            skip_input: false,
        }
    }
}
pub type ScrollEventHook<M> =
    fn(&Event, Layout<'_>, mouse::Cursor, &mut u32, Option<&Id>) -> ScrollReplay<M>;
pub type ScrollForwardHook<M> = fn(&M, mouse::Cursor, &mut Shell<'_, M>);

/// Wrapper that resolves rail messages and notifications locally. Widget IDs
/// must stay stable across rebuilds. Normal wheel/drag updates do not relayout,
/// invalidate widgets or emit application scroll messages.
pub struct LocalScroll<'a, M> {
    base: Element<'a, M>,
    popup: Element<'a, M>,
    observe: Option<fn(Rectangle, Rectangle) -> M>,
    scroll_id: Option<Id>,
    filter: Option<fn(&M) -> ScrollAction>,
    event_hook: Option<ScrollEventHook<M>>,
    before_forward: Option<ScrollForwardHook<M>>,
}
impl<'a, M: 'a> LocalScroll<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            base: content.into(),
            popup: iced_widget::Space::new().into(),
            observe: None,
            scroll_id: None,
            filter: None,
            event_hook: None,
            before_forward: None,
        }
    }
    pub fn track(mut self, id: impl Into<Id>) -> Self {
        self.scroll_id = Some(id.into());
        self
    }
    pub fn observe(mut self, callback: fn(Rectangle, Rectangle) -> M) -> Self {
        self.observe = Some(callback);
        self
    }
    pub fn filter(mut self, callback: fn(&M) -> ScrollAction) -> Self {
        self.filter = Some(callback);
        self
    }
    pub fn event_hook(mut self, callback: ScrollEventHook<M>) -> Self {
        self.event_hook = Some(callback);
        self
    }
    pub fn before_forward(mut self, callback: ScrollForwardHook<M>) -> Self {
        self.before_forward = Some(callback);
        self
    }
}
impl<'a, M: 'a> From<LocalScroll<'a, M>> for Element<'a, M> {
    fn from(spec: LocalScroll<'a, M>) -> Self {
        Element::new(spec)
    }
}
struct SyncRail {
    id: iced_core::widget::Id,
    metrics: Option<(f32, f32)>,
}
impl Operation for SyncRail {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self);
    }
    fn scrollable(
        &mut self,
        id: Option<&iced_core::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        _: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        if id == Some(&self.id) {
            let overflow = (content.height - bounds.height).max(0.0);
            self.metrics = Some((
                if overflow > 0.0 {
                    (translation.y / overflow).clamp(0.0, 1.0)
                } else {
                    0.0
                },
                (bounds.height / content.height.max(1.0)).clamp(0.0, 1.0),
            ));
        }
    }
    fn custom(
        &mut self,
        id: Option<&iced_core::widget::Id>,
        _: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if id == Some(&self.id)
            && let Some(rail) = state.downcast_mut::<ScrollRailState>()
        {
            rail.metrics = self.metrics;
        }
    }
}
#[derive(Default)]
struct LocalScrollState {
    replay_step: u32,
    geometry: Option<(Rectangle, Rectangle)>,
}
impl<M> Widget<M, Theme, Renderer> for LocalScroll<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LocalScrollState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(LocalScrollState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::new(&self.popup)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.base, &self.popup]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
        if self.observe.is_some()
            && matches!(
                event,
                Event::Window(iced_core::window::Event::RedrawRequested(_))
            )
            && let Some(content) = layout.children().next()
        {
            let geometry = (layout.bounds(), content.bounds());
            let state = tree.state.downcast_mut::<LocalScrollState>();
            if state.geometry != Some(geometry) {
                state.geometry = Some(geometry);
                shell.publish(self.observe.unwrap()(geometry.0, geometry.1));
            }
        }
        if let Some(id) = self.scroll_id.as_ref() {
            let mut messages = Vec::new();
            let mut local = Shell::new(&mut messages);
            if let Some(hook) = self.event_hook {
                let replay = hook(
                    event,
                    layout,
                    cursor,
                    &mut tree.state.downcast_mut::<LocalScrollState>().replay_step,
                    self.scroll_id.as_ref(),
                );
                for message in replay.messages {
                    shell.publish(message);
                }
                if replay.skip_input {
                    return;
                }
                for (synthetic, cursor) in replay.events {
                    self.base.as_widget_mut().update(
                        &mut tree.children[0],
                        &synthetic,
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        &mut local,
                        viewport,
                    );
                }
                if replay.redraw {
                    local.request_redraw();
                }
            }
            self.base.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                &mut local,
                viewport,
            );
            // Drain the local messages with a temporary shell, preserving all metadata.
            let mut forwarded = Vec::new();
            let mut filtered = Shell::new(&mut forwarded);
            filtered.merge(local, |m| m);
            let mut requests = Vec::new();
            // Shell has no filter-merge API; relay metadata before processing its buffer.
            let mut empty = Vec::new();
            let mut metadata = Shell::new(&mut empty);
            metadata.request_redraw_at(filtered.redraw_request());
            if filtered.is_event_captured() {
                metadata.capture_event();
            }
            if filtered.is_layout_invalid() {
                metadata.invalidate_layout();
            }
            if filtered.are_widgets_invalid() {
                metadata.invalidate_widgets();
            }
            metadata.request_input_method(filtered.input_method());
            drop(filtered);
            for message in forwarded {
                match self
                    .filter
                    .map_or(ScrollAction::Forward, |filter| filter(&message))
                {
                    ScrollAction::Vertical(fraction) => requests.push(fraction),
                    ScrollAction::Discard => {}
                    ScrollAction::Forward => {
                        if let Some(hook) = self.before_forward {
                            hook(&message, cursor, shell);
                        }
                        shell.publish(message);
                    }
                }
            }
            shell.merge(metadata, |m| m);
            for fraction in requests {
                let mut operation = iced_core::widget::operation::scrollable::snap_to::<()>(
                    id.clone(),
                    iced_core::widget::operation::scrollable::RelativeOffset {
                        x: None,
                        y: Some(fraction.clamp(0.0, 1.0)),
                    },
                );
                self.base.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut operation,
                );
                shell.request_redraw();
            }
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut SyncRail {
                    id: id.clone(),
                    metrics: None,
                },
            );
        } else {
            self.base.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
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
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        op: &mut dyn Operation,
    ) {
        self.base
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, op);
        if let Some(id) = self.scroll_id.as_ref() {
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut SyncRail {
                    id: id.clone(),
                    metrics: None,
                },
            );
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.base.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
/// Permanently reserved rail: the track remains visible even with no overflow.
pub struct Rail<M> {
    pub id: &'static str,
    pub on_scroll: fn(f32) -> M,
    pub tokens: Tokens,
    pub fraction: f32,
    pub visible: f32,
}
#[derive(Default)]
pub struct ScrollRailState {
    grab: Option<f32>,
    metrics: Option<(f32, f32)>,
}
impl<M> Widget<M, Theme, Renderer> for Rail<M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScrollRailState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(ScrollRailState::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(10.0), Length::Fill)
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(10.0, limits.max().height))
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        let (fraction, visible) = tree
            .state
            .downcast_ref::<ScrollRailState>()
            .metrics
            .unwrap_or((self.fraction, self.visible));
        let b = layout.bounds();
        let track = Rectangle {
            x: b.x + 3.0,
            y: b.y,
            width: 4.0,
            height: b.height,
        };
        RailVisual {
            track,
            thumb: if visible < 1.0 {
                let height = (track.height * visible).max(24.0).min(track.height);
                Some(Rectangle {
                    y: track.y + (track.height - height) * fraction,
                    height,
                    x: b.x + 2.0,
                    width: 6.0,
                })
            } else {
                None
            },
            track_color: color(self.tokens.border),
            thumb_color: color(self.tokens.secondary),
            track_radius: 3.0,
            thumb_radius: 3.0,
        }
        .draw_styled(renderer, self.tokens.visual.scrollbar);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<ScrollRailState>();
        let (fraction, visible) = state.metrics.unwrap_or((self.fraction, self.visible));
        let b = layout.bounds();
        let h = (b.height * visible).max(24.0).min(b.height);
        let emit = |position: Point, grab: f32| {
            (self.on_scroll)(((position.y - b.y - grab) / (b.height - h).max(1.0)).clamp(0.0, 1.0))
        };
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta })
                if cursor.is_over(b) && visible < 1.0 =>
            {
                let distance = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * 40.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -*y,
                };
                let overflow = (b.height / visible.max(0.0001) - b.height).max(1.0);
                shell.publish((self.on_scroll)(
                    (fraction + distance / overflow).clamp(0.0, 1.0),
                ));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(b) && visible < 1.0 =>
            {
                if let Some(p) = cursor.position() {
                    let top = b.y + (b.height - h) * fraction;
                    let grab = if p.y >= top && p.y < top + h {
                        p.y - top
                    } else {
                        h / 2.0
                    };
                    state.grab = Some(grab);
                    shell.publish(emit(p, grab));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.grab.is_some() => {
                shell.publish(emit(*position, state.grab.unwrap()));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.grab.take().is_some() {
                    shell.capture_event();
                }
            }
            Event::Window(iced_core::window::Event::Unfocused)
            | Event::Mouse(mouse::Event::CursorLeft) => state.grab = None,
            _ => {}
        }
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _: &Renderer,
        op: &mut dyn Operation,
    ) {
        op.custom(
            Some(&self.id.into()),
            layout.bounds(),
            tree.state.downcast_mut::<ScrollRailState>(),
        );
    }
    fn mouse_interaction(
        &self,
        _: &Tree,
        _: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }
}

impl ScrollRailState {
    pub fn metrics(&self) -> Option<(f32, f32)> {
        self.metrics
    }
    pub fn is_dragging(&self) -> bool {
        self.grab.is_some()
    }
}
/// Permanently reserved, ten-DIP external rail. Track/thumb geometry, minimum
/// thumb size and wheel step suit compact software-rendered views.
pub struct ScrollRail<M> {
    inner: Rail<M>,
}
impl<M> ScrollRail<M> {
    pub fn new(tokens: Tokens, id: &'static str, action: fn(f32) -> M) -> Self {
        Self {
            inner: Rail {
                id,
                on_scroll: action,
                tokens,
                fraction: 0.0,
                visible: 1.0,
            },
        }
    }
    pub fn metrics(mut self, fraction: f32, visible: f32) -> Self {
        self.inner.fraction = fraction;
        self.inner.visible = visible;
        self
    }
    pub fn build(self) -> Rail<M> {
        self.inner
    }
}
impl<'a, M: 'a> From<ScrollRail<M>> for Element<'a, M> {
    fn from(spec: ScrollRail<M>) -> Self {
        Element::new(spec.inner)
    }
}

// An inset horizontal rail sharing the native scrollable's local offset.
// The custom vertical file rail stays at the viewport edge; neither rail overlaps the other.
const HORIZONTAL_MARGIN: f32 = 16.;
const HORIZONTAL_FOOTER: f32 = 10.;
impl<'a, M: 'a> HorizontalScrollRail<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(
        appearance: Appearance,
        content: impl Into<Element<'a, M>>,
        content_width: f32,
    ) -> Self {
        Self {
            content: content.into(),
            content_width,
            appearance,
        }
    }
}
impl<'a, M: 'a> From<HorizontalScrollRail<'a, M>> for Element<'a, M> {
    fn from(spec: HorizontalScrollRail<'a, M>) -> Self {
        Element::new(spec)
    }
}
pub struct HorizontalScrollRail<'a, M> {
    content: Element<'a, M>,
    content_width: f32,
    appearance: Appearance,
}
#[derive(Default)]
struct HorizontalState {
    offset: f32,
    grab: Option<f32>,
    hover: bool,
}
struct Offset {
    value: Option<f32>,
    current: f32,
    done: bool,
}
impl Operation for Offset {
    fn scrollable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        if self.done {
            return;
        }
        self.done = true;
        self.current = translation.x;
        if let Some(value) = self.value {
            self.current = value.clamp(0., (content.width - bounds.width).max(0.));
            state.scroll_to(iced_core::widget::operation::scrollable::AbsoluteOffset {
                x: Some(self.current),
                y: None,
            });
        }
    }
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        if !self.done {
            f(self);
        }
    }
}
fn horizontal_track(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + HORIZONTAL_MARGIN,
        y: bounds.y + bounds.height - 6.,
        width: (bounds.width - HORIZONTAL_MARGIN * 2.).max(0.),
        height: 6.,
    }
}
fn horizontal_thumb(bounds: Rectangle, content: f32, offset: f32) -> Rectangle {
    let rail = horizontal_track(bounds);
    let width = (rail.width * bounds.width / content).clamp(12_f32.min(rail.width), rail.width);
    let overflow = (content - bounds.width).max(1.);
    Rectangle {
        x: rail.x + (rail.width - width) * offset.clamp(0., overflow) / overflow,
        width,
        ..rail
    }
}
impl<M> Widget<M, Theme, Renderer> for HorizontalScrollRail<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HorizontalState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(HorizontalState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.content]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        let max = limits.max();
        let footer = if self.content_width > max.width {
            HORIZONTAL_FOOTER
        } else {
            0.
        };
        let size = Size::new(max.width, (max.height - footer).max(0.));
        let child = self.content.as_widget_mut().layout(
            &mut t.children[0],
            r,
            &layout::Limits::new(Size::ZERO, size)
                .width(Length::Fill)
                .height(Length::Fill),
        );
        let mut offset = Offset {
            value: None,
            current: 0.,
            done: false,
        };
        self.content.as_widget_mut().operate(
            &mut t.children[0],
            Layout::new(&child),
            r,
            &mut offset,
        );
        t.state.downcast_mut::<HorizontalState>().offset = offset.current;
        layout::Node::with_children(
            Size::new(child.size().width, child.size().height + footer),
            vec![child],
        )
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
        let child = l.children().next().unwrap();
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, child, c, v);
        let bounds = l.bounds();
        if self.content_width <= bounds.width {
            return;
        }
        let Some(clip) = bounds.intersection(v) else {
            return;
        };
        let state = t.state.downcast_ref::<HorizontalState>();
        let tokens = self.appearance.tokens(th);
        r.with_layer(clip, |r| {
            RailVisual {
                track: horizontal_track(bounds),
                thumb: Some(horizontal_thumb(bounds, self.content_width, state.offset)),
                track_color: color(tokens.border),
                thumb_color: color(if state.hover || state.grab.is_some() {
                    tokens.text
                } else {
                    tokens.secondary
                }),
                track_radius: 3.0,
                thumb_radius: 3.0,
            }
            .draw_styled(r, tokens.visual.scrollbar);
        });
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
        let bounds = l.bounds();
        let overflow = (self.content_width - bounds.width).max(0.);
        let state = t.state.downcast_mut::<HorizontalState>();
        let rail = horizontal_track(bounds);
        let over = c
            .position()
            .is_some_and(|p| rail.expand(3.).contains(p) && v.contains(p));
        let mut target = None;
        if overflow > 0. {
            match e {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                    let p = c.position().unwrap();
                    let knob = horizontal_thumb(bounds, self.content_width, state.offset);
                    state.grab = Some(if knob.contains(p) {
                        p.x - knob.x
                    } else {
                        knob.width / 2.
                    });
                    target = Some(p.x);
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) if state.grab.is_some() => {
                    target = c.position().map(|p| p.x)
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    if state.grab.take().is_some() =>
                {
                    s.request_redraw();
                    s.capture_event();
                    return;
                }
                Event::Window(iced_core::window::Event::Unfocused) => state.grab = None,
                _ => {}
            }
        } else {
            state.grab = None;
        }
        if state.hover != over {
            state.hover = over;
            s.request_redraw();
        }
        if let Some(x) = target {
            let knob = horizontal_thumb(bounds, self.content_width, state.offset);
            let fraction =
                (x - rail.x - state.grab.unwrap_or(0.)) / (rail.width - knob.width).max(1.);
            let mut offset = Offset {
                value: Some(fraction.clamp(0., 1.) * overflow),
                current: 0.,
                done: false,
            };
            self.content.as_widget_mut().operate(
                &mut t.children[0],
                l.children().next().unwrap(),
                r,
                &mut offset,
            );
            state.offset = offset.current;
            s.request_redraw();
            s.capture_event();
            return;
        }
        self.content.as_widget_mut().update(
            &mut t.children[0],
            e,
            l.children().next().unwrap(),
            c,
            r,
            cb,
            s,
            v,
        );
        let mut offset = Offset {
            value: None,
            current: 0.,
            done: false,
        };
        self.content.as_widget_mut().operate(
            &mut t.children[0],
            l.children().next().unwrap(),
            r,
            &mut offset,
        );
        t.state.downcast_mut::<HorizontalState>().offset = offset.current;
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content.as_widget_mut().operate(
            &mut t.children[0],
            l.children().next().unwrap(),
            r,
            o,
        );
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        if t.state.downcast_ref::<HorizontalState>().grab.is_some()
            || (self.content_width > l.bounds().width
                && c.is_over(horizontal_track(l.bounds()).expand(3.)))
        {
            mouse::Interaction::Pointer
        } else {
            self.content.as_widget().mouse_interaction(
                &t.children[0],
                l.children().next().unwrap(),
                c,
                v,
                r,
            )
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
        self.content.as_widget_mut().overlay(
            &mut t.children[0],
            l.children().next().unwrap(),
            r,
            v,
            tr,
        )
    }
}

#[cfg(test)]
mod horizontal_tests {
    use super::*;
    #[test]
    fn rail_has_equal_margins_and_clears_the_vertical_rail() {
        let bounds = Rectangle {
            x: 20.,
            y: 30.,
            width: 320.,
            height: 400.,
        };
        let rail = horizontal_track(bounds);
        assert_eq!(rail.x - bounds.x, 16.);
        assert_eq!(bounds.x + bounds.width - (rail.x + rail.width), 16.);
        assert_eq!(rail.x + rail.width, bounds.x + bounds.width - 16.);
        let start = horizontal_thumb(bounds, 1000., 0.);
        let end = horizontal_thumb(bounds, 1000., 680.);
        assert_eq!(start.x, rail.x);
        assert!((end.x + end.width - rail.x - rail.width).abs() < 0.001);
    }
    #[test]
    fn dragging_rail_updates_native_offset_without_application_messages() {
        let content = iced_widget::scrollable(iced_widget::Space::new().width(1000).height(300))
            .direction(iced_widget::scrollable::Direction::Horizontal(
                iced_widget::scrollable::Scrollbar::hidden(),
            ))
            .width(Length::Fill)
            .height(Length::Fill);
        let mut view: Element<'_, ()> = HorizontalScrollRail::new(
            Appearance::themed(iced_core::Font::DEFAULT, crate::palette::from_theme),
            content,
            1000.,
        )
        .into();
        let mut tree = Tree::new(&view);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let size = Size::new(320., 200.);
        let node =
            view.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let mut messages = vec![];
        for (event, p) in [
            (
                mouse::Event::ButtonPressed(mouse::Button::Left),
                iced_core::Point::new(20., 197.),
            ),
            (
                mouse::Event::CursorMoved {
                    position: iced_core::Point::new(300., 197.),
                },
                iced_core::Point::new(300., 197.),
            ),
            (
                mouse::Event::ButtonReleased(mouse::Button::Left),
                iced_core::Point::new(300., 197.),
            ),
        ] {
            view.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                Layout::new(&node),
                mouse::Cursor::Available(p),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &node.bounds(),
            );
        }
        assert!(messages.is_empty());
        let mut offset = Offset {
            value: None,
            current: 0.,
            done: false,
        };
        view.as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut offset);
        assert!(offset.current > 600.);
    }
}
