// SPDX-License-Identifier: Apache-2.0
//! Shared input builders. Text editing keeps native state and operations.
//! Checkbox element conversion adds keyboard focus around the native control.
//! Native build methods remain available for specialized adapters.
use super::{Appearance, CheckState};
use crate::animation::Element;
use iced_core::{Font, Length, Padding, Pixels, Theme, text, widget};
use iced_widget::{checkbox, text_input};

pub struct Checkbox<'a, M> {
    inner: iced_widget::Checkbox<'a, M>,
    enabled: bool,
    appearance: Option<Appearance>,
    keyboard: bool,
    has_action: bool,
    id: Option<widget::Id>,
}
impl<'a, M: 'a> Checkbox<'a, M> {
    /// Defaults: app font/palette, 15-DIP box, 8-DIP gap, 13.333 text,
    /// content width. No callback means disabled. Selection stays app-owned.
    pub fn new(appearance: Appearance, checked: bool) -> Self {
        Self::with_state(
            appearance,
            if checked {
                CheckState::Checked
            } else {
                CheckState::Unchecked
            },
        )
    }
    pub fn with_state(appearance: Appearance, state: CheckState) -> Self {
        Self {
            inner: super::tri_state_checkbox(appearance.font, state),
            enabled: true,
            appearance: Some(appearance),
            keyboard: true,
            has_action: false,
            id: None,
        }
        .size(crate::design::size::CHECKBOX)
        .spacing(crate::design::space::MEDIUM)
        .text_size(crate::design::text::BODY)
        .style(move |theme, status| appearance.tokens(theme).check()(theme, status))
    }
    /// Preserve a specialized control's exact native defaults and styling.
    pub fn custom(inner: iced_widget::Checkbox<'a, M>) -> Self {
        Self {
            inner,
            enabled: true,
            appearance: None,
            keyboard: false,
            has_action: false,
            id: None,
        }
    }
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn keyboard(mut self, enabled: bool) -> Self {
        self.keyboard = enabled;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn with_appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }
    pub fn label(mut self, value: impl iced_widget::text::IntoFragment<'a>) -> Self {
        self.inner = self.inner.label(value);
        self
    }
    pub fn size(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(value);
        self
    }
    pub fn spacing(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.spacing(value);
        self
    }
    pub fn text_size(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.text_size(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn font(mut self, value: Font) -> Self {
        self.inner = self.inner.font(value);
        self
    }
    pub fn text_line_height(mut self, value: impl Into<text::LineHeight>) -> Self {
        self.inner = self.inner.text_line_height(value);
        self
    }
    pub fn icon(mut self, value: checkbox::Icon<Font>) -> Self {
        self.inner = self.inner.icon(value);
        self
    }
    pub fn on_toggle(self, action: impl Fn(bool) -> M + 'a) -> Self {
        self.on_toggle_maybe(Some(action))
    }
    pub fn on_toggle_maybe(mut self, action: Option<impl Fn(bool) -> M + 'a>) -> Self {
        self.has_action = action.is_some();
        self.inner = self.inner.on_toggle_maybe(action);
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme, checkbox::Status) -> checkbox::Style + 'a,
    ) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    /// Native adapter without shared keyboard decoration; prefer Element conversion in views.
    pub fn into_native(self) -> iced_widget::Checkbox<'a, M> {
        self.build()
    }
    pub fn build(self) -> iced_widget::Checkbox<'a, M> {
        if self.enabled {
            self.inner
        } else {
            self.inner.on_toggle_maybe(None::<fn(bool) -> M>)
        }
    }
}
impl<'a, M: 'a> From<Checkbox<'a, M>> for Element<'a, M> {
    fn from(spec: Checkbox<'a, M>) -> Self {
        let appearance = spec.appearance;
        let enabled = spec.enabled;
        let keyboard = spec.keyboard;
        let active = enabled && spec.has_action;
        let id = spec.id.clone();
        let inner = spec.build();
        let result = if let Some(appearance) = appearance {
            super::native_surface::decorate(
                inner,
                appearance,
                super::native_surface::Kind::Checkbox,
                enabled,
            )
        } else {
            inner.into()
        };
        if keyboard {
            let mut control = super::KeyboardControl::new(
                appearance.unwrap_or_else(|| {
                    Appearance::themed(iced_core::Font::DEFAULT, crate::palette::from_theme)
                }),
                result,
            )
            .enabled(active);
            if let Some(id) = id {
                control = control.id(id);
            }
            control.into()
        } else {
            result
        }
    }
}

pub struct TextInput<'a, M> {
    inner: iced_widget::TextInput<'a, M>,
    enabled: bool,
    appearance: Option<Appearance>,
}
impl<'a, M: Clone + 'a> TextInput<'a, M> {
    /// Uses the app font and themed input style with Iced's native sizing defaults.
    /// Value stays app-owned; editing requires an on_input callback. Disabling
    /// gates input, paste and submit together, regardless of setter order.
    pub fn new(appearance: Appearance, placeholder: &str, value: &str) -> Self {
        Self {
            inner: super::themed_text_input(appearance, placeholder, value),
            enabled: true,
            appearance: Some(appearance),
        }
    }
    /// Keep a specialized input's native styling and sizing without a new widget.
    pub fn custom(inner: iced_widget::TextInput<'a, M>) -> Self {
        Self {
            inner,
            enabled: true,
            appearance: None,
        }
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    /// Let a composite field own decoration while retaining native input state.
    pub fn without_decoration(mut self) -> Self {
        self.appearance = None;
        self
    }
    pub fn id(mut self, value: impl Into<widget::Id>) -> Self {
        self.inner = self.inner.id(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn text_size(self, value: impl Into<Pixels>) -> Self {
        self.size(value)
    }
    pub fn size(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(value);
        self
    }
    pub fn font(mut self, value: Font) -> Self {
        self.inner = self.inner.font(value);
        self
    }
    pub fn line_height(mut self, value: impl Into<text::LineHeight>) -> Self {
        self.inner = self.inner.line_height(value);
        self
    }
    pub fn secure(mut self, value: bool) -> Self {
        self.inner = self.inner.secure(value);
        self
    }
    pub fn icon(mut self, value: text_input::Icon<Font>) -> Self {
        self.inner = self.inner.icon(value);
        self
    }
    pub fn on_input(self, action: impl Fn(String) -> M + 'a) -> Self {
        self.on_input_maybe(Some(action))
    }
    pub fn on_input_maybe(mut self, action: Option<impl Fn(String) -> M + 'a>) -> Self {
        self.inner = self.inner.on_input_maybe(action);
        self
    }
    pub fn on_paste(self, action: impl Fn(String) -> M + 'a) -> Self {
        self.on_paste_maybe(Some(action))
    }
    pub fn on_paste_maybe(mut self, action: Option<impl Fn(String) -> M + 'a>) -> Self {
        self.inner = self.inner.on_paste_maybe(action);
        self
    }
    pub fn on_submit(mut self, action: M) -> Self {
        self.inner = self.inner.on_submit(action);
        self
    }
    pub fn on_submit_maybe(mut self, action: Option<M>) -> Self {
        self.inner = self.inner.on_submit_maybe(action);
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self {
        self.inner = self.inner.style(style);
        self
    }
    /// Native adapter retaining Iced editing state and operations.
    pub fn into_native(self) -> iced_widget::TextInput<'a, M> {
        self.build()
    }
    pub fn build(self) -> iced_widget::TextInput<'a, M> {
        if self.enabled {
            self.inner
        } else {
            self.inner
                .on_input_maybe(None::<fn(String) -> M>)
                .on_paste_maybe(None::<fn(String) -> M>)
                .on_submit_maybe(None)
        }
    }
}
impl<'a, M: Clone + 'a> From<TextInput<'a, M>> for Element<'a, M> {
    fn from(spec: TextInput<'a, M>) -> Self {
        let appearance = spec.appearance;
        let enabled = spec.enabled;
        let inner = spec.build();
        if let Some(appearance) = appearance {
            super::native_surface::decorate(
                inner,
                appearance,
                super::native_surface::Kind::Input,
                enabled,
            )
        } else {
            inner.into()
        }
    }
}
