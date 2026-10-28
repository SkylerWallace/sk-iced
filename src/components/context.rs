// SPDX-License-Identifier: Apache-2.0
//! Copyable, explicit per-view defaults; no global theme or motion state.
use super::{Appearance, Button, Checkbox, Segmented, TextInput};
use crate::animation::Element;
use crate::{
    design::{ControlProfile, DesignTokens},
    motion::Policy,
    typography::{TextRole, Typography},
};
use std::borrow::Cow;
/// Shared defaults can be customized per view without changing app-owned state.
///
/// ```
/// use iced_core::Font;
/// use sk_iced::{animation::Element, components::{Appearance, UiContext}, motion::Policy};
/// let ui = UiContext::new(
///     Appearance::fixed(Font::DEFAULT, sk_iced::palette::palette(true, false)),
///     Policy::DISABLED,
/// );
/// let button: Element<'_, u8> = ui.button("Save").on_press(1).enabled(true).build();
/// ```
#[derive(Clone, Copy)]
pub struct UiContext {
    pub appearance: Appearance,
    pub typography: Typography,
    pub design: DesignTokens,
    pub motion: Policy,
}
impl UiContext {
    pub fn new(appearance: Appearance, motion: Policy) -> Self {
        Self {
            appearance,
            typography: Typography::new(
                appearance.font,
                match appearance.palette {
                    super::Palette::Fixed(t) => {
                        t.assets.monospace.unwrap_or(iced_core::Font::MONOSPACE)
                    }
                    _ => iced_core::Font::MONOSPACE,
                },
            ),
            design: DesignTokens::default(),
            motion,
        }
    }
    pub fn with_typography(mut self, typography: Typography) -> Self {
        self.typography = typography;
        self.appearance.font = typography.body;
        self.design.text = typography.scale;
        self
    }
    pub fn with_design(mut self, design: DesignTokens) -> Self {
        self.design = design;
        self.typography.scale = design.text;
        self
    }
    pub fn text<'a>(
        self,
        role: TextRole,
        content: impl iced_widget::text::IntoFragment<'a>,
    ) -> iced_widget::Text<'a> {
        self.typography.text(role, content)
    }
    pub fn button<'a, M: Clone + 'a>(self, label: impl Into<Cow<'a, str>>) -> Button<'a, M> {
        self.button_with_profile(self.design.button, label)
    }
    pub fn button_with_profile<'a, M: Clone + 'a>(
        self,
        profile: ControlProfile,
        label: impl Into<Cow<'a, str>>,
    ) -> Button<'a, M> {
        Button::new(self.appearance, self.motion)
            .text(label)
            .height(profile.height)
            .text_size(profile.text_size)
            .padding([0.0, profile.horizontal_padding])
            .spacing(profile.icon_gap)
    }
    pub fn toggle_switch<'a, M: Clone + 'a>(self, checked: bool) -> super::ToggleSwitch<'a, M> {
        super::ToggleSwitch::new(self.appearance, self.motion, checked)
    }
    pub fn settings_card<'a, M: 'a>(
        self,
        title: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, M>>,
    ) -> super::SettingsCard<'a, M> {
        super::SettingsCard::new(self.appearance, title, control)
    }
    pub fn checkbox<'a, M: 'a>(self, checked: bool) -> Checkbox<'a, M> {
        Checkbox::new(self.appearance, checked)
            .size(self.design.checkbox_size)
            .spacing(self.design.checkbox_gap)
            .text_size(self.design.text.body)
    }
    pub fn text_input<'a, M: Clone + 'a>(self, placeholder: &str, value: &str) -> TextInput<'a, M> {
        TextInput::new(self.appearance, placeholder, value).text_size(self.design.text.body)
    }
    pub fn segmented<'a, M: Clone + 'a>(self) -> Segmented<'a, M> {
        Segmented::new(self.appearance, self.motion)
            .height(self.design.segment_height)
            .inset(self.design.segment_inset)
            .text_size(self.design.text.body)
    }
}
