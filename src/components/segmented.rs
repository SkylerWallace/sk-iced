// SPDX-License-Identifier: Apache-2.0
//! Equal-width, controlled choices built from the shared button implementation.
//! Selection belongs to the application: clicks emit messages without waiting for
//! animation. No action means noninteractive; enabled gates never invent actions.
use super::{Appearance, Button};
use crate::{animation::Element, motion::Policy, theme::color};
use iced_core::{Border, Length, Pixels};
use iced_widget::{button, container, row, tooltip};
use std::{borrow::Cow, time::Duration};

/// An individual choice. Labels and tooltip text can be borrowed or owned.
pub struct Segment<'a> {
    label: Cow<'a, str>,
    enabled: bool,
    tooltip: Option<Cow<'a, str>>,
}
impl<'a> Segment<'a> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            tooltip: None,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tooltip(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

/// Defaults: fill width, 28-DIP height, 2-DIP inset, 13.333 text, no selection
/// or action, enabled, and policy-controlled button feedback. Tooltips are opt-in,
/// appear below the choice after one second, and use 12-DIP text with 7-DIP padding.
/// Keep option order stable across rebuilds to retain each button's press state.
///
/// ```
/// use iced_core::Font;
/// use sk_iced::{animation::Element, components::{Appearance, Segmented}, motion::Policy};
/// let appearance = Appearance::fixed(Font::DEFAULT, sk_iced::palette::palette(true, false));
/// let control: Element<'_, u8> = Segmented::new(appearance, Policy::DISABLED)
///     .labels(["Light", "Dark", "Auto"])
///     .selected(1)
///     .width(198)
///     .on_select(|index| index as u8)
///     .build();
/// ```
pub struct Segmented<'a, M> {
    appearance: Appearance,
    policy: Policy,
    options: Vec<Segment<'a>>,
    selected: Option<usize>,
    choose: Option<Box<dyn Fn(usize) -> Option<M> + 'a>>,
    enabled: bool,
    width: Length,
    height: f32,
    inset: f32,
    text_size: f32,
    feedback: bool,
    tooltip_position: tooltip::Position,
    tooltip_delay: Duration,
    id: Option<iced_core::widget::Id>,
}
impl<'a, M: Clone + 'a> Segmented<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(appearance: Appearance, policy: Policy) -> Self {
        Self {
            appearance,
            policy,
            options: Vec::new(),
            selected: None,
            choose: None,
            enabled: true,
            width: Length::Fill,
            height: crate::design::size::SEGMENT,
            inset: crate::design::size::SEGMENT_INSET,
            text_size: crate::design::text::BODY,
            feedback: true,
            tooltip_position: tooltip::Position::Bottom,
            tooltip_delay: Duration::from_secs(1),
            id: None,
        }
    }
    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn option(mut self, option: Segment<'a>) -> Self {
        self.options.push(option);
        self
    }
    pub fn options(mut self, options: impl IntoIterator<Item = Segment<'a>>) -> Self {
        self.options.extend(options);
        self
    }
    pub fn labels(mut self, labels: impl IntoIterator<Item = impl Into<Cow<'a, str>>>) -> Self {
        self.options.extend(labels.into_iter().map(Segment::new));
        self
    }
    pub fn selected(self, index: usize) -> Self {
        self.selection(Some(index))
    }
    /// None or an out-of-range index paints no selected choice.
    pub fn selection(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }
    pub fn on_select(self, choose: impl Fn(usize) -> M + 'a) -> Self {
        self.on_select_maybe(move |index| Some(choose(index)))
    }
    /// Returning None disables that choice, independently of its selected state.
    pub fn on_select_maybe(mut self, choose: impl Fn(usize) -> Option<M> + 'a) -> Self {
        self.choose = Some(Box::new(choose));
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Fixed total height, including the inset around each choice.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(0.0);
        self
    }
    pub fn inset(mut self, inset: f32) -> Self {
        self.inset = inset.max(0.0);
        self
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into().0;
        self
    }
    /// Disable feedback for app profiles that previously used plain buttons.
    pub fn feedback(mut self, feedback: bool) -> Self {
        self.feedback = feedback;
        self
    }
    pub fn tooltip_position(mut self, position: tooltip::Position) -> Self {
        self.tooltip_position = position;
        self
    }
    pub fn tooltip_delay(mut self, delay: Duration) -> Self {
        self.tooltip_delay = delay;
        self
    }
}
impl<'a, M: Clone + 'a> From<Segmented<'a, M>> for Element<'a, M> {
    fn from(spec: Segmented<'a, M>) -> Self {
        let appearance = spec.appearance;
        let mut parts = row![].spacing(0);
        let inset = spec.inset.min(spec.height / 2.0);
        let mut actions = Vec::new();
        for (index, option) in spec.options.into_iter().enumerate() {
            let selected = spec.selected == Some(index);
            let message = if spec.enabled && option.enabled {
                spec.choose.as_ref().and_then(|choose| choose(index))
            } else {
                None
            };
            actions.push(message.clone());
            let inner = button(
                container(super::text(appearance.font, option.label).size(spec.text_size))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .height((spec.height - 2.0 * inset).max(0.0))
            .padding(0)
            .width(Length::Fill)
            .style(move |theme, status| {
                appearance.tokens(theme).segment(selected, false)(theme, status)
            });
            let part = container(
                Button::custom(appearance, spec.policy, inner)
                    .on_press_maybe(message)
                    .surface_role(super::SurfaceRole::Segment(selected))
                    .enabled(spec.enabled && option.enabled)
                    .feedback(spec.feedback),
            )
            .padding(inset)
            .width(Length::Fill);
            let part: Element<'a, M> = if let Some(hint) = option.tooltip {
                super::Tooltip::new(appearance, part, hint)
                    .position(spec.tooltip_position)
                    .delay(spec.tooltip_delay)
                    .into()
            } else {
                part.into()
            };
            parts = parts.push(part);
        }
        let inner: Element<'a, M> = container(parts)
            .height(spec.height)
            .width(spec.width)
            .style(move |theme| {
                let t = appearance.tokens(theme);
                t.visual
                    .segment_group
                    .unwrap_or(t.visual.input.normal)
                    .container_style(container::Style {
                        background: Some(color(t.background).into()),
                        border: Border {
                            color: color(t.border),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        ..Default::default()
                    })
            })
            .into();
        let enabled = actions.iter().any(Option::is_some);
        let available: Vec<_> = actions
            .iter()
            .enumerate()
            .filter_map(|(i, a)| a.as_ref().map(|_| i))
            .collect();

        let mut control = super::KeyboardControl::new(appearance, inner)
            .enabled(enabled)
            .activation(false)
            .selection_keys()
            .on_key(move |key| {
                use iced_core::keyboard::key::Named;
                let current = spec
                    .selected
                    .and_then(|i| available.iter().position(|v| *v == i));
                let target = match key {
                    Named::ArrowRight | Named::ArrowDown => {
                        current.map(|i| (i + 1) % available.len()).unwrap_or(0)
                    }
                    Named::ArrowLeft | Named::ArrowUp => current
                        .map(|i| (i + available.len() - 1) % available.len())
                        .unwrap_or(available.len().saturating_sub(1)),
                    Named::Home => 0,
                    Named::End => available.len().saturating_sub(1),
                    Named::Space | Named::Enter => current.unwrap_or(0),
                    _ => return None,
                };
                available.get(target).and_then(|i| actions[*i].clone())
            });
        if let Some(id) = spec.id {
            control = control.id(id);
        }
        control.into()
    }
}
