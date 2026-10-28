// SPDX-License-Identifier: Apache-2.0
use super::{Appearance, Button, Navigation, Panel, Popup};
use crate::{animation::Element, motion::Policy};
use iced_core::{Color, Length};
use iced_widget::{opaque, stack};

/// Menu body; use Popup for anchoring, outside-click/Escape dismissal and sizing.
type MenuStyle<'a> = Box<dyn Fn(&iced_core::Theme) -> iced_widget::container::Style + 'a>;
pub struct Menu<'a, M> {
    appearance: Appearance,
    policy: Policy,
    items: Navigation<'a, M>,
    body: Option<Element<'a, M>>,
    padding: iced_core::Padding,
    width: Length,
    clip: bool,
    style: Option<MenuStyle<'a>>,
}
impl<'a, M: Clone + 'a> Menu<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    pub fn new(appearance: Appearance, policy: Policy) -> Self {
        Self {
            appearance,
            policy,
            items: Navigation::vertical(),
            body: None,
            padding: 1.into(),
            width: Length::Fill,
            clip: true,
            style: None,
        }
    }
    pub fn content(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.body = Some(content.into());
        self
    }
    pub fn padding(mut self, padding: impl Into<iced_core::Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&iced_core::Theme) -> iced_widget::container::Style + 'a,
    ) -> Self {
        self.style = Some(Box::new(style));
        self
    }
    pub fn item(self, label: impl Into<std::borrow::Cow<'a, str>>, action: Option<M>) -> Self {
        let control = Button::new(self.appearance, self.policy)
            .text(label)
            .width(Length::Fill)
            .on_press_maybe(action);
        self.push(control)
    }
    pub fn push(mut self, content: impl Into<Element<'a, M>>) -> Self {
        if let Some(body) = self.body.take() {
            self.items = self.items.push(body);
        }
        self.items = self.items.push(content);
        self
    }
    pub fn separator(self) -> Self {
        self.push(iced_widget::rule::horizontal(1))
    }
    pub fn spacing(mut self, value: impl Into<iced_core::Pixels>) -> Self {
        self.items = self.items.spacing(value);
        self
    }
}
impl<'a, M: Clone + 'a> From<Menu<'a, M>> for Element<'a, M> {
    fn from(spec: Menu<'a, M>) -> Self {
        let body = spec.body.unwrap_or_else(|| spec.items.into());
        let mut panel = Panel::new(spec.appearance, body)
            .padding(spec.padding)
            .width(spec.width)
            .clip(spec.clip);
        let appearance = spec.appearance;
        if let Some(style) = spec.style {
            panel = panel.style(style);
        } else {
            panel = panel.style(move |theme| {
                let t = appearance.tokens(theme);
                t.visual.menu.container_style(t.card(false))
            });
        }
        super::Surface::new(appearance, panel.build())
            .role(super::SurfaceRole::Menu)
            .decorate()
            .into()
    }
}

/// Centered modal popup with an optional dismiss action. Specialized app dialogs
/// can retain their exact stack/backdrop layout via custom without new widgets.
pub struct Dialog<'a, M> {
    popup: Popup<'a, M>,
    commands: Option<super::DialogCommands<M>>,
}
impl<'a, M: Clone + 'a> Dialog<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    /// Keep this shell mounted while open changes to retain paint-only exits.
    pub fn new(base: impl Into<Element<'a, M>>, card: impl Into<Element<'a, M>>) -> Self {
        Self {
            popup: Popup::new(base, card)
                .open(true)
                .centered(true)
                .capture_outside(true)
                .max_width(f32::MAX)
                .scrim(Color::from_rgba(0.0, 0.0, 0.0, 0.55)),
            commands: None,
        }
    }
    /// Native-layout adapter for dialogs whose app already owns dismissal/focus
    /// and backdrop geometry. Returns the original stack directly.
    pub fn custom(
        base: impl Into<Element<'a, M>>,
        backdrop: impl Into<Element<'a, M>>,
    ) -> Element<'a, M> {
        stack![base.into(), opaque(backdrop.into())].into()
    }
    pub fn commands(mut self, commands: super::DialogCommands<M>) -> Self {
        self.commands = Some(commands);
        self
    }
    pub fn on_dismiss(mut self, action: M) -> Self {
        self.popup = self.popup.on_dismiss(action);
        self
    }
    pub fn policy(mut self, policy: Policy) -> Self {
        self.popup = self.popup.policy(policy);
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.popup = self.popup.open(open);
        self
    }
    pub fn max_width(mut self, width: f32) -> Self {
        self.popup = self.popup.max_width(width);
        self
    }
}
impl<'a, M: Clone + 'a> From<Dialog<'a, M>> for Element<'a, M> {
    fn from(spec: Dialog<'a, M>) -> Self {
        if let Some(commands) = spec.commands {
            spec.popup.on_key(move |key| commands.action(key)).into()
        } else {
            spec.popup.into()
        }
    }
}
