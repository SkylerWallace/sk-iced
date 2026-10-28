// SPDX-License-Identifier: Apache-2.0
//! Native tooltip overlays with shared text defaults and a custom-content path.
use super::Appearance;
use crate::animation::Element;
use iced_core::{Pixels, Theme};
use iced_widget::{container, tooltip as native_tooltip};
use std::{borrow::Cow, time::Duration};

type TooltipStyle<'a> = Box<dyn Fn(&Theme) -> container::Style + 'a>;
pub struct Tooltip<'a, M> {
    child: Element<'a, M>,
    hint: Element<'a, M>,
    position: native_tooltip::Position,
    delay: Duration,
    gap: f32,
    padding: f32,
    snap: bool,
    style: Option<TooltipStyle<'a>>,
}
impl<'a, M: 'a> Tooltip<'a, M> {
    /// Default: below the child, one-second delay, 12-DIP app-font text,
    /// seven-DIP content padding and the app's card surface. No animation.
    pub fn new(
        appearance: Appearance,
        child: impl Into<Element<'a, M>>,
        hint: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self::custom(
            child,
            container(super::text(appearance.font, hint.into()).size(crate::design::text::CAPTION))
                .padding(7)
                .style(move |theme| appearance.tokens(theme).card(false)),
            native_tooltip::Position::Bottom,
        )
        .delay(Duration::from_secs(1))
    }
    /// Retain existing rich content, styling and Iced's immediate tooltip delay.
    pub fn custom(
        child: impl Into<Element<'a, M>>,
        hint: impl Into<Element<'a, M>>,
        position: native_tooltip::Position,
    ) -> Self {
        Self {
            child: child.into(),
            hint: hint.into(),
            position,
            delay: Duration::ZERO,
            gap: 0.0,
            padding: 5.0,
            snap: true,
            style: None,
        }
    }
    pub fn position(mut self, value: native_tooltip::Position) -> Self {
        self.position = value;
        self
    }
    pub fn delay(mut self, value: Duration) -> Self {
        self.delay = value;
        self
    }
    pub fn gap(mut self, value: impl Into<Pixels>) -> Self {
        self.gap = value.into().0;
        self
    }
    pub fn padding(mut self, value: impl Into<Pixels>) -> Self {
        self.padding = value.into().0;
        self
    }
    pub fn snap_within_viewport(mut self, value: bool) -> Self {
        self.snap = value;
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self {
        self.style = Some(Box::new(style));
        self
    }
    pub fn build(self) -> iced_widget::Tooltip<'a, M> {
        let mut inner = iced_widget::tooltip(self.child, self.hint, self.position)
            .delay(self.delay)
            .gap(self.gap)
            .padding(self.padding)
            .snap_within_viewport(self.snap);
        if let Some(style) = self.style {
            inner = inner.style(style);
        }
        inner
    }
}
impl<'a, M: 'a> From<Tooltip<'a, M>> for Element<'a, M> {
    fn from(spec: Tooltip<'a, M>) -> Self {
        spec.build().into()
    }
}
/// Compatibility constructor for rich app-specific tooltip contents.
pub fn tooltip<'a, M: 'a>(
    child: impl Into<Element<'a, M>>,
    hint: impl Into<Element<'a, M>>,
    position: native_tooltip::Position,
) -> Tooltip<'a, M> {
    Tooltip::custom(child, hint, position)
}
