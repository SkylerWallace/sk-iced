// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Tiny stateless share meter: cap radii must fit even a subpixel fill.
use super::Appearance;
use crate::animation::{Element, Renderer};
use iced_core::{Layout, Renderer as _, Widget, layout, mouse, renderer, widget::Tree};
use iced_core::{Length, Rectangle, Size, Theme};
pub struct ShareMeter {
    appearance: Appearance,
    percent: f32,
    height: f32,
}
impl ShareMeter {
    /// Percent in 0..100. Tiny rounded fills fit even subpixel widths.
    pub fn new(appearance: Appearance, percent: f32) -> Self {
        Self {
            appearance,
            percent,
            height: 8.0,
        }
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    pub fn build<'a, M: 'a>(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<ShareMeter> for Element<'a, M> {
    fn from(meter: ShareMeter) -> Self {
        meter.build()
    }
}
impl<M> Widget<M, Theme, Renderer> for ShareMeter {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(self.height), Size::ZERO))
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let t = self.appearance.tokens(theme);
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            for (index, (width, tone)) in [
                (bounds.width, t.border),
                (bounds.width * self.percent.clamp(0., 100.) / 100., t.accent),
            ]
            .into_iter()
            .enumerate()
            {
                if width <= 0. {
                    continue;
                }
                let height = bounds.height.min(width);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            width,
                            height,
                            y: bounds.center_y() - height / 2.,
                            ..bounds
                        },
                        border: iced_core::Border {
                            radius: t
                                .visual
                                .progress
                                .shape
                                .map(|shape| shape.radius())
                                .unwrap_or((bounds.height.min(width) / 2.).into()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    (if index == 1 {
                        t.visual.progress.fill
                    } else {
                        None
                    })
                    .map(|fill| fill.background(t.visual.progress.opacity()))
                    .unwrap_or(crate::theme::color(tone).into()),
                );
            }
        });
    }
}
