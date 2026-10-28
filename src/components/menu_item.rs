// SPDX-License-Identifier: Apache-2.0
//! Standard menu-row composition using the shared Button interaction path.
use super::{Appearance, Button};
use crate::{animation::Element, motion::Policy, theme::color};
use iced_core::{Length, Padding, Pixels, Theme};
use iced_widget::{Space, button, row};
use std::borrow::Cow;
type ItemStyle<'a> = Box<dyn Fn(&Theme, button::Status) -> button::Style + 'a>;
pub struct MenuItem<'a, M> {
    appearance: Appearance,
    policy: Policy,
    label: Cow<'a, str>,
    icon: Option<Element<'a, M>>,
    check: Option<Element<'a, M>>,
    shortcut: Option<Cow<'a, str>>,
    action: Option<M>,
    enabled: bool,
    feedback: bool,
    padding: Padding,
    height: Option<Length>,
    size: f32,
    style: Option<ItemStyle<'a>>,
}
impl<'a, M: Clone + 'a> MenuItem<'a, M> {
    pub fn new(appearance: Appearance, policy: Policy, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            appearance,
            policy,
            label: label.into(),
            icon: None,
            check: None,
            shortcut: None,
            action: None,
            enabled: true,
            feedback: true,
            padding: [8, 10].into(),
            height: None,
            size: 13.0,
            style: None,
        }
    }
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn check(mut self, check: impl Into<Element<'a, M>>) -> Self {
        self.check = Some(check.into());
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.check = Some(
            super::text(self.appearance.font, if checked { "✓" } else { "" })
                .size(13)
                .width(16)
                .into(),
        );
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    pub fn on_press(mut self, action: M) -> Self {
        self.action = Some(action);
        self
    }
    pub fn on_press_maybe(mut self, action: Option<M>) -> Self {
        self.action = action;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn feedback(mut self, feedback: bool) -> Self {
        self.feedback = feedback;
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.style = Some(Box::new(style));
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let appearance = self.appearance;
        let mut content = row![];
        if let Some(check) = self.check {
            content = content.push(check);
        }
        if let Some(icon) = self.icon {
            content = content.push(icon);
        }
        content = content.push(super::text(appearance.font, self.label).size(self.size));
        if let Some(shortcut) = self.shortcut {
            content = content.push(Space::new().width(Length::Fill)).push(
                super::text(appearance.font, shortcut)
                    .size(11)
                    .style(move |theme: &Theme| iced_widget::text::Style {
                        color: Some(color(appearance.tokens(theme).secondary)),
                    }),
            );
        }
        let raw = iced_widget::button(content.spacing(8).align_y(iced_core::Alignment::Center))
            .padding(self.padding)
            .width(Length::Fill);
        let mut button = Button::custom(appearance, self.policy, raw)
            .on_press_maybe(self.action)
            .enabled(self.enabled)
            .feedback(self.feedback);
        if let Some(height) = self.height {
            button = button.height(height);
        }
        if let Some(style) = self.style {
            button = button.style(style);
        } else {
            button = button
                .style(move |theme, status| appearance.tokens(theme).flat_dimmed()(theme, status));
        }
        button.build()
    }
}
impl<'a, M: Clone + 'a> From<MenuItem<'a, M>> for Element<'a, M> {
    fn from(item: MenuItem<'a, M>) -> Self {
        item.build()
    }
}
