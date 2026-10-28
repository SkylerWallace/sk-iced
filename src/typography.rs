// SPDX-License-Identifier: Apache-2.0
//! Fonts are selected and registered by the app. These helpers return native
//! text widgets and never reload fonts, install fonts or add wrapper state.
use crate::design::TextScale;
use iced_core::{Font, Length, Pixels};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Tiny,
    Badge,
    Small,
    Caption,
    Control,
    Body,
    Emphasis,
    Heading,
    Subtitle,
    Title,
    Monospace,
}
#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub body: Font,
    pub monospace: Font,
    pub scale: TextScale,
}
impl Typography {
    pub fn new(body: Font, monospace: Font) -> Self {
        Self {
            body,
            monospace,
            scale: TextScale::default(),
        }
    }
    pub fn with_scale(mut self, scale: TextScale) -> Self {
        self.scale = scale;
        self
    }
    pub fn size(self, role: TextRole) -> f32 {
        match role {
            TextRole::Tiny => self.scale.tiny,
            TextRole::Badge => self.scale.badge,
            TextRole::Small => self.scale.small,
            TextRole::Caption => self.scale.caption,
            TextRole::Control => self.scale.control,
            TextRole::Body | TextRole::Monospace => self.scale.body,
            TextRole::Emphasis => self.scale.emphasis,
            TextRole::Heading => self.scale.heading,
            TextRole::Subtitle => self.scale.subtitle,
            TextRole::Title => self.scale.title,
        }
    }
    /// Compatibility path: preserve the renderer's inherited text size and all
    /// native defaults. Existing apps can migrate roles without global restyling.
    pub fn raw<'a>(
        self,
        content: impl iced_widget::text::IntoFragment<'a>,
    ) -> iced_widget::Text<'a> {
        iced_widget::text(content).font(self.body)
    }
    pub fn text<'a>(
        self,
        role: TextRole,
        content: impl iced_widget::text::IntoFragment<'a>,
    ) -> iced_widget::Text<'a> {
        self.raw(content)
            .font(if role == TextRole::Monospace {
                self.monospace
            } else {
                self.body
            })
            .size(self.size(role))
    }
}
/// Optional fluent text builder. Font weight, wrapping and line height retain
/// native defaults; roles specify family/size without inventing bold styling.
pub struct Text<'a> {
    inner: iced_widget::Text<'a>,
}
impl<'a> Text<'a> {
    pub fn new(
        typography: Typography,
        role: TextRole,
        content: impl iced_widget::text::IntoFragment<'a>,
    ) -> Self {
        Self {
            inner: typography.text(role, content),
        }
    }
    pub fn custom(inner: iced_widget::Text<'a>) -> Self {
        Self { inner }
    }
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(size);
        self
    }
    pub fn size(self, size: impl Into<Pixels>) -> Self {
        self.text_size(size)
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }
    pub fn font(mut self, font: Font) -> Self {
        self.inner = self.inner.font(font);
        self
    }
    pub fn tone(mut self, role: crate::design::ColorRole) -> Self {
        self.inner = self.inner.style(move |theme| iced_widget::text::Style {
            color: Some(crate::theme::color(
                role.resolve(crate::palette::from_theme(theme)),
            )),
        });
        self
    }
    pub fn color(mut self, color: iced_core::Color) -> Self {
        self.inner = self.inner.color(color);
        self
    }
    pub fn wrapping(mut self, wrapping: iced_core::text::Wrapping) -> Self {
        self.inner = self.inner.wrapping(wrapping);
        self
    }
    pub fn line_height(mut self, height: impl Into<iced_core::text::LineHeight>) -> Self {
        self.inner = self.inner.line_height(height);
        self
    }
    pub fn build(self) -> iced_widget::Text<'a> {
        self.inner
    }
}
impl<'a, M: 'a> From<Text<'a>>
    for iced_core::Element<'a, M, iced_core::Theme, iced_renderer::Renderer>
{
    fn from(spec: Text<'a>) -> Self {
        spec.inner.into()
    }
}
