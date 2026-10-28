// SPDX-License-Identifier: Apache-2.0
//! App-owned artwork with shared sizing, tinting and optional raster mip caches.
//! Assets and their licenses stay in the app; handles are prepared outside paint.
pub use crate::design::ColorRole as IconTone;
use crate::{animation::Element, components::Appearance, design::size};
use iced_core::{Font, Length};
use iced_widget::image;
use std::collections::BTreeMap;

// Icons share the button's themed disabled ink instead of a separate opacity rule.
fn disabled_ink(t: crate::theme::Tokens) -> iced_core::Color {
    let s = t
        .visual
        .button
        .button(iced_widget::button::Status::Disabled);
    s.text_color
        .map(|c| c.color().scale_alpha(s.opacity()))
        .unwrap_or_else(|| crate::theme::color(t.secondary).scale_alpha(t.opacity_disabled as f32))
}
#[derive(Clone)]
enum Source {
    Glyph(Font, char),
    Raster(image::Handle),
    #[cfg(feature = "svg")]
    Svg(iced_widget::svg::Handle),
}
#[derive(Clone)]
pub struct Icon {
    appearance: Appearance,
    source: Source,
    width: Length,
    height: Length,
    tone: IconTone,
    enabled: bool,
    opacity: f32,
}
impl Icon {
    pub fn raster(appearance: Appearance, handle: image::Handle) -> Self {
        Self {
            appearance,
            source: Source::Raster(handle),
            width: size::ICON.into(),
            height: size::ICON.into(),
            tone: IconTone::Text,
            enabled: true,
            opacity: 1.0,
        }
    }
    #[cfg(feature = "svg")]
    pub fn svg(appearance: Appearance, handle: iced_widget::svg::Handle) -> Self {
        Self {
            appearance,
            source: Source::Svg(handle),
            width: size::ICON.into(),
            height: size::ICON.into(),
            tone: IconTone::Text,
            enabled: true,
            opacity: 1.0,
        }
    }
    pub fn size(mut self, size: impl Into<iced_core::Pixels>) -> Self {
        let size = size.into().0;
        self.width = size.into();
        self.height = size.into();
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn tone(mut self, tone: IconTone) -> Self {
        self.tone = tone;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
    pub fn build<'a, M: 'a>(self) -> Element<'a, M> {
        match self.source {
            Source::Glyph(font, character) => {
                let icon = iced_widget::text(character.to_string())
                    .font(font)
                    .size(match self.height {
                        Length::Fixed(height) => height,
                        _ => size::ICON,
                    })
                    .style(move |theme: &iced_core::Theme| {
                        let t = self.appearance.tokens(theme);
                        iced_widget::text::Style {
                            color: Some(if self.enabled {
                                crate::theme::color(self.tone.resolve(t)).scale_alpha(self.opacity)
                            } else {
                                disabled_ink(t).scale_alpha(self.opacity)
                            }),
                        }
                    });
                iced_widget::container(icon)
                    .center_x(self.width)
                    .center_y(self.height)
                    .into()
            }
            // Raster artwork is already colored by the cache; never retint a
            // branded or multi-color image while drawing it.
            Source::Raster(handle) => {
                if !self.enabled
                    && matches!(
                        self.appearance.palette,
                        crate::components::Palette::Themed(_)
                    )
                {
                    Element::new(ThemedRaster {
                        appearance: self.appearance,
                        handle,
                        width: self.width,
                        height: self.height,
                        opacity: self.opacity,
                    })
                } else {
                    let disabled = match self.appearance.palette {
                        crate::components::Palette::Fixed(t) if !self.enabled => {
                            t.opacity_disabled as f32
                        }
                        _ => 1.0,
                    };
                    iced_widget::image(handle)
                        .width(self.width)
                        .height(self.height)
                        .opacity(self.opacity * disabled)
                        .into()
                }
            }
            #[cfg(feature = "svg")]
            Source::Svg(handle) => iced_widget::svg(handle)
                .width(self.width)
                .height(self.height)
                .style(move |theme, _| {
                    let t = self.appearance.tokens(theme);
                    iced_widget::svg::Style {
                        color: Some(if self.enabled {
                            crate::theme::color(self.tone.resolve(t)).scale_alpha(self.opacity)
                        } else {
                            disabled_ink(t).scale_alpha(self.opacity)
                        }),
                    }
                })
                .into(),
        }
    }
}
impl<'a, M: 'a> From<Icon> for Element<'a, M> {
    fn from(spec: Icon) -> Self {
        spec.build()
    }
}

// Only dynamically themed, disabled raster images need theme-time opacity.
// Enabled and fixed-palette icons retain their original native widget path.
struct ThemedRaster {
    appearance: Appearance,
    handle: image::Handle,
    width: Length,
    height: Length,
    opacity: f32,
}
impl ThemedRaster {
    fn image(&self, opacity: f32) -> iced_widget::Image {
        iced_widget::image(self.handle.clone())
            .width(self.width)
            .height(self.height)
            .opacity(opacity)
    }
}
impl<M> iced_core::Widget<M, iced_core::Theme, crate::animation::Renderer> for ThemedRaster {
    fn size(&self) -> iced_core::Size<Length> {
        <iced_widget::Image as iced_core::Widget<M, iced_core::Theme, crate::animation::Renderer>>::size(&self.image(1.0))
    }
    fn layout(
        &mut self,
        tree: &mut iced_core::widget::Tree,
        renderer: &crate::animation::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        <iced_widget::Image as iced_core::Widget<M, iced_core::Theme, crate::animation::Renderer>>::layout(&mut self.image(1.0), tree, renderer, limits)
    }
    fn draw(
        &self,
        tree: &iced_core::widget::Tree,
        renderer: &mut crate::animation::Renderer,
        theme: &iced_core::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        let image =
            self.image(self.opacity * self.appearance.tokens(theme).opacity_disabled as f32);
        <iced_widget::Image as iced_core::Widget<
            M,
            iced_core::Theme,
            crate::animation::Renderer,
        >>::draw(
            &image, tree, renderer, theme, style, layout, cursor, viewport,
        );
    }
}

/// Bounded tint cache for alpha-mask artwork such as disclosure chevrons.
pub struct AlphaIconCache<K: Ord> {
    handles: BTreeMap<(K, u32), image::Handle>,
    limit: usize,
}
impl<K: Ord> Default for AlphaIconCache<K> {
    fn default() -> Self {
        Self::new(256)
    }
}
impl<K: Ord> AlphaIconCache<K> {
    pub fn new(limit: usize) -> Self {
        Self {
            handles: BTreeMap::new(),
            limit: limit.max(1),
        }
    }
    pub fn get_or_insert(
        &mut self,
        key: K,
        tint: u32,
        width: u32,
        height: u32,
        alpha: &[u8],
    ) -> Option<image::Handle> {
        if width == 0
            || height == 0
            || (width as usize).checked_mul(height as usize)? != alpha.len()
        {
            return None;
        }
        let key = (key, tint);
        if let Some(handle) = self.handles.get(&key) {
            return Some(handle.clone());
        }
        if self.handles.len() >= self.limit {
            self.handles.clear();
        }
        let mut rgba = Vec::with_capacity(alpha.len().checked_mul(4)?);
        for &a in alpha {
            rgba.extend_from_slice(&[tint as u8, (tint >> 8) as u8, (tint >> 16) as u8, a]);
        }
        let handle = image::Handle::from_rgba(width, height, rgba);
        self.handles.insert(key, handle.clone());
        Some(handle)
    }
    pub fn len(&self) -> usize {
        self.handles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}

#[cfg(feature = "raster-icons")]
pub struct RasterIconSet {
    sources: Vec<::image::RgbaImage>,
    scale: f32,
    handles: BTreeMap<(Option<u32>, u16), Vec<image::Handle>>,
}
#[cfg(feature = "raster-icons")]
impl RasterIconSet {
    /// Decoding remains app-owned; this cache needs no image codecs.
    pub fn new(sources: Vec<::image::RgbaImage>) -> Self {
        Self {
            sources,
            scale: 1.0,
            handles: BTreeMap::new(),
        }
    }
    pub fn set_scale(&mut self, scale: f32) -> bool {
        let scale = scale.clamp(0.5, 4.0);
        if (scale - self.scale).abs() < 0.01 {
            return false;
        }
        self.scale = scale;
        self.handles.clear();
        true
    }
    /// Lanczos3 at physical size, matching the original CPU icon preparation.
    /// This is called during preparation/scale changes, never during paint.
    pub fn prepare(&mut self, tint: Option<u32>, logical_size: u16) {
        let key = (tint, logical_size);
        if self.handles.contains_key(&key) {
            return;
        }
        let physical = (logical_size as f32 * self.scale).round().max(1.0) as u32;
        let handles = self
            .sources
            .iter()
            .map(|source| {
                let mut out = ::image::imageops::resize(
                    source,
                    physical,
                    physical,
                    ::image::imageops::FilterType::Lanczos3,
                );
                if let Some(tint) = tint {
                    for p in out.pixels_mut() {
                        p.0[0] = tint as u8;
                        p.0[1] = (tint >> 8) as u8;
                        p.0[2] = (tint >> 16) as u8;
                    }
                }
                image::Handle::from_rgba(physical, physical, out.into_raw())
            })
            .collect();
        self.handles.insert(key, handles);
    }
    pub fn get(&self, index: usize, tint: Option<u32>, logical_size: u16) -> Option<image::Handle> {
        self.handles.get(&(tint, logical_size))?.get(index).cloned()
    }
}

mod named;
pub use named::NamedIcon;

#[cfg(test)]
mod disabled_ink_checks {
    #[test]
    fn neumorphic_icons_use_navigation_disabled_ink() {
        let t = crate::themes::neumorphic(crate::palette::palette(true, false)).tokens;
        let expected = (t.button(false, false))(
            &iced_core::Theme::Dark,
            iced_widget::button::Status::Disabled,
        )
        .text_color;
        assert_eq!(super::disabled_ink(t), expected);
    }
}
