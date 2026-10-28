// SPDX-License-Identifier: Apache-2.0
//! Layout-only settings rows. Actions and settings values stay in the app.
use super::Appearance;
use crate::{animation::Element, theme::color};
use iced_core::{Length, Pixels};
use iced_widget::{column, container, row};
use std::borrow::Cow;

pub struct SettingsRow<'a, M> {
    appearance: Appearance,
    label: Option<Element<'a, M>>,
    control: Element<'a, M>,
    hint: Option<Cow<'a, str>>,
    spacing: f32,
    hint_spacing: f32,
    hint_size: f32,
    height: Option<Length>,
}
impl<'a, M: 'a> SettingsRow<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    /// 13.333-DIP label, fill label width, centered row, no gap or forced height.
    /// Optional secondary hint uses 11-DIP text and a four-DIP vertical gap.
    pub fn new(
        appearance: Appearance,
        label: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, M>>,
    ) -> Self {
        Self::content(appearance, control).label_content(
            super::text(appearance.font, label.into())
                .size(crate::design::text::BODY)
                .width(Length::Fill),
        )
    }
    /// A full-width control or an existing specialized layout, with no extra row.
    pub fn content(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            appearance,
            label: None,
            control: content.into(),
            hint: None,
            spacing: 0.0,
            hint_spacing: crate::design::space::SMALL,
            hint_size: crate::design::text::SMALL,
            height: None,
        }
    }
    pub fn label_content(mut self, label: impl Into<Element<'a, M>>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn spacing(mut self, value: impl Into<iced_core::Pixels>) -> Self {
        self.spacing = value.into().0;
        self
    }
    pub fn hint_spacing(mut self, value: impl Into<iced_core::Pixels>) -> Self {
        self.hint_spacing = value.into().0;
        self
    }
    pub fn hint_size(mut self, value: impl Into<Pixels>) -> Self {
        self.hint_size = value.into().0;
        self
    }
    /// Center the main row/control in this height, without changing hint spacing.
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = Some(value.into());
        self
    }
}
impl<'a, M: 'a> From<SettingsRow<'a, M>> for Element<'a, M> {
    fn from(spec: SettingsRow<'a, M>) -> Self {
        let appearance = spec.appearance;
        let mut body = if let Some(label) = spec.label {
            row![label, spec.control]
                .spacing(spec.spacing)
                .align_y(iced_core::Alignment::Center)
                .into()
        } else {
            spec.control
        };
        if let Some(height) = spec.height {
            body = container(body).center_y(height).into();
        }
        if let Some(hint) = spec.hint {
            column![
                body,
                super::text(appearance.font, hint)
                    .size(spec.hint_size)
                    .style(move |theme| iced_widget::text::Style {
                        color: Some(color(appearance.tokens(theme).secondary))
                    })
            ]
            .spacing(spec.hint_spacing)
            .into()
        } else {
            body
        }
    }
}
