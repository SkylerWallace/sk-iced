// SPDX-License-Identifier: Apache-2.0
//! Opt-in full-frame damage for renderers that cannot safely retain shadow pixels.
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Renderer as _, Shell, Size, Theme, Vector, Widget,
    layout, mouse, overlay, renderer,
    widget::{Operation, Tree, tree},
};

/// Keeps native layout, input, state and overlays intact. With full_frame(true),
/// an invisible frame marker makes each requested frame damage the entire view.
/// It requests no frames itself. Use at the window root with stock Iced 0.14.1
/// tiny-skia shadows; leave disabled with a renderer that handles partial damage.
pub struct RedrawBoundary<'a, M> {
    content: Element<'a, M>,
    full_frame: bool,
}
#[derive(Default)]
struct Frame(bool);
impl<'a, M: 'a> RedrawBoundary<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            full_frame: false,
        }
    }
    pub fn full_frame(mut self, enabled: bool) -> Self {
        self.full_frame = enabled;
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<RedrawBoundary<'a, M>> for Element<'a, M> {
    fn from(boundary: RedrawBoundary<'a, M>) -> Self {
        boundary.build()
    }
}
impl<M> Widget<M, Theme, Renderer> for RedrawBoundary<'_, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Frame>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(Frame::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if self.full_frame {
            // Both colors are exactly transparent. Their distinct RGB values
            // invalidate retained damage without changing any displayed pixel.
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                iced_core::Color::from_rgba(
                    if tree.state.downcast_ref::<Frame>().0 {
                        1.
                    } else {
                        0.
                    },
                    0.,
                    0.,
                    0.,
                ),
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if self.full_frame
            && matches!(
                event,
                Event::Window(iced_core::window::Event::RedrawRequested(_))
            )
        {
            let frame = tree.state.downcast_mut::<Frame>();
            frame.0 = !frame.0;
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_core::{Font, renderer::Headless as _};

    #[test]
    fn full_frame_marker_is_pixel_neutral_and_never_schedules_idle_redraws() {
        let mut view: Element<'_, ()> = RedrawBoundary::new(
            iced_widget::container(iced_widget::Space::new())
                .width(80)
                .height(40)
                .style(|_| iced_widget::container::Style {
                    background: Some(iced_core::Color::from_rgb8(17, 34, 51).into()),
                    ..Default::default()
                }),
        )
        .full_frame(true)
        .into();
        let mut tree = Tree::new(&view);
        let mut renderer = Renderer::new(Font::DEFAULT, 13.0.into());
        let bounds = Rectangle::with_size(Size::new(80., 40.));
        let node = view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, bounds.size()),
        );
        let mut screenshots = Vec::new();
        for _ in 0..2 {
            let mut messages = Vec::new();
            let mut shell = Shell::new(&mut messages);
            view.as_widget_mut().update(
                &mut tree,
                &Event::Window(iced_core::window::Event::RedrawRequested(
                    std::time::Instant::now(),
                )),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut shell,
                &bounds,
            );
            assert_eq!(
                shell.redraw_request(),
                iced_core::window::RedrawRequest::Wait
            );
            renderer.reset(bounds);
            view.as_widget().draw(
                &tree,
                &mut renderer,
                &Theme::Dark,
                &renderer::Style::default(),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &bounds,
            );
            screenshots.push(renderer.screenshot(Size::new(80, 40), 1., iced_core::Color::BLACK));
        }
        assert_eq!(screenshots[0], screenshots[1]);
        assert!(
            screenshots[0]
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [17, 34, 51, 255])
        );
    }
}
