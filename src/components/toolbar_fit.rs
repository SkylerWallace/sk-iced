// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
// Measure the actual single-row widget tree before choosing its layout.
use iced_core::{
    Clipboard, Shell, Widget,
    layout::{self, Layout},
    mouse, overlay, renderer,
    widget::{self, Tree},
};
use iced_core::{Element, Event, Length, Rectangle, Size, Vector};

/// Chooses the compact ribbon only when the measured single row would overflow.
/// Event, drawing, and overlay forwarding follow Iced’s Responsive widget.
pub struct AdaptiveToolbar<
    'a,
    Message,
    Theme = iced_core::Theme,
    Renderer = crate::animation::Renderer,
> {
    probe: Element<'a, Message, Theme, Renderer>,
    probe_tree: Tree,
    view: Box<dyn Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a>,
    width: Length,
    height: Length,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> AdaptiveToolbar<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// Measures the probe and passes the overflow result to the content builder.
    pub fn new(
        probe: Element<'a, Message, Theme, Renderer>,
        view: impl Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a,
    ) -> Self {
        let probe_tree = Tree::new(&probe);
        Self {
            probe,
            probe_tree,
            view: Box::new(view),
            width: Length::Fill,
            height: Length::Shrink,
            content: Element::new(iced_widget::Space::new()),
        }
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn build(self) -> Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: 'a,
    {
        Element::new(self)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AdaptiveToolbar<'_, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer,
{
    fn diff(&self, _tree: &mut Tree) {
        // Diff is deferred to layout
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.max();

        let required = self
            .probe
            .as_widget_mut()
            .layout(
                &mut self.probe_tree,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(100000.0, 200.0)),
            )
            .size()
            .width;
        self.content = (self.view)(size.width < required);
        tree.diff_children(std::slice::from_ref(&self.content));

        let node =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());

        let size = limits.resolve(self.width, self.height, node.size());

        layout::Node::with_children(size, vec![node])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
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
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<AdaptiveToolbar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(responsive: AdaptiveToolbar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(responsive)
    }
}
