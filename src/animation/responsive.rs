//! Cache a responsive view within the current app view while its limits stay
//! unchanged. Animation relayouts reuse elements; a new app view always rebuilds
//! from current data. In contrast, Iced's Responsive rebuilds on every layout.
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};

pub fn responsive<'a, M: 'a>(view: impl Fn(Size) -> Element<'a, M> + 'a) -> Responsive<'a, M> {
    Responsive {
        view: Box::new(view),
        content: None,
        height: Length::Fill,
    }
}
pub struct Responsive<'a, M> {
    view: Box<dyn Fn(Size) -> Element<'a, M> + 'a>,
    content: Option<(Size, Element<'a, M>)>,
    height: Length,
}
impl<M> Responsive<'_, M> {
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}
impl<M> Widget<M, Theme, Renderer> for Responsive<'_, M> {
    fn diff(&self, _: &mut Tree) {} // Current size is only known during layout.
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, self.height)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(self.height);
        let available = limits.max();
        if self
            .content
            .as_ref()
            .is_none_or(|(size, _)| *size != available)
        {
            self.content = Some((available, (self.view)(available)));
            tree.diff_children(std::slice::from_ref(&self.content.as_ref().unwrap().1));
        }
        let node = self.content.as_mut().unwrap().1.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.loose(),
        );
        layout::Node::with_children(
            limits.resolve(Length::Fill, self.height, node.size()),
            vec![node],
        )
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
        self.content.as_mut().unwrap().1.as_widget_mut().update(
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
        self.content.as_ref().unwrap().1.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
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
        self.content.as_mut().unwrap().1.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
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
        self.content
            .as_ref()
            .unwrap()
            .1
            .as_widget()
            .mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
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
        self.content.as_mut().unwrap().1.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}
impl<'a, M: 'a> From<Responsive<'a, M>> for Element<'a, M> {
    fn from(widget: Responsive<'a, M>) -> Self {
        Element::new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    #[test]
    fn relayout_reuses_elements_and_resize_and_new_views_get_fresh_data() {
        let builds = Rc::new(Cell::new(0));
        let make = || {
            let builds = builds.clone();
            responsive(move |_| {
                builds.set(builds.get() + 1);
                iced_widget::Space::new().height(20).into()
            })
            .height(Length::Shrink)
        };
        let mut view: Element<'_, ()> = make().into();
        let mut tree = Tree::new(&view);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        for _ in 0..10 {
            view.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(300.0, 200.0)),
            );
        }
        assert_eq!(builds.get(), 1);
        view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0)),
        );
        assert_eq!(builds.get(), 2);
        view = make().into();
        tree.diff(&view);
        view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0)),
        );
        assert_eq!(builds.get(), 3);
    }
}
