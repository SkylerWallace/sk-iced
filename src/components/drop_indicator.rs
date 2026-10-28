// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Allocation-free drop feedback painting with a clipped antialias fringe.
use crate::animation::Renderer;
use iced_core::{Border, Color, Rectangle, Renderer as _, renderer};
pub struct DropIndicator {
    bounds: Rectangle,
    fill: Color,
    border: Border,
    clip: Option<Rectangle>,
    fringe: f32,
}
impl DropIndicator {
    pub fn new(bounds: Rectangle, fill: Color) -> Self {
        Self {
            bounds,
            fill,
            border: Border::default(),
            clip: None,
            fringe: 1.0,
        }
    }
    pub fn border(mut self, color: Color, width: f32, radius: f32) -> Self {
        self.border = Border {
            color,
            width,
            radius: radius.into(),
        };
        self
    }
    pub fn clip(mut self, clip: Option<Rectangle>) -> Self {
        self.clip = clip;
        self
    }
    pub fn fringe(mut self, fringe: f32) -> Self {
        self.fringe = fringe.max(0.0);
        self
    }
    pub fn draw(self, renderer: &mut Renderer) {
        let expanded = self.bounds.expand(self.fringe);
        let clip = self
            .clip
            .map_or(Some(expanded), |clip| expanded.intersection(&clip));
        if let Some(clip) = clip {
            renderer.with_layer(clip, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: self.bounds,
                        border: self.border,
                        ..Default::default()
                    },
                    self.fill,
                )
            });
        }
    }
}
