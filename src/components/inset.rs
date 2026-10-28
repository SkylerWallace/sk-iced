// SPDX-License-Identifier: Apache-2.0
use crate::{animation::Renderer, visual::SurfaceStyle};
use iced_core::{Border, Color, Rectangle, Renderer as _, renderer};
pub(crate) fn paint(renderer: &mut Renderer, bounds: Rectangle, style: SurfaceStyle) {
    let Some(inset) = style.inset else {
        return;
    };
    let width = inset
        .width
        .max(0.)
        .min(bounds.height / 2.)
        .min(bounds.width / 2.);
    if width <= 0. {
        return;
    }
    let radius = style
        .shape
        .map(|shape| shape.radius())
        .unwrap_or(2.0.into());
    // Corner-sized clips preserve rounded corners without a seam halfway along a side.
    let corner = <[f32; 4]>::from(radius)
        .into_iter()
        .fold(width, f32::max)
        .min(bounds.height / 2.)
        .min(bounds.width / 2.);
    for (clip, color) in [
        (
            Rectangle {
                height: corner,
                width: bounds.width - corner,
                ..bounds
            },
            inset.shade.color(),
        ),
        (
            Rectangle {
                y: bounds.y + corner,
                width: corner,
                height: bounds.height - corner,
                ..bounds
            },
            inset.shade.color(),
        ),
        (
            Rectangle {
                x: bounds.x + corner,
                y: bounds.y + bounds.height - corner,
                width: bounds.width - corner,
                height: corner,
            },
            inset.light.color(),
        ),
        (
            Rectangle {
                x: bounds.x + bounds.width - corner,
                width: corner,
                height: bounds.height - corner,
                ..bounds
            },
            inset.light.color(),
        ),
    ] {
        renderer.with_layer(clip, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius,
                        width,
                        color: color.scale_alpha(style.opacity()),
                    },
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        });
    }
}
