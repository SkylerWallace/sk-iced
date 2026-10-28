// SPDX-License-Identifier: Apache-2.0
//! Native form composition; validation rules and settings state stay app-owned.
use super::{Appearance, InteractionGate};
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::{Length, Padding, Pixels, Theme};
use iced_widget::{Column, row, slider};
use std::{borrow::Cow, ops::RangeInclusive};

pub struct Slider<'a, T, M> {
    inner: iced_widget::Slider<'a, T, M>,
    enabled: Option<bool>,
    appearance: Appearance,
    id: Option<iced_core::widget::Id>,
}
impl<'a, T: Copy + From<u8> + PartialOrd, M: Clone + 'a> Slider<'a, T, M> {
    pub fn new(range: RangeInclusive<T>, value: T, action: impl Fn(T) -> M + 'a) -> Self {
        Self {
            inner: iced_widget::Slider::new(range, value, action).style(|theme, status| {
                crate::visual::slider_style(
                    crate::palette::from_theme(theme),
                    slider::default(theme, status),
                )
            }),
            enabled: None,
            appearance: Appearance::themed(iced_core::Font::DEFAULT, crate::palette::from_theme),
            id: None,
        }
    }
    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn with_appearance(mut self, appearance: super::Appearance) -> Self {
        self.appearance = appearance;
        self.inner = self.inner.style(move |theme, status| {
            crate::visual::slider_style(appearance.tokens(theme), slider::default(theme, status))
        });
        self
    }
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.inner = self.inner.step(step);
        self
    }
    pub fn shift_step(mut self, step: impl Into<T>) -> Self {
        self.inner = self.inner.shift_step(step);
        self
    }
    pub fn default_value(mut self, value: impl Into<T>) -> Self {
        self.inner = self.inner.default(value);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.inner = self.inner.height(height);
        self
    }
    pub fn on_release(mut self, action: M) -> Self {
        self.inner = self.inner.on_release(action);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme, slider::Status) -> slider::Style + 'a) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    pub fn build(self) -> Element<'a, M>
    where
        iced_widget::Slider<'a, T, M>: Into<Element<'a, M>>,
    {
        let inner = self.inner.into();
        let inner = if let Some(enabled) = self.enabled {
            InteractionGate::new(inner, enabled).into()
        } else {
            inner
        };
        let mut control = super::KeyboardControl::new(self.appearance, inner)
            .enabled(self.enabled.unwrap_or(true))
            .activation(false)
            .native_arrows();
        if let Some(id) = self.id {
            control = control.id(id);
        }
        control.into()
    }
}
impl<'a, T: Copy + From<u8> + PartialOrd + 'a, M: Clone + 'a> From<Slider<'a, T, M>>
    for Element<'a, M>
where
    iced_widget::Slider<'a, T, M>: Into<Element<'a, M>>,
{
    fn from(slider: Slider<'a, T, M>) -> Self {
        slider.build()
    }
}

pub struct FormField<'a, M> {
    appearance: Appearance,
    label: Cow<'a, str>,
    control: Element<'a, M>,
    label_width: Length,
    label_size: f32,
    spacing: f32,
    hint: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
}
impl<'a, M: 'a> FormField<'a, M> {
    pub fn new(
        appearance: Appearance,
        label: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, M>>,
    ) -> Self {
        Self {
            appearance,
            label: label.into(),
            control: control.into(),
            label_width: Length::Fixed(115.0),
            label_size: crate::design::text::BODY,
            spacing: 8.0,
            hint: None,
            error: None,
        }
    }
    pub fn label_width(mut self, width: impl Into<Length>) -> Self {
        self.label_width = width.into();
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.label_size = size.into().0;
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
    pub fn hint(mut self, hint: impl Into<Cow<'a, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn error(mut self, error: impl Into<Cow<'a, str>>) -> Self {
        self.error = Some(error.into());
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let label = super::text(self.appearance.font, self.label)
            .size(self.label_size)
            .width(self.label_width);
        let line: Element<'a, M> = row![label, self.control]
            .spacing(self.spacing)
            .align_y(iced_core::Alignment::Center)
            .into();
        if self.hint.is_none() && self.error.is_none() {
            return line;
        }
        let mut body = Column::new().push(line).spacing(4);
        let appearance = self.appearance;
        if let Some(hint) = self.hint {
            body = body.push(super::text(appearance.font, hint).size(11).style(
                move |theme: &Theme| iced_widget::text::Style {
                    color: Some(color(appearance.tokens(theme).secondary)),
                },
            ));
        }
        if let Some(error) = self.error {
            body = body.push(super::text(appearance.font, error).size(12).style(
                move |theme: &Theme| iced_widget::text::Style {
                    color: Some(color(appearance.tokens(theme).failure)),
                },
            ));
        }
        body.into()
    }
}
impl<'a, M: 'a> From<FormField<'a, M>> for Element<'a, M> {
    fn from(field: FormField<'a, M>) -> Self {
        field.build()
    }
}

pub struct FormSection<'a, M> {
    inner: Column<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> FormSection<'a, M> {
    pub fn new() -> Self {
        Self {
            inner: Column::new(),
        }
    }
    pub fn push(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.inner = self.inner.push(content);
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(spacing);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }
    pub fn build(self) -> Column<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> Default for FormSection<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a, M: 'a> From<FormSection<'a, M>> for Element<'a, M> {
    fn from(section: FormSection<'a, M>) -> Self {
        section.inner.into()
    }
}

/// Native choice grid; app-owned tiles retain their exact appearance/actions.
pub struct ChoiceGrid<'a, M> {
    items: Vec<Element<'a, M>>,
    columns: usize,
    spacing: f32,
    width: Option<Length>,
}
impl<'a, M: 'a> ChoiceGrid<'a, M> {
    pub fn new(columns: usize) -> Self {
        Self {
            items: Vec::new(),
            columns: columns.max(1),
            spacing: 4.0,
            width: None,
        }
    }
    pub fn push(mut self, choice: impl Into<Element<'a, M>>) -> Self {
        self.items.push(choice.into());
        self
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
    pub fn build(self) -> Column<'a, M, Theme, Renderer> {
        let mut body = Column::new().spacing(self.spacing);
        let mut items = self.items.into_iter().peekable();
        while items.peek().is_some() {
            let cells = items.by_ref().take(self.columns).collect::<Vec<_>>();
            body = body.push(iced_widget::row(cells).spacing(self.spacing));
        }
        if let Some(width) = self.width {
            body = body.width(width);
        }
        body
    }
}
impl<'a, M: 'a> From<ChoiceGrid<'a, M>> for Element<'a, M> {
    fn from(grid: ChoiceGrid<'a, M>) -> Self {
        grid.build().into()
    }
}

/// Two app-owned cells for properties, reports and summary tables.
pub struct KeyValueRow<'a, M> {
    inner: iced_widget::Row<'a, M, Theme, Renderer>,
}
impl<'a, M: 'a> KeyValueRow<'a, M> {
    pub fn new(key: impl Into<Element<'a, M>>, value: impl Into<Element<'a, M>>) -> Self {
        Self {
            inner: row![key.into(), value.into()],
        }
    }
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(spacing);
        self
    }
    pub fn align_y(mut self, align: impl Into<iced_core::alignment::Vertical>) -> Self {
        self.inner = self.inner.align_y(align);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn build(self) -> iced_widget::Row<'a, M, Theme, Renderer> {
        self.inner
    }
}
impl<'a, M: 'a> From<KeyValueRow<'a, M>> for Element<'a, M> {
    fn from(row: KeyValueRow<'a, M>) -> Self {
        row.inner.into()
    }
}
