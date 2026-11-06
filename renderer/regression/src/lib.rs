// SPDX-License-Identifier: Apache-2.0
//! Headless renderer contracts shared by software-rendered Iced applications.
use iced_core::{
    Element, Event, Font, Layout, Point, Rectangle, Renderer as _, Shell, Size, Theme, Vector,
    clipboard, layout, mouse, renderer, widget::Tree,
};
type View<'a, M> = Element<'a, M, Theme, iced_renderer::Renderer>;
/// Open the caller's first dropdown and require partial repaints to match a full frame.
/// The fixture places that dropdown at (40,24), width 198, with another shadowed
/// control below it. It schedules no native window, application task or idle frame.
pub fn assert_open_dropdown_repaints<M: Clone>(
    mut view: View<'_, M>,
    theme: &Theme,
    background: iced_core::Color,
    font: Font,
    font_bytes: &'static [u8],
    name: &str,
    scale: f32,
) {
    iced_graphics::text::font_system()
        .write()
        .unwrap()
        .load_font(std::borrow::Cow::Borrowed(font_bytes));
    let bounds = Rectangle::with_size(Size::new(420., 260.));
    let mut renderer = iced_renderer::Renderer::new(font, 13.333.into());
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, bounds.size()),
    );
    let mut messages = vec![];
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(50., 35.)),
        &renderer,
        &mut clipboard::Null,
        &mut Shell::new(&mut messages),
        &bounds,
    );
    renderer.reset(bounds);
    view.as_widget().draw(
        &tree,
        &mut renderer,
        theme,
        &renderer::Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    let overlay = view
        .as_widget_mut()
        .overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &bounds,
            Vector::ZERO,
        )
        .expect("Style menu must open");
    let mut overlay = iced_core::overlay::Nested::new(overlay);
    let overlay_node = overlay.layout(&renderer, bounds.size());
    overlay.draw(
        &mut renderer,
        theme,
        &renderer::Style::default(),
        Layout::new(&overlay_node),
        mouse::Cursor::Unavailable,
    );
    let physical = Size::new((420. * scale) as u32, (260. * scale) as u32);
    let viewport = iced_graphics::Viewport::with_physical_size(physical, scale);
    let mut mask = tiny_skia::Mask::new(physical.width, physical.height).unwrap();
    let mut full = tiny_skia::Pixmap::new(physical.width, physical.height).unwrap();
    renderer.draw(
        &mut full.as_mut(),
        &mut mask,
        &viewport,
        &[bounds],
        background,
    );
    let mut partial = full.clone();
    for _ in 0..3 {
        renderer.draw(
            &mut partial.as_mut(),
            &mut mask,
            &viewport,
            &[
                Rectangle::new(Point::new(60., 80.), Size::new(16., 12.)),
                Rectangle::new(Point::new(212., 98.), Size::new(14., 8.)),
            ],
            background,
        );
    }
    if let Some(folder) = std::env::var_os("SK_ICED_RENDERER_CAPTURE_DIR") {
        let folder = std::path::PathBuf::from(folder);
        std::fs::create_dir_all(&folder).unwrap();
        for (name_kind, pixels) in [("open", &full), ("partial", &partial)] {
            let mut rgba = pixels.data().to_vec();
            for p in rgba.chunks_exact_mut(4) {
                p.swap(0, 2);
            }
            let mut ppm = format!("P6\n{} {}\n255\n", physical.width, physical.height).into_bytes();
            for pixel in rgba.chunks_exact(4) {
                ppm.extend_from_slice(&pixel[..3]);
            }
            std::fs::write(folder.join(format!("{name}-{scale}-{name_kind}.ppm")), ppm).unwrap();
        }
    }

    let changed = full
        .data()
        .chunks_exact(4)
        .zip(partial.data().chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        changed, 0,
        "{name} scale={scale}: unchanged open menu corrupted by partial repaint"
    );
}
