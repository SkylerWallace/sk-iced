// SPDX-License-Identifier: Apache-2.0
//! Layout builders: models, stable IDs, selection and commands remain app-owned.
use super::{Appearance, Button, ButtonTone};
use crate::{
    animation::{Element, keyed, reveal},
    motion::Policy,
};
use iced_core::{Length, Padding, Theme};
use iced_widget::{column, container, row};
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub enum NavigationAxis {
    Horizontal,
    Vertical,
}
/// A sidebar or top navigation layout. Arbitrary items permit rich icons, badges,
/// dividers and collapse affordances without teaching the library about routes.
pub struct Navigation<'a, M> {
    items: Vec<Element<'a, M>>,
    axis: NavigationAxis,
    spacing: f32,
    width: Option<Length>,
    height: Option<Length>,
}
impl<'a, M: 'a> Navigation<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(axis: NavigationAxis) -> Self {
        Self {
            items: vec![],
            axis,
            spacing: 0.0,
            width: None,
            height: None,
        }
    }
    pub fn horizontal() -> Self {
        Self::new(NavigationAxis::Horizontal)
    }
    pub fn vertical() -> Self {
        Self::new(NavigationAxis::Vertical)
    }
    pub fn push(mut self, item: impl Into<Element<'a, M>>) -> Self {
        self.items.push(item.into());
        self
    }
    pub fn spacing(mut self, value: impl Into<iced_core::Pixels>) -> Self {
        self.spacing = value.into().0;
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = Some(value.into());
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = Some(value.into());
        self
    }
}
impl<'a, M: Clone + 'a> Navigation<'a, M> {
    /// A default route item; callers may instead push a customized shared Button.
    pub fn item(
        self,
        appearance: Appearance,
        policy: Policy,
        label: impl Into<Cow<'a, str>>,
        selected: bool,
        action: Option<M>,
    ) -> Self {
        self.push(
            Button::new(appearance, policy)
                .text(label)
                .tone(if selected {
                    ButtonTone::Important
                } else {
                    ButtonTone::Default
                })
                .on_press_maybe(action),
        )
    }
}
impl<'a, M: 'a> From<Navigation<'a, M>> for Element<'a, M> {
    fn from(spec: Navigation<'a, M>) -> Self {
        match spec.axis {
            NavigationAxis::Horizontal => {
                let mut layout = row(spec.items).spacing(spec.spacing);
                if let Some(width) = spec.width {
                    layout = layout.width(width);
                }
                if let Some(height) = spec.height {
                    layout = layout.height(height);
                }
                layout.into()
            }
            NavigationAxis::Vertical => {
                let mut layout = column(spec.items).spacing(spec.spacing);
                if let Some(width) = spec.width {
                    layout = layout.width(width);
                }
                if let Some(height) = spec.height {
                    layout = layout.height(height);
                }
                layout.into()
            }
        }
    }
}

/// Shared panel surface. Specialized layouts can retain their native container.
pub struct Panel<'a, M> {
    inner: iced_widget::Container<'a, M>,
    appearance: Option<Appearance>,
}
impl<'a, M: 'a> Panel<'a, M> {
    /// State-preserving panel topology animation; resize updates retain the
    /// existing engine's immediate layout and pointer-resize snapping behavior.
    pub fn grid<S: Clone + 'static>(
        items: Vec<keyed::Item<'a, M, S>>,
        regions: impl Fn(iced_core::Size) -> Vec<iced_core::Rectangle> + 'a,
        size: iced_core::Size,
        topology: String,
        policy: Policy,
        paint: impl Fn(&S, f32) -> Element<'a, M> + 'a,
    ) -> Element<'a, M> {
        keyed::grid(items, regions, size, topology, policy, paint)
    }

    pub fn new(appearance: Appearance, body: impl Into<Element<'a, M>>) -> Self {
        Self {
            inner: super::card(appearance, body, super::CardLayout::default()),
            appearance: Some(appearance),
        }
    }
    pub fn with_header(
        appearance: Appearance,
        header: impl Into<Element<'a, M>>,
        body: impl Into<Element<'a, M>>,
    ) -> Self {
        Self::new(appearance, column![header.into(), body.into()])
    }
    pub fn custom(inner: iced_widget::Container<'a, M>) -> Self {
        Self {
            inner,
            appearance: None,
        }
    }
    pub fn with_appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
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
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn clip(mut self, value: bool) -> Self {
        self.inner = self.inner.clip(value);
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    pub fn build(self) -> iced_widget::Container<'a, M> {
        self.inner
    }
}
impl<'a, M: 'a> From<Panel<'a, M>> for Element<'a, M> {
    fn from(spec: Panel<'a, M>) -> Self {
        if let Some(appearance) = spec.appearance {
            super::Surface::new(appearance, spec.inner)
                .decorate()
                .into()
        } else {
            spec.inner.into()
        }
    }
}

/// Header plus lazy, policy-controlled expanding content. Header actions remain
/// live during transitions; closing bodies immediately lose input and operations.
pub struct ExpandableCard<'a, M> {
    appearance: Appearance,
    policy: Policy,
    header: Option<Element<'a, M>>,
    body: Option<Box<dyn Fn() -> Element<'a, M> + 'a>>,
    assembled: Option<Element<'a, M>>,
    expanded: bool,
    animated: bool,
    spacing: f32,
}
impl<'a, M: 'a> ExpandableCard<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(appearance: Appearance, policy: Policy, header: impl Into<Element<'a, M>>) -> Self {
        Self {
            appearance,
            policy,
            header: Some(header.into()),
            body: None,
            assembled: None,
            expanded: false,
            animated: true,
            spacing: 0.0,
        }
    }
    /// Preserve an existing header/body layout with app-specific content mounting.
    pub fn custom(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            appearance,
            policy: Policy::DISABLED,
            header: None,
            body: None,
            assembled: Some(content.into()),
            expanded: false,
            animated: false,
            spacing: 0.0,
        }
    }
    pub fn body(mut self, build: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.body = Some(Box::new(build));
        self
    }
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn animated(mut self, value: bool) -> Self {
        self.animated = value;
        self
    }
    pub fn spacing(mut self, value: impl Into<iced_core::Pixels>) -> Self {
        self.spacing = value.into().0;
        self
    }
    /// Reuse the same reveal within a customized header/card layout.
    pub fn reveal(
        open: bool,
        policy: Policy,
        build: impl Fn() -> Element<'a, M> + 'a,
    ) -> Element<'a, M> {
        reveal::vertical(open, policy, build)
    }
}
impl<'a, M: 'a> From<ExpandableCard<'a, M>> for Element<'a, M> {
    fn from(spec: ExpandableCard<'a, M>) -> Self {
        let content = if let Some(content) = spec.assembled {
            content
        } else {
            let mut parts = column![spec.header.unwrap()].spacing(spec.spacing);
            if let Some(body) = spec.body {
                if spec.animated {
                    parts = parts.push(reveal::vertical(spec.expanded, spec.policy, body));
                } else if spec.expanded {
                    parts = parts.push(body());
                }
            }
            parts.into()
        };
        let appearance = spec.appearance;
        let body =
            super::card(appearance, content, super::CardLayout::default()).style(move |theme| {
                let t = appearance.tokens(theme);
                t.visual.disclosure.container_style(t.card(false))
            });
        super::Surface::new(appearance, body)
            .role(super::SurfaceRole::Disclosure)
            .decorate()
            .into()
    }
}

pub struct Tab<'a, M> {
    content: Element<'a, M>,
}
impl<'a, M: Clone + 'a> Tab<'a, M> {
    /// Optional independent close action. Use parts/custom to supply rich labels,
    /// icons or a different close-control appearance without changing tab state.
    pub fn close(self, appearance: Appearance, policy: Policy, action: Option<M>) -> Self {
        if action.is_none() {
            return self;
        }
        Self::parts(
            self.content,
            Button::new(appearance, policy)
                .text("×")
                .width(28)
                .height(30)
                .padding(0)
                .on_press_maybe(action)
                .style(move |theme, status| {
                    super::panel_control_style(appearance.tokens(theme), status)
                }),
        )
    }

    /// Default text tab; selection is external and actions dispatch immediately.
    pub fn new(
        appearance: Appearance,
        policy: Policy,
        label: impl Into<Cow<'a, str>>,
        selected: bool,
        action: Option<M>,
    ) -> Self {
        Self::custom(
            Button::new(appearance, policy)
                .text(label)
                .height(30)
                .tone(if selected {
                    ButtonTone::Important
                } else {
                    ButtonTone::Default
                })
                .on_press_maybe(action),
        )
    }
}
impl<'a, M: 'a> Tab<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn custom(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
        }
    }
    /// Independently interactive title and close controls; no encompassing button
    /// that could accidentally select a tab when the close control is clicked.
    pub fn parts(title: impl Into<Element<'a, M>>, close: impl Into<Element<'a, M>>) -> Self {
        Self::custom(row![title.into(), close.into()].align_y(iced_core::Alignment::Center))
    }
}
impl<'a, M: 'a> From<Tab<'a, M>> for Element<'a, M> {
    fn from(spec: Tab<'a, M>) -> Self {
        spec.content
    }
}

/// Stable-key tab strip with the existing state-preserving animation engine.
/// Snapshots must contain small owned paint data, never app tab/worker state.
type TabPainter<'a, M, S> = Box<dyn Fn(&S, f32) -> Element<'a, M> + 'a>;
pub struct TabBar<'a, M, S> {
    items: Vec<keyed::Item<'a, M, S>>,
    policy: Policy,
    overflow: Option<(u64, f32, f32)>,
    paint: TabPainter<'a, M, S>,
}
impl<'a, M: 'a, S: Clone + 'static> TabBar<'a, M, S> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(policy: Policy, paint: impl Fn(&S, f32) -> Element<'a, M> + 'a) -> Self {
        Self {
            items: vec![],
            policy,
            paint: Box::new(paint),
            overflow: None,
        }
    }
    /// Hidden native horizontal rail plus selection reveal. Manual scrolling
    /// is retained until selection/range or viewport width changes.
    pub fn overflow(mut self, identity: u64, start: f32, end: f32) -> Self {
        self.overflow = Some((identity, start, end));
        self
    }
    pub fn items(mut self, items: Vec<keyed::Item<'a, M, S>>) -> Self {
        self.items = items;
        self
    }
    pub fn item(mut self, key: u64, content: impl Into<Element<'a, M>>, snapshot: S) -> Self {
        self.items.push(keyed::Item {
            key: keyed::Key::Item(key),
            content: content.into(),
            snapshot: Some(snapshot),
        });
        self
    }
    pub fn control(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.items.push(keyed::Item {
            key: keyed::Key::Control,
            content: content.into(),
            snapshot: None,
        });
        self
    }
}
impl<'a, M: 'a, S: Clone + 'static> From<TabBar<'a, M, S>> for Element<'a, M> {
    fn from(spec: TabBar<'a, M, S>) -> Self {
        let content = keyed::view(spec.items, spec.policy, spec.paint);
        if let Some((identity, start, end)) = spec.overflow {
            super::TabViewport::new(
                iced_widget::scrollable(content)
                    .direction(iced_widget::scrollable::Direction::Horizontal(
                        iced_widget::scrollable::Scrollbar::hidden(),
                    ))
                    .width(Length::Fill),
                identity,
                start,
                end,
            )
            .into()
        } else {
            content
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TabSizing {
    minimum: f32,
    maximum: f32,
    reserved: f32,
    spacing: f32,
}
impl Default for TabSizing {
    fn default() -> Self {
        Self {
            minimum: 72.0,
            maximum: 188.0,
            reserved: 0.0,
            spacing: 2.0,
        }
    }
}
impl TabSizing {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn range(mut self, minimum: f32, maximum: f32) -> Self {
        self.minimum = minimum.max(0.0);
        self.maximum = maximum.max(self.minimum);
        self
    }
    pub fn reserved(mut self, width: f32) -> Self {
        self.reserved = width;
        self
    }
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    pub fn width(self, available: f32, count: usize) -> f32 {
        let count = count.max(1);
        ((available - self.reserved - self.spacing * count.saturating_sub(1) as f32) / count as f32)
            .clamp(self.minimum, self.maximum)
    }
}
