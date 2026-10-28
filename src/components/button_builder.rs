// SPDX-License-Identifier: Apache-2.0
//! The shared button builder. Enabled state is independent of tone and border.
//!
//! Typical construction: `Button::new(appearance, policy).icon(icon).text("Save")`
//! followed by `.on_press(message).tone(ButtonTone::Important)` and conversion
//! into an Element. Width and height accept Iced lengths; omitted values use
//! 32-DIP height. Icon-only buttons default to 32-DIP width; text fits its content.
//! Icons always precede text.
//! Warning and Danger both use the red failure palette role.
//! Use app-owned presets for consistent font, sizing and appearance.
//! Custom controls can retain their original Iced button with Button::custom.
//! Supplying a tone or border replaces its inherited style; supply .style(...)
//! as well to retain a custom style while removing its border. Replacing custom
//! content requires rebinding the action. Motion policy only controls feedback;
//! actions are delivered by Iced immediately, including when motion is disabled.
use super::{ActionLayout, Appearance, ButtonLayout};
use crate::{
    animation::{
        Element,
        feedback::{self, RefreshIcon},
    },
    motion::Policy,
};
use iced_core::{Length, Padding, Theme};
use iced_widget::{button, container};
use std::borrow::Cow;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonTone {
    #[default]
    Default,
    Important,
    Warning,
    Danger,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonBorder {
    #[default]
    Standard,
    None,
}
enum Binding<M> {
    Inherited,
    Message(Option<M>),
}
type Style<'a> = Box<dyn Fn(&Theme, button::Status) -> button::Style + 'a>;
pub struct Button<'a, M> {
    appearance: Appearance,
    policy: Policy,
    raw: Option<iced_widget::Button<'a, M>>,
    content: Option<Element<'a, M>>,
    icon: Option<Element<'a, M>>,
    text: Option<Cow<'a, str>>,
    action: Binding<M>,
    enabled: bool,
    selected: bool,
    tone: Option<ButtonTone>,
    border: Option<ButtonBorder>,
    width: Option<Length>,
    height: Option<Length>,
    padding: Option<Padding>,
    center_x: Option<Length>,
    text_size: f32,
    wrapping: iced_core::text::Wrapping,
    spacing: f32,
    style: Option<Style<'a>>,
    refresh: RefreshIcon,
    feedback: bool,
    keyboard: bool,
    id: Option<iced_core::widget::Id>,
    surface_role: Option<super::SurfaceRole>,
}
impl<'a, M: Clone + 'a> Button<'a, M> {
    pub fn build(self) -> Element<'a, M> {
        self.into()
    }
    /// Defaults: 32-DIP height, square icon-only buttons, content-sized text buttons,
    /// 13.333 text, five-DIP icon gap, enabled gate, neutral tone, standard border.
    /// Standard content is centered horizontally and vertically. Custom native buttons retain their layout.
    /// A button without an action is noninteractive regardless of the enabled gate.
    pub fn new(appearance: Appearance, policy: Policy) -> Self {
        Self {
            appearance,
            policy,
            raw: None,
            content: None,
            icon: None,
            text: None,
            action: Binding::Message(None),
            enabled: true,
            selected: false,
            tone: Some(ButtonTone::Default),
            border: Some(ButtonBorder::Standard),
            width: None,
            height: Some(crate::design::size::BUTTON.into()),
            padding: None,
            center_x: None,
            text_size: crate::design::text::BODY,
            wrapping: iced_core::text::Wrapping::Word,
            spacing: crate::design::ControlProfile::STANDARD.icon_gap,
            style: None,
            refresh: RefreshIcon::None,
            feedback: true,
            keyboard: true,
            id: None,
            surface_role: None,
        }
    }
    /// Wrap a native button while preserving its Iced
    /// content, geometry and style; action, enabled and visual options can override them.
    pub fn custom(appearance: Appearance, policy: Policy, raw: iced_widget::Button<'a, M>) -> Self {
        Self {
            raw: Some(raw),
            keyboard: false,
            action: Binding::Inherited,
            tone: None,
            border: None,
            width: None,
            height: None,
            padding: None,
            ..Self::new(appearance, policy)
        }
    }
    fn standard_content(&mut self) {
        if self.raw.take().is_some() {
            self.action = Binding::Message(None);
        }
        self.content = None;
    }
    pub fn icon(mut self, icon: impl Into<Element<'a, M>>) -> Self {
        self.standard_content();
        self.icon = Some(icon.into());
        self
    }
    pub fn text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.standard_content();
        self.text = Some(text.into());
        self
    }
    /// Custom content replaces the standard icon/text row, without wrapping it in another row.
    pub fn content(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.standard_content();
        self.icon = None;
        self.text = None;
        self.content = Some(content.into());
        self
    }
    pub fn on_press(mut self, message: M) -> Self {
        self.action = Binding::Message(Some(message));
        self
    }
    pub fn on_press_maybe(mut self, message: Option<M>) -> Self {
        self.action = Binding::Message(message);
        self
    }
    /// Persistent on/off state, independent of tone and enabled state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tone(mut self, tone: ButtonTone) -> Self {
        self.tone = Some(tone);
        self
    }
    pub fn border(mut self, border: ButtonBorder) -> Self {
        self.border = Some(border);
        self
    }
    pub fn borderless(self) -> Self {
        self.border(ButtonBorder::None)
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    pub fn center_x(mut self, width: Length) -> Self {
        self.center_x = Some(width);
        self
    }
    pub fn text_size(mut self, size: impl Into<iced_core::Pixels>) -> Self {
        self.text_size = size.into().0;
        self
    }
    pub fn wrapping(mut self, wrapping: iced_core::text::Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }
    pub fn spacing(mut self, spacing: impl Into<iced_core::Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }
    pub fn with_refresh(mut self, refresh: RefreshIcon) -> Self {
        self.refresh = refresh;
        self
    }
    /// Explicit visual elevation for specialized controls. Custom native styles are otherwise undecorated.
    pub fn surface_role(mut self, role: super::SurfaceRole) -> Self {
        self.surface_role = Some(role);
        self
    }
    /// Native adapters opt in explicitly; pass the actual enabled state as well.
    pub fn keyboard(mut self, enabled: bool) -> Self {
        self.keyboard = enabled;
        self
    }
    pub fn id(mut self, id: impl Into<iced_core::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    /// Let specialized controls retain their existing feedback behavior.
    pub fn feedback(mut self, feedback: bool) -> Self {
        self.feedback = feedback;
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.style = Some(Box::new(style));
        self
    }
    pub fn layout(mut self, layout: ButtonLayout) -> Self {
        self.width = Some(layout.width);
        self.height = Some(layout.height.into());
        self.padding = Some(layout.padding);
        self.center_x = layout.center_x;
        self
    }
    pub fn action_layout(mut self, layout: ActionLayout) -> Self {
        self = self.layout(layout.button);
        self.text_size = layout.text_size;
        self.wrapping = layout.wrapping;
        self
    }
}
impl<'a, M: Clone + 'a> From<Button<'a, M>> for Element<'a, M> {
    fn from(spec: Button<'a, M>) -> Self {
        let appearance = spec.appearance;
        let native_custom = spec.raw.is_some() || spec.style.is_some();
        let icon_only = spec.content.is_none() && spec.text.is_none();
        let width = spec.width.or_else(|| {
            spec.raw.is_none().then_some(if icon_only {
                Length::Fixed(crate::design::size::BUTTON)
            } else {
                Length::Shrink
            })
        });
        let padding = spec.padding.or_else(|| {
            spec.raw.is_none().then_some(if icon_only {
                0.into()
            } else {
                [0., crate::design::space::CONTROL].into()
            })
        });
        let mut inner = if let Some(raw) = spec.raw {
            raw
        } else {
            let content = if let Some(content) = spec.content {
                content
            } else if let Some(icon) = spec.icon {
                super::icon_label(
                    appearance.font,
                    icon,
                    spec.text,
                    spec.text_size,
                    spec.spacing,
                )
                .into()
            } else if let Some(label) = spec.text {
                super::text(appearance.font, label)
                    .size(spec.text_size)
                    .wrapping(spec.wrapping)
                    .into()
            } else {
                iced_widget::Space::new().into()
            };
            let content =
                container(content)
                    .center_y(Length::Fill)
                    .center_x(spec.center_x.unwrap_or(match width {
                        None | Some(Length::Shrink) => Length::Shrink,
                        _ => Length::Fill,
                    }));
            button(content)
        };
        if let Some(width) = width {
            inner = inner.width(width);
        }
        if let Some(height) = spec.height {
            inner = inner.height(height);
        }
        if let Some(padding) = padding {
            inner = inner.padding(padding);
        }
        if spec.style.is_some() || spec.tone.is_some() || spec.border.is_some() {
            inner = inner.style(move |theme, status| {
                let mut style = if let Some(style) = &spec.style {
                    style(theme, status)
                } else {
                    let t = appearance.tokens(theme);
                    match spec.tone.unwrap_or_default() {
                        ButtonTone::Default => t.toggle_button(spec.selected)(theme, status),
                        ButtonTone::Important => t.button(true, false)(theme, status),
                        ButtonTone::Warning | ButtonTone::Danger => {
                            t.button(false, true)(theme, status)
                        }
                    }
                };
                if spec.border == Some(ButtonBorder::None) {
                    style.border.width = 0.0;
                }
                style
            });
        }
        let has_action = match spec.action {
            Binding::Message(message) => {
                let active = message.is_some();
                inner = inner.on_press_maybe(if spec.enabled { message } else { None });
                active
            }
            Binding::Inherited => {
                if !spec.enabled {
                    inner = inner.on_press_maybe(None);
                }
                true
            }
        };
        let role = if spec.border == Some(ButtonBorder::None) {
            super::SurfaceRole::Flat
        } else {
            match spec.tone.unwrap_or_default() {
                ButtonTone::Important => super::SurfaceRole::Important,
                ButtonTone::Danger | ButtonTone::Warning => super::SurfaceRole::Danger,
                _ => super::SurfaceRole::Button,
            }
        };
        let role = spec.surface_role.unwrap_or(if spec.selected {
            super::SurfaceRole::Toggle(true)
        } else if native_custom {
            super::SurfaceRole::Custom(Default::default())
        } else {
            role
        });
        let inner = super::Surface::new(appearance, inner)
            .role(role)
            .enabled(spec.enabled && has_action)
            .decorate();
        let inner = if spec.feedback {
            feedback::wrap(inner, spec.policy, spec.refresh, spec.enabled && has_action)
        } else {
            inner.into()
        };
        if spec.keyboard {
            let mut control =
                super::KeyboardControl::new(appearance, inner).enabled(spec.enabled && has_action);
            if let Some(id) = spec.id {
                control = control.id(id);
            }
            control.into()
        } else {
            inner
        }
    }
}

#[cfg(test)]
mod builder_tests {
    use super::*;
    use iced_core::Font;
    fn appearance() -> Appearance {
        Appearance::fixed(Font::DEFAULT, crate::palette::palette(true, false))
    }
    #[test]
    fn automatic_icon_buttons_are_square_and_text_buttons_fit_their_labels() {
        use iced_core::{Size, layout, widget::Tree};
        let renderer = crate::animation::Renderer::new(Font::DEFAULT, 13.0.into());
        for label in [None, Some("A longer action label")] {
            let spec = Button::new(appearance(), Policy::DISABLED).on_press(());
            let mut widget = if let Some(label) = label {
                spec.text(label).build()
            } else {
                spec.icon(iced_widget::Space::new().width(16).height(16))
                    .build()
            };
            let mut tree = Tree::new(&widget);
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400., 100.)),
            );
            assert_eq!(node.size().height, 32.);
            if label.is_some() {
                assert!(node.size().width > 32.);
            } else {
                assert_eq!(node.size().width, 32.);
            }
        }
    }
    #[test]
    fn standard_fixed_button_centers_content_on_both_axes() {
        use iced_core::{Point, Size, layout, widget::Tree};
        let mut button = Button::new(appearance(), Policy::DISABLED)
            .icon(iced_widget::Space::new().width(16).height(16))
            .width(80)
            .height(40)
            .padding(0)
            .on_press(())
            .build();
        let mut tree = Tree::new(&button);
        let renderer = crate::animation::Renderer::new(Font::DEFAULT, 13.0.into());
        let node = button.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(100., 100.)),
        );
        fn visit(n: &layout::Node, origin: Point, found: &mut Vec<Point>) {
            let b = n.bounds();
            let origin = Point::new(origin.x + b.x, origin.y + b.y);
            if b.size() == Size::new(16., 16.) {
                found.push(Point::new(origin.x + 8., origin.y + 8.));
            }
            for c in n.children() {
                visit(c, origin, found);
            }
        }
        let mut centers = vec![];
        visit(&node, Point::ORIGIN, &mut centers);
        assert!(!centers.is_empty());
        assert!(
            centers.iter().all(|p| *p == Point::new(40., 20.)),
            "{centers:?}"
        );
    }
    #[test]
    fn defaults_and_options_are_independent() {
        let spec = Button::<u8>::new(appearance(), Policy::DISABLED);
        assert_eq!(spec.height, Some(Length::Fixed(32.0)));
        assert_eq!(spec.width, None);
        assert_eq!(spec.tone, Some(ButtonTone::Default));
        assert_eq!(spec.border, Some(ButtonBorder::Standard));
        assert!(spec.enabled);
        assert!(matches!(spec.action, Binding::Message(None)));
        let spec = spec
            .enabled(false)
            .on_press(7)
            .tone(ButtonTone::Warning)
            .borderless()
            .width(120)
            .height(28);
        assert!(!spec.enabled);
        assert!(matches!(spec.action, Binding::Message(Some(7))));
        assert_eq!(spec.tone, Some(ButtonTone::Warning));
        assert_eq!(spec.border, Some(ButtonBorder::None));
        assert_eq!(spec.width, Some(Length::Fixed(120.0)));
        let spec = spec.enabled(true).on_press_maybe(None);
        assert!(spec.enabled);
        assert!(matches!(spec.action, Binding::Message(None)));
    }
    #[test]
    fn icon_and_text_survive_either_setter_order() {
        let spec = Button::<u8>::new(appearance(), Policy::DISABLED)
            .text("Save")
            .icon(iced_widget::Space::new().width(16));
        assert_eq!(spec.text.as_deref(), Some("Save"));
        assert!(spec.icon.is_some());
        let spec = spec.text("Open");
        assert!(spec.icon.is_some());
        assert_eq!(spec.text.as_deref(), Some("Open"));
    }
}
