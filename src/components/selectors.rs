// SPDX-License-Identifier: Apache-2.0
//! Native fixed-list and searchable dropdown builders. Search state belongs to
//! the caller, so filtering and selection survive view rebuilds.
use super::Appearance;
use crate::animation::Element;
use iced_core::{Font, Length, Padding, Pixels, Theme};
use iced_widget::{combo_box, overlay::menu, pick_list, text_input};
pub struct Dropdown<'a, T: ToString + PartialEq + Clone, M> {
    inner: iced_widget::PickList<'a, T, Vec<T>, T, M>,
    enabled: Option<bool>,
    appearance: Option<Appearance>,
    keys: Option<Box<dyn Fn(iced_core::keyboard::key::Named) -> Option<M> + 'a>>,
    id: Option<iced_core::widget::Id>,
}
impl<'a, T: ToString + PartialEq + Clone + 'a, M: Clone + 'a> Dropdown<'a, T, M> {
    pub fn new(
        appearance: Appearance,
        options: Vec<T>,
        selected: Option<T>,
        action: impl Fn(T) -> M + 'a,
    ) -> Self {
        let key_options = options.clone();
        let key_selected = selected.clone();
        let action: std::rc::Rc<dyn Fn(T) -> M + 'a> = std::rc::Rc::new(action);
        let pointer_action = action.clone();
        let mut result = Self::custom(iced_widget::pick_list(options, selected, move |value| {
            pointer_action(value)
        }))
        .font(appearance.font)
        .text_size(crate::design::text::BODY)
        .style(move |theme, status| {
            crate::visual::dropdown_style(
                appearance.tokens(theme),
                pick_list::default(theme, status),
            )
        })
        .menu_style(move |theme| {
            crate::visual::menu_style(appearance.tokens(theme), menu::default(theme))
        });
        result.appearance = Some(appearance);
        result.keys = Some(Box::new(move |key| {
            use iced_core::keyboard::key::Named;
            let current = key_selected
                .as_ref()
                .and_then(|v| key_options.iter().position(|o| o == v));
            let index = match key {
                Named::ArrowDown | Named::ArrowRight => current
                    .map(|i| (i + 1).min(key_options.len().saturating_sub(1)))
                    .unwrap_or(0),
                Named::ArrowUp | Named::ArrowLeft => current
                    .map(|i| i.saturating_sub(1))
                    .unwrap_or(key_options.len().saturating_sub(1)),
                Named::Home => 0,
                Named::End => key_options.len().saturating_sub(1),
                _ => return None,
            };
            key_options
                .get(index)
                .filter(|v| Some(*v) != key_selected.as_ref())
                .map(|v| action(v.clone()))
        }));
        result
    }
    pub fn custom(inner: iced_widget::PickList<'a, T, Vec<T>, T, M>) -> Self {
        Self {
            inner,
            enabled: None,
            appearance: None,
            keys: None,
            id: None,
        }
    }
    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.inner = self.inner.placeholder(value);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn menu_height(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.menu_height(value);
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn font(mut self, value: Font) -> Self {
        self.inner = self.inner.font(value);
        self
    }
    pub fn text_size(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.text_size(value);
        self
    }
    pub fn handle(mut self, value: pick_list::Handle<Font>) -> Self {
        self.inner = self.inner.handle(value);
        self
    }
    pub fn on_open(mut self, value: M) -> Self {
        self.inner = self.inner.on_open(value);
        self
    }
    pub fn on_close(mut self, value: M) -> Self {
        self.inner = self.inner.on_close(value);
        self
    }
    pub fn style(
        mut self,
        value: impl Fn(&Theme, pick_list::Status) -> pick_list::Style + 'a,
    ) -> Self {
        self.inner = self.inner.style(value);
        self
    }
    pub fn menu_style(mut self, value: impl Fn(&Theme) -> menu::Style + 'a) -> Self {
        self.inner = self.inner.menu_style(value);
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let inner: Element<'a, M> = self.inner.into();
        let inner = if let Some(appearance) = self.appearance {
            super::Surface::new(appearance, inner)
                .role(super::SurfaceRole::Dropdown)
                .enabled(self.enabled.unwrap_or(true))
                .decorate()
                .into()
        } else {
            inner
        };
        let inner = if let Some(enabled) = self.enabled {
            super::InteractionGate::new(inner, enabled).into()
        } else {
            inner
        };
        if let Some(keys) = self.keys {
            let appearance = self.appearance.expect("standard dropdown appearance");
            let mut control = super::KeyboardControl::new(appearance, inner)
                .enabled(self.enabled.unwrap_or(true))
                .on_key(keys)
                .selection_keys()
                .close_on_key();
            if let Some(id) = self.id {
                control = control.id(id);
            }
            control.into()
        } else {
            inner
        }
    }
}
impl<'a, T: ToString + PartialEq + Clone + 'a, M: Clone + 'a> From<Dropdown<'a, T, M>>
    for Element<'a, M>
{
    fn from(spec: Dropdown<'a, T, M>) -> Self {
        spec.build()
    }
}
pub fn dropdown<'a, T: ToString + PartialEq + Clone + 'a, M: Clone + 'a>(
    options: Vec<T>,
    selected: Option<T>,
    action: impl Fn(T) -> M + 'a,
) -> Dropdown<'a, T, M> {
    Dropdown::custom(iced_widget::pick_list(options, selected, action))
}
pub struct SearchableDropdown<'a, T: std::fmt::Display + Clone, M> {
    inner: iced_widget::ComboBox<'a, T, M>,
    enabled: Option<bool>,
    appearance: Appearance,
}
impl<'a, T: std::fmt::Display + Clone + 'static, M: Clone + 'static> SearchableDropdown<'a, T, M> {
    pub fn new(
        appearance: Appearance,
        state: &'a combo_box::State<T>,
        placeholder: &str,
        selected: Option<&T>,
        action: impl Fn(T) -> M + 'static,
    ) -> Self {
        Self {
            inner: iced_widget::ComboBox::new(state, placeholder, selected, action)
                .font(appearance.font)
                .size(crate::design::text::BODY)
                .input_style(move |theme, status| {
                    crate::visual::input_style(
                        {
                            let mut t = appearance.tokens(theme);
                            t.visual.input = t.visual.dropdown.unwrap_or(t.visual.input);
                            t
                        },
                        text_input::default(theme, status),
                        status,
                    )
                })
                .menu_style(move |theme| {
                    crate::visual::menu_style(appearance.tokens(theme), menu::default(theme))
                }),
            enabled: None,
            appearance,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.width(value);
        self
    }
    pub fn menu_height(mut self, value: impl Into<Length>) -> Self {
        self.inner = self.inner.menu_height(value);
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(value);
        self
    }
    pub fn text_size(self, value: impl Into<Pixels>) -> Self {
        self.size(value)
    }
    pub fn build(self) -> Element<'a, M> {
        let inner: Element<'a, M> = super::Surface::new(self.appearance, self.inner)
            .role(super::SurfaceRole::Dropdown)
            .enabled(self.enabled.unwrap_or(true))
            .decorate()
            .into();
        if let Some(enabled) = self.enabled {
            super::InteractionGate::new(inner, enabled).into()
        } else {
            inner
        }
    }
    pub fn size(mut self, value: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(value);
        self
    }
    pub fn on_input(mut self, value: impl Fn(String) -> M + 'static) -> Self {
        self.inner = self.inner.on_input(value);
        self
    }
    pub fn on_open(mut self, value: M) -> Self {
        self.inner = self.inner.on_open(value);
        self
    }
    pub fn on_close(mut self, value: M) -> Self {
        self.inner = self.inner.on_close(value);
        self
    }
    pub fn input_style(
        mut self,
        value: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self {
        self.inner = self.inner.input_style(value);
        self
    }
    pub fn menu_style(mut self, value: impl Fn(&Theme) -> menu::Style + 'a) -> Self {
        self.inner = self.inner.menu_style(value);
        self
    }
}
impl<'a, T: std::fmt::Display + Clone + 'static, M: Clone + 'static>
    From<SearchableDropdown<'a, T, M>> for Element<'a, M>
{
    fn from(spec: SearchableDropdown<'a, T, M>) -> Self {
        spec.build()
    }
}
