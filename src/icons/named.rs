// SPDX-License-Identifier: Apache-2.0
//! Semantic icon lookup with a retained, noninteractive native rendering cache.
use super::*;
use crate::animation::Renderer;
use iced_core::{
    Layout, Rectangle, Renderer as _, Size, Theme, Widget, layout, mouse, renderer,
    widget::{Tree, tree},
};
use std::cell::RefCell;
pub struct NamedIcon {
    fallback: Icon,
    name: &'static str,
}
struct Cached {
    tokens: crate::theme::Tokens,
    size: Size,
    signature: u64,
    element: Element<'static, u8>,
    tree: Tree,
    node: layout::Node,
}
#[derive(Default)]
struct State(RefCell<Option<Cached>>);
impl Icon {
    /// Resolve a semantic name through the current theme's pack, falling back to this icon.
    pub fn semantic(self, name: &'static str) -> NamedIcon {
        NamedIcon {
            fallback: self,
            name,
        }
    }
    pub fn from_asset(appearance: Appearance, asset: crate::visual::IconAsset) -> Option<Self> {
        use crate::visual::IconAsset;
        match asset {
            IconAsset::Svg(bytes) => {
                #[cfg(feature = "svg")]
                {
                    Some(Self::svg(
                        appearance,
                        iced_widget::svg::Handle::from_memory(bytes),
                    ))
                }
                #[cfg(not(feature = "svg"))]
                {
                    let _ = bytes;
                    None
                }
            }
            IconAsset::Rgba {
                width,
                height,
                pixels,
            } => {
                if width == 0
                    || height == 0
                    || (width as usize)
                        .checked_mul(height as usize)?
                        .checked_mul(4)?
                        != pixels.len()
                {
                    return None;
                }
                Some(Self::raster(
                    appearance,
                    image::Handle::from_rgba(width, height, pixels.to_vec()),
                ))
            }
            IconAsset::Glyph { font, character } => Some(Self {
                appearance,
                source: Source::Glyph(font, character),
                width: size::ICON.into(),
                height: size::ICON.into(),
                tone: IconTone::Text,
                enabled: true,
                opacity: 1.,
            }),
        }
    }
}
impl<M> Widget<M, Theme, Renderer> for NamedIcon {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.fallback.width, self.fallback.height)
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(
            self.fallback.width,
            self.fallback.height,
            Size::new(16., 16.),
        ))
    }
    fn draw(
        &self,
        tree: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use std::hash::{Hash, Hasher};
        let tokens = self.fallback.appearance.tokens(theme);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // View rebuilds can change disabled state/tone without changing the pack.
        self.fallback.enabled.hash(&mut hasher);
        self.fallback.opacity.to_bits().hash(&mut hasher);
        self.fallback.tone.resolve(tokens).hash(&mut hasher);
        self.name.hash(&mut hasher);
        match &self.fallback.source {
            Source::Raster(handle) => handle.id().hash(&mut hasher),
            #[cfg(feature = "svg")]
            Source::Svg(handle) => handle.id().hash(&mut hasher),
            Source::Glyph(font, character) => {
                font.hash(&mut hasher);
                character.hash(&mut hasher);
            }
        }
        let signature = hasher.finish();
        let mut cache = tree.state.downcast_ref::<State>().0.borrow_mut();
        let size = layout.bounds().size();
        if cache
            .as_ref()
            .is_none_or(|c| c.tokens != tokens || c.size != size || c.signature != signature)
        {
            let mut icon = tokens
                .assets
                .icons
                .and_then(|pack| pack(self.name))
                .and_then(|asset| Icon::from_asset(self.fallback.appearance, asset))
                .unwrap_or_else(|| self.fallback.clone());
            icon.enabled = self.fallback.enabled;
            icon.tone = self.fallback.tone;
            icon.opacity = self.fallback.opacity;
            icon.width = size.width.into();
            icon.height = size.height.into();
            let mut element: Element<'static, u8> = icon.build();
            let mut tree = Tree::new(&element);
            let node =
                element
                    .as_widget_mut()
                    .layout(&mut tree, r, &layout::Limits::new(size, size));
            *cache = Some(Cached {
                tokens,
                size,
                signature,
                element,
                tree,
                node,
            });
        }
        let c = cache.as_ref().unwrap();
        // Translate the cached local node without changing hit testing or application state.
        r.with_translation(
            iced_core::Vector::new(layout.bounds().x, layout.bounds().y),
            |r| {
                c.element.as_widget().draw(
                    &c.tree,
                    r,
                    theme,
                    style,
                    Layout::new(&c.node),
                    mouse::Cursor::Unavailable,
                    &Rectangle {
                        x: viewport.x - layout.bounds().x,
                        y: viewport.y - layout.bounds().y,
                        ..*viewport
                    },
                );
            },
        );
        let _ = cursor;
    }
}
impl NamedIcon {
    pub fn build<'a, M: 'a>(self) -> Element<'a, M> {
        Element::new(self)
    }
}
