// SPDX-License-Identifier: Apache-2.0
//! Static feedback states: no spinner, delay, subscription or scan-batch animation.
use super::Appearance;
use crate::{
    animation::{Element, Renderer},
    design::ColorRole,
    theme::color,
};
use iced_core::{Length, Padding, Pixels, Theme};
use iced_widget::{column, container, row};
use std::borrow::Cow;

pub struct Notice<'a, M> {
    appearance: Appearance,
    label: Cow<'a, str>,
    content: Option<Element<'a, M>>,
    tone: ColorRole,
    text_size: f32,
    icon: Option<Element<'a, M>>,
    action: Option<Element<'a, M>>,
    hint: Option<Element<'a, M>>,
    spacing: f32,
    center: Option<Length>,
    padding: Option<Padding>,
    surface: bool,
}
impl<'a, M: 'a> Notice<'a, M> {
    pub fn new(appearance: Appearance, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            appearance,
            label: label.into(),
            content: None,
            tone: ColorRole::Secondary,
            text_size: crate::design::text::BODY,
            icon: None,
            action: None,
            hint: None,
            spacing: 8.0,
            center: None,
            padding: None,
            surface: false,
        }
    }
    pub fn content(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        let mut spec = Self::new(appearance, "");
        spec.content = Some(content.into());
        spec
    }
    pub fn tone(mut self, tone: ColorRole) -> Self {
        self.tone = tone;
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into().0;
        self
    }
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn action(mut self, action: impl Into<Element<'a, M>>) -> Self {
        self.action = Some(action.into());
        self
    }
    pub fn hint(mut self, hint: impl Into<Element<'a, M>>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
    pub fn center(mut self, length: impl Into<Length>) -> Self {
        self.center = Some(length.into());
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    pub fn surface(mut self, surface: bool) -> Self {
        self.surface = surface;
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let appearance = self.appearance;
        let tone = self.tone;
        let mut body = self.content.unwrap_or_else(|| {
            super::text(appearance.font, self.label)
                .size(self.text_size)
                .style(move |theme: &Theme| iced_widget::text::Style {
                    color: Some(color(tone.resolve(appearance.tokens(theme)))),
                })
                .into()
        });
        if self.icon.is_some() || self.action.is_some() {
            let mut line = row![];
            if let Some(icon) = self.icon {
                line = line.push(icon);
            }
            line = line.push(body);
            if let Some(action) = self.action {
                line = line.push(action);
            }
            body = line
                .spacing(self.spacing)
                .align_y(iced_core::Alignment::Center)
                .into();
        }
        if let Some(hint) = self.hint {
            body = column![body, hint].spacing(4).into();
        }
        if self.center.is_none() && self.padding.is_none() && !self.surface {
            return body;
        }
        let mut panel = container(body);
        if let Some(length) = self.center {
            panel = panel.center(length);
        }
        if let Some(padding) = self.padding {
            panel = panel.padding(padding);
        }
        if self.surface {
            panel = panel.style(move |theme| appearance.tokens(theme).card(false));
        }
        panel.into()
    }
}
impl<'a, M: 'a> From<Notice<'a, M>> for Element<'a, M> {
    fn from(notice: Notice<'a, M>) -> Self {
        notice.build()
    }
}

/// Empty/loading/error presentation uses the same static layout. The message and
/// optional retry action come from the app; loading never adds a motion timer.
pub struct EmptyState<'a, M> {
    inner: Notice<'a, M>,
}
impl<'a, M: 'a> EmptyState<'a, M> {
    pub fn new(appearance: Appearance, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            inner: Notice::new(appearance, label),
        }
    }
    pub fn tone(mut self, tone: ColorRole) -> Self {
        self.inner = self.inner.tone(tone);
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.text_size(size);
        self
    }
    pub fn action(mut self, action: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.action(action);
        self
    }
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }
    pub fn center(mut self, length: impl Into<Length>) -> Self {
        self.inner = self.inner.center(length);
        self
    }
    pub fn build(self) -> Element<'a, M> {
        self.inner.build()
    }
}
impl<'a, M: 'a> From<EmptyState<'a, M>> for Element<'a, M> {
    fn from(state: EmptyState<'a, M>) -> Self {
        state.build()
    }
}

/// Native preview title/body/footer slots. Rendering, document state, export,
/// printing and async loading remain app-owned.
pub struct PreviewSurface<'a, M> {
    body: Element<'a, M>,
    toolbar: Option<Element<'a, M>>,
    footer: Option<Element<'a, M>>,
    spacing: f32,
}
impl<'a, M: 'a> PreviewSurface<'a, M> {
    pub fn new(body: impl Into<Element<'a, M>>) -> Self {
        Self {
            body: body.into(),
            toolbar: None,
            footer: None,
            spacing: 16.0,
        }
    }
    pub fn toolbar(mut self, toolbar: impl Into<Element<'a, M>>) -> Self {
        self.toolbar = Some(toolbar.into());
        self
    }
    pub fn footer(mut self, footer: impl Into<Element<'a, M>>) -> Self {
        self.footer = Some(footer.into());
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
    pub fn build(self) -> iced_widget::Column<'a, M, Theme, Renderer> {
        let mut parts = column![];
        if let Some(toolbar) = self.toolbar {
            parts = parts.push(toolbar);
        }
        parts = parts.push(self.body);
        if let Some(footer) = self.footer {
            parts = parts.push(footer);
        }
        parts.spacing(self.spacing)
    }
}
impl<'a, M: 'a> From<PreviewSurface<'a, M>> for Element<'a, M> {
    fn from(surface: PreviewSurface<'a, M>) -> Self {
        surface.build().into()
    }
}

/// Standard title/body/actions slots; existing Dialog owns modality and policy.
pub struct DialogContent<'a, M> {
    inner: PreviewSurface<'a, M>,
}
impl<'a, M: 'a> DialogContent<'a, M> {
    pub fn new(body: impl Into<Element<'a, M>>) -> Self {
        Self {
            inner: PreviewSurface::new(body),
        }
    }
    pub fn title(mut self, title: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.toolbar(title);
        self
    }
    pub fn actions(mut self, actions: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.footer(actions);
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(spacing);
        self
    }
    pub fn build(self) -> iced_widget::Column<'a, M, Theme, Renderer> {
        self.inner.build()
    }
}
impl<'a, M: 'a> From<DialogContent<'a, M>> for Element<'a, M> {
    fn from(dialog: DialogContent<'a, M>) -> Self {
        dialog.build().into()
    }
}
