// SPDX-License-Identifier: Apache-2.0
// Clipping pixel oracles contributed by PC Tonic.
use super::*;
#[cfg(test)]
mod pctonic_cached_clip_tests {
    use super::*;
    use crate::core::{
        Font, Pixels, alignment,
        text::{Alignment, Shaping},
    };
    #[test]
    fn cached_label_respects_local_viewport_inside_a_larger_layer() {
        let layer = Rectangle::new(crate::core::Point::ORIGIN, Size::new(100., 80.));
        let local = Rectangle::new(crate::core::Point::new(0., 32.), Size::new(100., 6.));
        let label = |clip_bounds| Text::Cached {
            content: "\u{e800}\u{e800}".into(),
            bounds: Rectangle::new(crate::core::Point::new(10., 20.), Size::new(60., 32.)),
            color: Color::WHITE,
            size: Pixels(32.),
            line_height: Pixels(32.),
            font: Font::with_name("Iced-Icons"),
            align_x: Alignment::Left,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            clip_bounds,
        };
        let mut engine = Engine::new();
        let mut full = tiny_skia::Pixmap::new(100, 80).unwrap();
        let mut clipped = full.clone();
        let mut mask = tiny_skia::Mask::new(100, 80).unwrap();
        engine.draw_text(
            &label(layer),
            Transformation::IDENTITY,
            &mut full.as_mut(),
            &mut mask,
            layer,
        );
        engine.draw_text(
            &label(local),
            Transformation::IDENTITY,
            &mut clipped.as_mut(),
            &mut mask,
            layer,
        );
        let mut inside = 0;
        let mut outside = 0;
        for y in 0..80 {
            for x in 0..100 {
                let i = (y * 100 + x) * 4;
                let pixel = &full.data()[i..i + 4];
                if (32..38).contains(&y) {
                    inside += usize::from(pixel[3] > 0);
                    assert_eq!(&clipped.data()[i..i + 4], pixel);
                } else {
                    outside += usize::from(pixel[3] > 0);
                    assert_eq!(&clipped.data()[i..i + 4], &[0; 4]);
                }
            }
        }
        assert!(
            inside > 0 && outside > 0,
            "fixture must straddle the viewport"
        );
    }
}

#[cfg(test)]
mod pctonic_shadow_clip_tests {
    use super::*;
    #[test]
    fn shadow_halo_is_clipped_even_when_the_quad_fits_inside_damage() {
        let full_bounds = Rectangle::new(crate::core::Point::ORIGIN, Size::new(100., 80.));
        let clip = Rectangle::new(crate::core::Point::new(20., 20.), Size::new(50., 35.));
        let quad = Quad {
            bounds: Rectangle::new(crate::core::Point::new(30., 28.), Size::new(30., 20.)),
            border: crate::core::Border {
                radius: 5.0.into(),
                ..Default::default()
            },
            shadow: crate::core::Shadow {
                color: Color::BLACK,
                offset: Vector::new(4., 3.),
                blur_radius: 12.,
            },
            ..Default::default()
        };
        let mut engine = Engine::new();
        let mut mask = tiny_skia::Mask::new(100, 80).unwrap();
        let mut full = tiny_skia::Pixmap::new(100, 80).unwrap();
        let mut clipped = full.clone();
        engine.adjust_clip_mask(&mut mask, full_bounds);
        engine.draw_quad(
            &quad,
            &Background::Color(Color::WHITE),
            Transformation::IDENTITY,
            &mut full.as_mut(),
            &mut mask,
            full_bounds,
        );
        engine.adjust_clip_mask(&mut mask, clip);
        engine.draw_quad(
            &quad,
            &Background::Color(Color::WHITE),
            Transformation::IDENTITY,
            &mut clipped.as_mut(),
            &mut mask,
            clip,
        );
        let mut outside_ink = 0;
        for y in 0..80 {
            for x in 0..100 {
                let i = (y * 100 + x) * 4;
                if (20..70).contains(&x) && (20..55).contains(&y) {
                    assert_eq!(&clipped.data()[i..i + 4], &full.data()[i..i + 4]);
                } else {
                    outside_ink += usize::from(full.data()[i + 3] > 0);
                    assert_eq!(&clipped.data()[i..i + 4], &[0; 4]);
                }
            }
        }
        assert!(
            outside_ink > 0,
            "fixture must have a shadow outside the clip"
        );
    }
}
