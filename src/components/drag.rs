// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Visible drag geometry collected through native scroll offsets and clips.
use iced_core::{Point, Rectangle, Vector, widget::Operation};
#[derive(Clone)]
pub struct DragRegion<T> {
    pub marker: T,
    pub bounds: Rectangle,
    pub full: Rectangle,
}
pub struct DragRegions<T> {
    pub regions: Vec<DragRegion<T>>,
    offset: Vector,
    clip: Option<Rectangle>,
    pending: Option<(Rectangle, Vector)>,
    pub focused_input: bool,
}
impl<T> DragRegions<T> {
    pub fn new() -> Self {
        Self {
            regions: vec![],
            offset: Vector::ZERO,
            clip: None,
            pending: None,
            focused_input: false,
        }
    }
}
impl<T: Clone + Send + 'static> Operation for DragRegions<T> {
    fn focusable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        _: Rectangle,
        state: &mut dyn iced_core::widget::operation::Focusable,
    ) {
        self.focused_input |= state.is_focused();
    }
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        let previous = (self.offset, self.clip);
        if let Some((bounds, translation)) = self.pending.take() {
            let bounds = bounds + self.offset;
            self.clip = match self.clip {
                Some(clip) => Some(clip.intersection(&bounds).unwrap_or_default()),
                None => Some(bounds),
            };
            self.offset -= translation;
        }
        f(self);
        self.offset = previous.0;
        self.clip = previous.1;
    }
    fn scrollable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        bounds: Rectangle,
        _: Rectangle,
        translation: Vector,
        _: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        self.pending = Some((bounds, translation));
    }
    fn custom(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if let Some(marker) = state.downcast_ref::<T>() {
            let full = bounds + self.offset;
            let bounds = if let Some(clip) = self.clip {
                full.intersection(&clip)
            } else {
                Some(full)
            };
            if let Some(bounds) = bounds {
                self.regions.push(DragRegion {
                    marker: marker.clone(),
                    bounds,
                    full,
                });
            }
        }
    }
}

impl<T> Default for DragRegions<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct DragGesture {
    pub origin: Point,
    pub lifted: bool,
}
impl DragGesture {
    pub fn begin(&mut self, origin: Point) {
        self.origin = origin;
        self.lifted = false;
    }
    pub fn advance(&mut self, point: Point, threshold: f32) {
        self.lifted |= point.distance(self.origin) >= threshold;
    }
    pub fn reset(&mut self) {
        self.lifted = false;
    }
    pub fn crossed(origin: Point, point: Point, threshold: f32) -> bool {
        point.distance(origin) > threshold
    }
}
/// Edge scrolling uses the caller's existing tick; no independent timer.
pub fn edge_scroll(bounds: Rectangle, point: Point, inset: f32, step: f32) -> f32 {
    if point.y < bounds.y + inset {
        -step
    } else if point.y > bounds.y + bounds.height - inset {
        step
    } else {
        0.0
    }
}
pub fn cancel_drag(event: &iced_core::Event, leave: bool) -> bool {
    matches!(
        event,
        iced_core::Event::Window(iced_core::window::Event::Unfocused)
    ) || leave
        && matches!(
            event,
            iced_core::Event::Mouse(iced_core::mouse::Event::CursorLeft)
        )
        || matches!(
            event,
            iced_core::Event::Keyboard(iced_core::keyboard::Event::KeyPressed {
                key: iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape),
                ..
            })
        )
}
