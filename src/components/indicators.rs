// SPDX-License-Identifier: Apache-2.0
//! Semantic badge and determinate progress visuals. Labels/status semantics are
//! supplied by the app; progress never introduces an animation or command delay.
use super::Appearance;
use crate::{
    animation::Element,
    theme::{blend, color},
};
use iced_core::{Length, Padding, Theme};
use iced_widget::{container, progress_bar};
use std::{borrow::Cow, ops::RangeInclusive};
pub struct StatusBadge<'a, M> {
    inner: iced_widget::Container<'a, M>,
    appearance: Option<Appearance>,
}
impl<'a, M: 'a> StatusBadge<'a, M> {
    pub fn new(appearance: Appearance, label: impl Into<Cow<'a, str>>, tone: u32) -> Self {
        Self::custom(
            container(
                super::text(appearance.font, label.into())
                    .size(crate::design::text::BADGE)
                    .color(color(tone)),
            )
            .center_x(64)
            .center_y(crate::design::size::BADGE)
            .style(move |theme| {
                let t = appearance.tokens(theme);
                t.visual.badge.container_style(container::Style {
                    background: Some(blend(t.surface, tone, t.opacity_selected).into()),
                    border: iced_core::Border {
                        radius: crate::design::radius::BADGE.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            }),
        )
        .with_appearance(appearance)
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
    pub fn center_y(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.center_y(value);
        self
    }
    pub fn center_x(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.center_x(value);
        self
    }
    pub fn clip(mut self, value: bool) -> Self {
        self.inner = self.inner.clip(value);
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
    pub fn style(mut self, style: impl Fn(&Theme) -> container::Style + 'a) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    pub fn build(self) -> iced_widget::Container<'a, M> {
        self.inner
    }
}
impl<'a, M: 'a> From<StatusBadge<'a, M>> for Element<'a, M> {
    fn from(spec: StatusBadge<'a, M>) -> Self {
        let appearance = spec.appearance;
        let inner = spec.build();
        if let Some(appearance) = appearance {
            super::Surface::new(appearance, inner)
                .role(super::SurfaceRole::Badge)
                .decorate()
                .into()
        } else {
            inner.into()
        }
    }
}
pub struct Progress<'a> {
    inner: iced_widget::ProgressBar<'a>,
}
impl<'a> Progress<'a> {
    pub fn new(appearance: Appearance, range: RangeInclusive<f32>, value: f32, tone: u32) -> Self {
        Self::custom(iced_widget::progress_bar(range, value))
            .style(move |theme| appearance.tokens(theme).progress(tone)(theme))
    }
    pub fn custom(inner: iced_widget::ProgressBar<'a>) -> Self {
        Self { inner }
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.length(value);
        self
    }
    pub fn height(self, value: impl Into<Length>) -> Self {
        self.girth(value)
    }
    pub fn girth(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.girth(value);
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme) -> progress_bar::Style + 'a) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    pub fn build(self) -> iced_widget::ProgressBar<'a> {
        self.inner
    }
}
impl<'a, M: 'a> From<Progress<'a>> for Element<'a, M> {
    fn from(spec: Progress<'a>) -> Self {
        spec.build().into()
    }
}
pub fn progress(range: RangeInclusive<f32>, value: f32) -> Progress<'static> {
    Progress::custom(iced_widget::progress_bar(range, value))
}
