//! Keyed strip and panel motion with immediate logical layout and visual hit bounds.
//! Only owned paint snapshots survive removal; closed items expose no widget
//! operations, input, overlays, or application state. No per-frame app messages.
use crate::animation::primitives::{self as motion, Easing, Policy, Tween};
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};
use iced_core::{Event, Length, Point, Rectangle, Size, Theme, Vector};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Item(u64),
    Control,
}

pub struct Item<'a, M, S> {
    pub key: Key,
    pub content: Element<'a, M>,
    /// None for persistent controls, which move but never enter or leave.
    pub snapshot: Option<S>,
}

/// The builder creates paint-only outgoing items from small owned snapshots.
pub fn view<'a, M: 'a, S: Clone + 'static>(
    items: Vec<Item<'a, M, S>>,
    policy: Policy,
    paint: impl Fn(&S, f32) -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    Element::new(Strip {
        items,
        policy,
        paint: Box::new(paint),
        nodes: vec![],
        ghosts: vec![],
        grid: None,
    })
}
/// Two-dimensional variant. The first item is a persistent resize surface.
/// Layout topology changes animate; pointer resizing snaps immediately.
pub fn grid<'a, M: 'a, S: Clone + 'static>(
    items: Vec<Item<'a, M, S>>,
    regions: impl Fn(Size) -> Vec<Rectangle> + 'a,
    size: Size,
    topology: String,
    policy: Policy,
    paint: impl Fn(&S, f32) -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    Element::new(Strip {
        items,
        policy,
        paint: Box::new(paint),
        nodes: vec![],
        ghosts: vec![],
        grid: Some((Box::new(regions), size, topology)),
    })
}
struct Entry<S> {
    snapshot: Option<S>,
    target: Rectangle,
    x: Tween,
    y: Tween,
    reveal: Tween,
}
struct Ghost<S> {
    key: Key,
    snapshot: S,
    bounds: Rectangle,
    reveal: Tween,
}
struct State<S> {
    child_keys: Vec<Key>,
    order: Vec<Key>,
    entries: BTreeMap<Key, Entry<S>>,
    ghosts: Vec<Ghost<S>>,
    now: Instant,
    initialized: bool,
    topology: String,
    layout_changed: bool,
}
impl<S: Clone> State<S> {
    fn new(keys: Vec<Key>) -> Self {
        Self {
            child_keys: keys,
            order: vec![],
            entries: BTreeMap::new(),
            ghosts: vec![],
            now: Instant::now(),
            initialized: false,
            topology: String::new(),
            layout_changed: false,
        }
    }
    fn sync(&mut self, slots: Vec<(Key, Option<S>, Rectangle)>, now: Instant, policy: Policy) {
        let order: Vec<_> = slots.iter().map(|(key, _, _)| *key).collect();
        let topology_changed = std::mem::take(&mut self.layout_changed);
        let changed = self.order != order || topology_changed;
        let resized = slots.iter().any(|(key, _, bounds)| {
            self.entries
                .get(key)
                .is_some_and(|entry| entry.target != *bounds)
        });
        if changed || resized || !self.initialized {
            self.now = self.now.max(now);
        }
        let keys: BTreeSet<_> = order.iter().copied().collect();
        self.ghosts
            .retain(|g| !keys.contains(&g.key) && g.reveal.active(self.now, policy));
        let removed: Vec<_> = self
            .entries
            .keys()
            .filter(|key| !keys.contains(key))
            .copied()
            .collect();
        for key in removed {
            let entry = self.entries.remove(&key).unwrap();
            if let Some(snapshot) = entry
                .snapshot
                .filter(|_| policy.enabled() && self.initialized)
            {
                let amount = entry.reveal.value(self.now, policy);
                if amount > 0.0 {
                    let mut reveal = Tween::new(amount, self.now);
                    reveal.retarget(0.0, self.now, motion::QUICK, Easing::EaseOut, policy);
                    self.ghosts.push(Ghost {
                        key,
                        snapshot,
                        bounds: Rectangle {
                            x: entry.x.value(self.now, policy),
                            y: entry.y.value(self.now, policy) + 4.0 * (1.0 - amount),
                            ..entry.target
                        },
                        reveal,
                    });
                }
            }
        }
        // Bound transient paint work even under rapid key repeat.
        if self.ghosts.len() > 16 {
            self.ghosts.drain(..self.ghosts.len() - 16);
        }
        for (key, snapshot, bounds) in slots {
            if let Some(entry) = self.entries.get_mut(&key) {
                if resized && !changed {
                    // Resizing follows the pointer directly instead of chasing it.
                    entry.x = Tween::new(bounds.x, self.now);
                    entry.y = Tween::new(bounds.y, self.now);
                    entry.reveal = Tween::new(1.0, self.now);
                } else {
                    entry.x.retarget(
                        bounds.x,
                        self.now,
                        motion::STANDARD,
                        Easing::EaseOut,
                        policy,
                    );
                    entry.y.retarget(
                        bounds.y,
                        self.now,
                        motion::STANDARD,
                        Easing::EaseOut,
                        policy,
                    );
                    entry.y.settle_if_disabled(policy);
                    entry.x.settle_if_disabled(policy);
                    entry.reveal.settle_if_disabled(policy);
                }
                entry.target = bounds;
                entry.snapshot = snapshot;
            } else {
                let entering = self.initialized && snapshot.is_some() && policy.enabled();
                let mut reveal = Tween::new(if entering { 0.0 } else { 1.0 }, self.now);
                reveal.retarget(1.0, self.now, motion::STANDARD, Easing::EaseOut, policy);
                self.entries.insert(
                    key,
                    Entry {
                        snapshot,
                        target: bounds,
                        x: Tween::new(bounds.x, self.now),
                        y: Tween::new(bounds.y, self.now),
                        reveal,
                    },
                );
            }
        }
        if !policy.enabled() || (resized && !changed) {
            self.ghosts.clear();
        }
        self.order = order;
        self.initialized = true;
    }
    fn tick<M>(&mut self, now: Instant, policy: Policy, shell: &mut Shell<'_, M>) {
        self.now = self.now.max(now);
        for entry in self.entries.values_mut() {
            entry.y.settle_if_disabled(policy);
            entry.y.request_frame(self.now, policy, shell);
            entry.x.settle_if_disabled(policy);
            entry.reveal.settle_if_disabled(policy);
            entry.x.request_frame(self.now, policy, shell);
            entry.reveal.request_frame(self.now, policy, shell);
        }
        self.ghosts
            .retain(|ghost| ghost.reveal.active(self.now, policy));
        for ghost in &self.ghosts {
            ghost.reveal.request_frame(self.now, policy, shell);
        }
    }
}
struct GhostView<'a, M> {
    key: Key,
    content: Element<'a, M>,
    tree: Tree,
    node: layout::Node,
}
type SnapshotPainter<'a, M, S> = Box<dyn Fn(&S, f32) -> Element<'a, M> + 'a>;
type GridLayout<'a> = (Box<dyn Fn(Size) -> Vec<Rectangle> + 'a>, Size, String);
struct Strip<'a, M, S> {
    items: Vec<Item<'a, M, S>>,
    policy: Policy,
    paint: SnapshotPainter<'a, M, S>,
    nodes: Vec<layout::Node>,
    ghosts: Vec<GhostView<'a, M>>,
    grid: Option<GridLayout<'a>>,
}
fn position<'a, S>(
    node: &'a layout::Node,
    entry: &Entry<S>,
    state: &State<S>,
    origin: Point,
    policy: Policy,
) -> Layout<'a> {
    Layout::with_offset(
        Vector::new(
            origin.x + entry.x.value(state.now, policy) - node.bounds().x,
            origin.y + entry.y.value(state.now, policy) - node.bounds().y
                + 4.0 * (1.0 - entry.reveal.value(state.now, policy)),
        ),
        node,
    )
}
impl<M, S: Clone + 'static> Widget<M, Theme, Renderer> for Strip<'_, M, S> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<S>>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::<S>::new(
            self.items.iter().map(|item| item.key).collect(),
        ))
    }
    fn children(&self) -> Vec<Tree> {
        self.items
            .iter()
            .map(|item| Tree::new(&item.content))
            .collect()
    }
    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<S>>();
        let mut children: BTreeMap<_, _> = state
            .child_keys
            .iter()
            .copied()
            .zip(std::mem::take(&mut tree.children))
            .collect();
        tree.children = self
            .items
            .iter()
            .map(|item| {
                let mut child = children
                    .remove(&item.key)
                    .unwrap_or_else(|| Tree::new(&item.content));
                child.diff(&item.content);
                child
            })
            .collect();
        state.child_keys = self.items.iter().map(|item| item.key).collect();
        debug_assert_eq!(
            state.child_keys.iter().collect::<BTreeSet<_>>().len(),
            self.items.len()
        );
    }
    fn size(&self) -> Size<Length> {
        if let Some((_, size, _)) = &self.grid {
            Size::new(Length::Fixed(size.width), Length::Fill)
        } else {
            Size::new(Length::Shrink, Length::Shrink)
        }
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let grid_size = self.grid.as_ref().map(|(_, size, _)| {
            limits.resolve(Length::Fixed(size.width), Length::Fill, Size::ZERO)
        });
        let regions = self
            .grid
            .as_ref()
            .map(|(regions, _, _)| regions(grid_size.unwrap()));
        let mut x = 0.0f32;
        let mut height = 0.0f32;
        self.nodes = self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .enumerate()
            .map(|(index, (item, child))| {
                if let Some(regions) = &regions {
                    let region = regions[index];
                    return item
                        .content
                        .as_widget_mut()
                        .layout(
                            child,
                            renderer,
                            &layout::Limits::new(region.size(), region.size()),
                        )
                        .move_to(region.position());
                }
                let node = item
                    .content
                    .as_widget_mut()
                    .layout(child, renderer, &limits.loose());
                height = height.max(node.size().height);
                let width = node.size().width;
                let node = node.move_to(Point::new(x, 0.0));
                x += width + 2.0;
                node
            })
            .collect();
        let size = limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(if self.items.is_empty() { 0.0 } else { x - 2.0 }, height),
        );
        let size = grid_size.unwrap_or(size);
        for node in &mut self.nodes {
            if self.grid.is_some() {
                break;
            }
            node.move_to_mut(Point::new(
                node.bounds().x,
                (size.height - node.size().height) / 2.0,
            ));
        }
        let state = tree.state.downcast_mut::<State<S>>();
        if let Some((_, _, topology)) = &self.grid
            && state.topology != *topology
        {
            state.layout_changed = true;
            state.topology = topology.clone();
        }
        state.sync(
            self.items
                .iter()
                .zip(&self.nodes)
                .map(|(item, node)| (item.key, item.snapshot.clone(), node.bounds()))
                .collect(),
            Instant::now(),
            self.policy,
        );
        self.ghosts = state
            .ghosts
            .iter()
            .map(|ghost| {
                let mut content = (self.paint)(&ghost.snapshot, ghost.bounds.width);
                let mut child = Tree::new(&content);
                let size = ghost.bounds.size();
                let node = content.as_widget_mut().layout(
                    &mut child,
                    renderer,
                    &layout::Limits::new(size, size),
                );
                crate::animation::reveal::update_visual(
                    &mut content,
                    &mut child,
                    &Event::Window(iced_core::window::Event::RedrawRequested(state.now)),
                    Layout::new(&node),
                    renderer,
                    &mut iced_core::clipboard::Null,
                    &Rectangle::with_size(size),
                );
                GhostView {
                    key: ghost.key,
                    content,
                    tree: child,
                    node,
                }
            })
            .collect();
        layout::Node::with_children(
            size,
            self.nodes
                .iter()
                .skip(usize::from(self.grid.is_some()))
                .cloned()
                .collect(),
        )
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
        let state = tree.state.downcast_mut::<State<S>>();
        // Input and drag markers use the last painted pose, not a newer clock.
        if let Event::Window(iced_core::window::Event::RedrawRequested(now)) = event {
            state.tick(*now, self.policy, shell);
        }
        self.ghosts
            .retain(|view| state.ghosts.iter().any(|ghost| ghost.key == view.key));
        for index in (0..self.items.len()).rev() {
            // Accepted panel presses must not start split drags underneath.
            if self.grid.is_some()
                && self.items[index].key == Key::Control
                && shell.is_event_captured()
                && matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(_))
                        | Event::Touch(iced_core::touch::Event::FingerPressed { .. })
                )
            {
                continue;
            }
            let child_layout = position(
                &self.nodes[index],
                &state.entries[&self.items[index].key],
                state,
                layout.position(),
                self.policy,
            );
            self.items[index].content.as_widget_mut().update(
                &mut tree.children[index],
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let state = tree.state.downcast_ref::<State<S>>();
        let draw_ghosts = |renderer: &mut Renderer| {
            for view in &self.ghosts {
                let Some(ghost) = state.ghosts.iter().find(|ghost| ghost.key == view.key) else {
                    continue;
                };
                let bounds = ghost.bounds + Vector::new(layout.position().x, layout.position().y);
                let clip = Rectangle {
                    width: bounds.width * ghost.reveal.value(state.now, self.policy),
                    ..bounds
                };
                if let Some(clip) = clip
                    .intersection(viewport)
                    .filter(|clip| clip.width > 0.0 && clip.height > 0.0)
                {
                    renderer.with_layer(clip, |renderer| {
                        view.content.as_widget().draw(
                            &view.tree,
                            renderer,
                            theme,
                            style,
                            Layout::with_offset(Vector::new(bounds.x, bounds.y), &view.node),
                            mouse::Cursor::Unavailable,
                            &clip,
                        )
                    });
                }
            }
        };
        if self.grid.is_none() {
            draw_ghosts(renderer);
        }
        for (index, item) in self.items.iter().enumerate() {
            let entry = &state.entries[&item.key];
            let child_layout = position(
                &self.nodes[index],
                entry,
                state,
                layout.position(),
                self.policy,
            );
            let reveal = entry.reveal.value(state.now, self.policy);
            // Settled grid panels may paint elevation outside their hit bounds.
            let clip = if self.grid.is_some() && reveal >= 1.0 {
                *viewport
            } else {
                Rectangle {
                    width: child_layout.bounds().width * reveal,
                    ..child_layout.bounds()
                }
            };
            if let Some(clip) = clip
                .intersection(viewport)
                .filter(|clip| clip.width > 0.0 && clip.height > 0.0)
            {
                renderer.with_layer(clip, |renderer| {
                    item.content.as_widget().draw(
                        &tree.children[index],
                        renderer,
                        theme,
                        style,
                        child_layout,
                        cursor,
                        &clip,
                    )
                });
            }
        }
        if self.grid.is_some() {
            draw_ghosts(renderer);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<State<S>>();
        // Last painted item is first for overlapping drag hit regions.
        for index in (0..self.items.len()).rev() {
            let child_layout = position(
                &self.nodes[index],
                &state.entries[&self.items[index].key],
                state,
                layout.position(),
                self.policy,
            );
            self.items[index].content.as_widget_mut().operate(
                &mut tree.children[index],
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
        let state = tree.state.downcast_ref::<State<S>>();
        for (index, item) in self.items.iter().enumerate().rev() {
            let child_layout = position(
                &self.nodes[index],
                &state.entries[&item.key],
                state,
                layout.position(),
                self.policy,
            );
            let interaction = item.content.as_widget().mouse_interaction(
                &tree.children[index],
                child_layout,
                cursor,
                viewport,
                renderer,
            );
            if interaction != mouse::Interaction::None {
                return interaction;
            }
        }
        mouse::Interaction::None
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State<S>>();
        let origin = layout.position();
        let nodes = &self.nodes;
        let policy = self.policy;
        let overlays = self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(nodes)
            .filter_map(|((item, child), node)| {
                item.content.as_widget_mut().overlay(
                    child,
                    position(node, &state.entries[&item.key], state, origin, policy),
                    renderer,
                    viewport,
                    translation,
                )
            })
            .collect::<Vec<_>>();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
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
    struct Probe(u64);
    struct ProbeState(u64);
    impl Widget<u64, Theme, Renderer> for Probe {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<ProbeState>()
        }
        fn state(&self) -> tree::State {
            tree::State::new(ProbeState(self.0))
        }
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fill)
        }
        fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
            layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
        }
        fn draw(
            &self,
            _: &Tree,
            renderer: &mut Renderer,
            _: &Theme,
            _: &renderer::Style,
            layout: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                if self.0.is_multiple_of(2) {
                    iced_core::Color::from_rgb(0.0, 1.0, 0.0)
                } else {
                    iced_core::Color::WHITE
                },
            );
        }
        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            _: &Renderer,
            operation: &mut dyn Operation,
        ) {
            operation.custom(
                None,
                layout.bounds(),
                tree.state.downcast_mut::<ProbeState>(),
            );
        }
    }
    fn strip(
        keys: &[u64],
        width: f32,
        policy: Policy,
        paints: Rc<Cell<usize>>,
    ) -> Element<'static, u64> {
        view(
            keys.iter()
                .map(|key| Item {
                    key: Key::Item(*key),
                    snapshot: Some(*key),
                    content: iced_widget::button(Element::new(Probe(*key)))
                        .width(width)
                        .height(30)
                        .padding(0)
                        .on_press(*key)
                        .into(),
                })
                .collect(),
            policy,
            move |key, width| {
                paints.set(paints.get() + 1);
                iced_widget::container(Element::new(Probe(*key)))
                    .width(width)
                    .height(30)
                    .into()
            },
        )
    }
    fn layout(widget: &mut Element<'_, u64>, tree: &mut Tree, renderer: &Renderer) -> layout::Node {
        widget.as_widget_mut().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(f32::INFINITY, 30.0)),
        )
    }
    fn event(
        widget: &mut Element<'_, u64>,
        tree: &mut Tree,
        renderer: &Renderer,
        node: &layout::Node,
        event: Event,
        cursor: mouse::Cursor,
    ) -> (Vec<u64>, iced_core::window::RedrawRequest) {
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        widget.as_widget_mut().update(
            tree,
            &event,
            Layout::new(node),
            cursor,
            renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(500.0, 30.0)),
        );
        let request = shell.redraw_request();
        assert!(
            !shell.is_layout_invalid(),
            "strip motion must stay local to paint/input geometry"
        );
        (messages, request)
    }
    fn frame(now: Instant) -> Event {
        Event::Window(iced_core::window::Event::RedrawRequested(now))
    }
    fn click(
        widget: &mut Element<'_, u64>,
        tree: &mut Tree,
        renderer: &Renderer,
        node: &layout::Node,
        point: Point,
    ) -> Vec<u64> {
        let cursor = mouse::Cursor::Available(point);
        event(
            widget,
            tree,
            renderer,
            node,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            cursor,
        );
        event(
            widget,
            tree,
            renderer,
            node,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            cursor,
        )
        .0
    }
    #[derive(Default)]
    struct Bounds(Vec<(u64, Rectangle)>);
    impl Operation for Bounds {
        fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
            f(self);
        }
        fn custom(
            &mut self,
            _: Option<&iced_core::widget::Id>,
            bounds: Rectangle,
            state: &mut dyn std::any::Any,
        ) {
            if let Some(state) = state.downcast_ref::<ProbeState>() {
                self.0.push((state.0, bounds));
            }
        }
    }
    fn bounds(
        widget: &mut Element<'_, u64>,
        tree: &mut Tree,
        renderer: &Renderer,
        node: &layout::Node,
    ) -> Bounds {
        let mut bounds = Bounds::default();
        widget
            .as_widget_mut()
            .operate(tree, Layout::new(node), renderer, &mut bounds);
        bounds
    }

    #[test]
    fn keyed_children_and_visual_hit_bounds_survive_reordering() {
        let paints = Rc::new(Cell::new(0));
        let mut widget = strip(&[1, 2, 3], 80.0, ON, paints.clone());
        let mut tree = Tree::new(&widget);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let initial = layout(&mut widget, &mut tree, &renderer);
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &initial,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            mouse::Cursor::Available(Point::new(122.0, 15.0)),
        );
        widget = strip(&[3, 1, 2], 80.0, ON, paints);
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        assert_eq!(
            event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                mouse::Cursor::Available(Point::new(122.0, 15.0))
            )
            .0,
            vec![2],
            "a pressed button must follow its tab identity across a reorder"
        );
        let current = bounds(&mut widget, &mut tree, &renderer, &node);
        assert_eq!(
            current.0.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            vec![2, 1, 3]
        );
        let third = current.0.iter().find(|(key, _)| *key == 3).unwrap().1;
        assert_eq!(third.x, 164.0);
        assert_eq!(
            click(&mut widget, &mut tree, &renderer, &node, third.center()),
            vec![3]
        );
        let now = tree.state.downcast_ref::<State<u64>>().now;
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + motion::STANDARD / 2),
            mouse::Cursor::Unavailable,
        );
        let current = bounds(&mut widget, &mut tree, &renderer, &node);
        let third = current.0.iter().find(|(key, _)| *key == 3).unwrap().1;
        assert!(third.x > 0.0 && third.x < 164.0);
        assert_eq!(
            click(&mut widget, &mut tree, &renderer, &node, third.center()),
            vec![3]
        );
        let (_, request) = event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + motion::STANDARD),
            mouse::Cursor::Unavailable,
        );
        assert_eq!(request, iced_core::window::RedrawRequest::Wait);
        assert_eq!(
            bounds(&mut widget, &mut tree, &renderer, &node)
                .0
                .iter()
                .find(|(key, _)| *key == 3)
                .unwrap()
                .1
                .x,
            0.0
        );
    }

    #[test]
    fn opening_is_immediately_actionable_and_closing_paint_has_no_input_or_operations() {
        let paints = Rc::new(Cell::new(0));
        let mut widget = strip(&[1], 80.0, ON, paints.clone());
        let mut tree = Tree::new(&widget);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        layout(&mut widget, &mut tree, &renderer);
        widget = strip(&[1, 2], 80.0, ON, paints.clone());
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        let state = tree.state.downcast_ref::<State<u64>>();
        assert_eq!(
            state.entries[&Key::Item(2)].reveal.value(state.now, ON),
            0.0
        );
        assert_eq!(
            click(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                Point::new(120.0, 15.0)
            ),
            vec![2]
        );
        let now = tree.state.downcast_ref::<State<u64>>().now;
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + motion::STANDARD),
            mouse::Cursor::Unavailable,
        );
        widget = strip(&[1], 80.0, ON, paints.clone());
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        assert_eq!(
            node.size().width,
            80.0,
            "removed items must not hold logical slots"
        );
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.state.downcast_ref::<State<u64>>().ghosts.len(), 1);
        assert_eq!(bounds(&mut widget, &mut tree, &renderer, &node).0.len(), 1);
        assert!(
            click(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                Point::new(120.0, 15.0)
            )
            .is_empty()
        );
        let count = paints.get();
        let now = tree.state.downcast_ref::<State<u64>>().now;
        for ms in [20, 40, 60, 80, 100] {
            let (_, request) = event(
                &mut widget,
                &mut tree,
                &renderer,
                &node,
                frame(now + std::time::Duration::from_millis(ms)),
                mouse::Cursor::Unavailable,
            );
            assert_eq!(
                paints.get(),
                count,
                "closing paint is not rebuilt per frame"
            );
            assert_eq!(
                request,
                if ms == 100 {
                    iced_core::window::RedrawRequest::Wait
                } else {
                    iced_core::window::RedrawRequest::NextFrame
                }
            );
        }
        assert!(tree.state.downcast_ref::<State<u64>>().ghosts.is_empty());
    }

    #[test]
    fn policy_changes_snap_and_resize_does_not_chase_pointer() {
        {
            let off = Policy::DISABLED;
            let paints = Rc::new(Cell::new(0));
            let mut widget = strip(&[1, 2, 3], 80.0, ON, paints.clone());
            let mut tree = Tree::new(&widget);
            let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
            layout(&mut widget, &mut tree, &renderer);
            widget = strip(&[3, 1], 80.0, ON, paints.clone());
            tree.diff(&widget);
            layout(&mut widget, &mut tree, &renderer);
            widget = strip(&[3, 1], 80.0, off, paints.clone());
            tree.diff(&widget);
            let node = layout(&mut widget, &mut tree, &renderer);
            let state = tree.state.downcast_ref::<State<u64>>();
            assert!(state.ghosts.is_empty());
            assert_eq!(state.entries[&Key::Item(3)].x.value(state.now, ON), 0.0);
            assert_eq!(
                event(
                    &mut widget,
                    &mut tree,
                    &renderer,
                    &node,
                    frame(Instant::now()),
                    mouse::Cursor::Unavailable
                )
                .1,
                iced_core::window::RedrawRequest::Wait
            );
            widget = strip(&[3, 1], 120.0, ON, paints);
            tree.diff(&widget);
            let node = layout(&mut widget, &mut tree, &renderer);
            let state = tree.state.downcast_ref::<State<u64>>();
            assert_eq!(state.entries[&Key::Item(1)].x.value(state.now, ON), 122.0);
            assert_eq!(
                event(
                    &mut widget,
                    &mut tree,
                    &renderer,
                    &node,
                    frame(Instant::now()),
                    mouse::Cursor::Unavailable
                )
                .1,
                iced_core::window::RedrawRequest::Wait
            );
        }
    }

    #[test]
    fn repeated_moves_are_continuous_and_exit_paint_is_bounded() {
        let slots = |keys: &[u64]| {
            keys.iter()
                .enumerate()
                .map(|(i, key)| {
                    (
                        Key::Item(*key),
                        Some(*key),
                        Rectangle::new(Point::new(i as f32 * 82.0, 0.0), Size::new(80.0, 30.0)),
                    )
                })
                .collect()
        };
        let now = Instant::now();
        let mut state = State::new(vec![]);
        state.sync(slots(&[1, 2, 3]), now, ON);
        state.sync(slots(&[3, 1, 2]), now, ON);
        let mut messages: Vec<()> = vec![];
        state.tick(now + motion::QUICK / 2, ON, &mut Shell::new(&mut messages));
        let pose = state.entries[&Key::Item(3)].x.value(state.now, ON);
        state.sync(slots(&[1, 2, 3]), state.now, ON);
        assert_eq!(state.entries[&Key::Item(3)].x.value(state.now, ON), pose);
        let keys: Vec<_> = (1..=40).collect();
        let mut state = State::new(vec![]);
        state.sync(slots(&keys), now, ON);
        for start in 1..=32 {
            state.sync(
                slots(&keys[start..]),
                now + std::time::Duration::from_millis(start as u64),
                ON,
            );
        }
        assert!(state.ghosts.len() <= 16);
        let mut shell = Shell::new(&mut messages);
        state.tick(now + std::time::Duration::from_secs(1), ON, &mut shell);
        assert_eq!(
            shell.redraw_request(),
            iced_core::window::RedrawRequest::Wait
        );
        assert!(state.ghosts.is_empty());
    }

    #[test]
    fn intermediate_paint_moves_and_stays_inside_the_viewport() {
        let paints = Rc::new(Cell::new(0));
        let mut widget = strip(&[1, 2], 80.0, ON, paints.clone());
        let mut tree = Tree::new(&widget);
        let mut renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        layout(&mut widget, &mut tree, &renderer);
        widget = strip(&[2, 1], 80.0, ON, paints);
        tree.diff(&widget);
        let node = layout(&mut widget, &mut tree, &renderer);
        let now = tree.state.downcast_ref::<State<u64>>().now;
        let viewport = Rectangle::new(Point::new(10.0, 0.0), Size::new(140.0, 30.0));
        let paint = |widget: &Element<'_, u64>, tree: &Tree, renderer: &mut Renderer| {
            renderer.reset(Rectangle::with_size(Size::new(200.0, 30.0)));
            widget.as_widget().draw(
                tree,
                renderer,
                &Theme::Dark,
                &renderer::Style::default(),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &viewport,
            );
            renderer.screenshot(Size::new(200, 30), 1.0, iced_core::Color::BLACK)
        };
        let first = paint(&widget, &tree, &mut renderer);
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + motion::STANDARD / 2),
            mouse::Cursor::Unavailable,
        );
        let middle = paint(&widget, &tree, &mut renderer);
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + motion::STANDARD),
            mouse::Cursor::Unavailable,
        );
        let last = paint(&widget, &tree, &mut renderer);
        assert_ne!(first, middle);
        assert_ne!(middle, last);
        for y in 0..30 {
            for x in (0..10).chain(150..200) {
                let i = (y * 200 + x) * 4;
                assert_eq!(&middle[i..i + 3], &[0, 0, 0]);
            }
        }
    }

    #[test]
    fn panel_grid_moves_in_two_dimensions_and_preserves_children() {
        fn board(
            keys: &[u64],
            slots: Vec<Rectangle>,
            topology: &str,
            policy: Policy,
        ) -> Element<'static, u64> {
            let mut items = vec![Item {
                key: Key::Control,
                snapshot: None,
                content: iced_widget::Space::new().into(),
            }];
            items.extend(keys.iter().map(|key| Item {
                key: Key::Item(*key),
                snapshot: Some(*key),
                content: Element::new(Probe(*key)),
            }));
            grid(
                items,
                move |size| {
                    std::iter::once(Rectangle::with_size(size))
                        .chain(slots.clone())
                        .collect()
                },
                Size::new(400.0, 300.0),
                topology.into(),
                policy,
                |key, _| Element::new(Probe(*key)),
            )
        }
        let rect = |x, y| Rectangle {
            x,
            y,
            width: 100.0,
            height: 100.0,
        };
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        let limits = layout::Limits::new(Size::new(400.0, 300.0), Size::new(400.0, 300.0));
        let mut widget = board(&[1], vec![rect(0.0, 0.0)], "one", ON);
        let mut tree = Tree::new(&widget);
        widget.as_widget_mut().layout(&mut tree, &renderer, &limits);
        tree.children[1].state.downcast_mut::<ProbeState>().0 = 91;
        widget = board(&[1, 2], vec![rect(200.0, 150.0), rect(0.0, 0.0)], "two", ON);
        tree.diff(&widget);
        let node = widget.as_widget_mut().layout(&mut tree, &renderer, &limits);
        assert_eq!(tree.children[1].state.downcast_ref::<ProbeState>().0, 91);
        let state = tree.state.downcast_ref::<State<u64>>();
        let now = state.now;
        assert_eq!(state.entries[&Key::Item(1)].y.value(now, ON), 0.0);
        assert_eq!(state.entries[&Key::Item(2)].reveal.value(now, ON), 0.0);
        event(
            &mut widget,
            &mut tree,
            &renderer,
            &node,
            frame(now + std::time::Duration::from_millis(60)),
            mouse::Cursor::Unavailable,
        );
        let state = tree.state.downcast_ref::<State<u64>>();
        let entry = &state.entries[&Key::Item(1)];
        assert!(entry.y.value(state.now, ON) > 0.0 && entry.y.value(state.now, ON) < 150.0);
        let visual = bounds(&mut widget, &mut tree, &renderer, &node);
        assert!(
            visual
                .0
                .iter()
                .any(|(key, b)| *key == 91 && b.y > 0.0 && b.y < 150.0)
        );
        widget = board(&[1], vec![rect(0.0, 0.0)], "one", ON);
        tree.diff(&widget);
        widget.as_widget_mut().layout(&mut tree, &renderer, &limits);
        assert_eq!(tree.children[1].state.downcast_ref::<ProbeState>().0, 91);
        assert_eq!(tree.state.downcast_ref::<State<u64>>().ghosts.len(), 1);
        widget = board(&[1], vec![rect(40.0, 30.0)], "one", Policy::DISABLED);
        tree.diff(&widget);
        widget.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let state = tree.state.downcast_ref::<State<u64>>();
        assert!(state.ghosts.is_empty());
        assert_eq!(state.entries[&Key::Item(1)].y.value(state.now, ON), 30.0);
    }

    #[test]
    fn topology_change_is_consumed_on_first_layout_even_when_keys_change() {
        let mut state = State::new(vec![]);
        let now = state.now;
        state.layout_changed = true;
        state.sync(
            vec![(
                Key::Item(1),
                Some(1u64),
                Rectangle::with_size(Size::new(100.0, 100.0)),
            )],
            now,
            ON,
        );
        assert!(!state.layout_changed);
        let rect = Rectangle {
            y: 40.0,
            ..Rectangle::with_size(Size::new(100.0, 100.0))
        };
        state.sync(vec![(Key::Item(1), Some(1), rect)], now, ON);
        assert_eq!(
            state.entries[&Key::Item(1)].y.value(now, ON),
            40.0,
            "layout negotiation must snap, not animate cold content"
        );
    }
}
