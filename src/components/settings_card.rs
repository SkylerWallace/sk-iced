// SPDX-License-Identifier: Apache-2.0
//! Responsive setting cards with stable label/control trees.
use super::{Appearance, Surface, SurfaceRole};
use crate::{
    animation::{Element, Renderer},
    theme::color,
};
use iced_core::{
    Clipboard, Event, Layout, Length, Rectangle, Shell, Size, Theme, Vector, Widget, layout, mouse,
    overlay, renderer,
    widget::{Operation, Tree},
};
use iced_widget::column;
use std::borrow::Cow;
pub struct SettingsCard<'a, M> {
    appearance: Appearance,
    label: Element<'a, M>,
    control: Element<'a, M>,
    description: Option<Cow<'a, str>>,
    padding: f32,
    gap: f32,
    stacked: bool,
}
impl<'a, M: 'a> SettingsCard<'a, M> {
    pub fn new(
        appearance: Appearance,
        title: impl Into<Cow<'a, str>>,
        control: impl Into<Element<'a, M>>,
    ) -> Self {
        let title = super::text(appearance.font, title.into())
            .style(move |theme| iced_widget::text::Style {
                color: Some(color(appearance.tokens(theme).text)),
            })
            .size(14)
            .font(iced_core::Font {
                weight: iced_core::font::Weight::Semibold,
                ..appearance.font
            });
        Self {
            appearance,
            label: title.into(),
            control: control.into(),
            description: None,
            padding: 16.,
            gap: 16.,
            stacked: false,
        }
    }
    pub fn description(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.);
        self
    }
    pub fn build(self) -> Element<'a, M> {
        let appearance = self.appearance;
        let mut labels = column![self.label].spacing(5).width(Length::Fill);
        if let Some(description) = self.description {
            labels = labels.push(super::text(appearance.font, description).size(12.5).style(
                move |theme| iced_widget::text::Style {
                    color: Some(color(appearance.tokens(theme).secondary)),
                },
            ));
        }
        Surface::new(
            appearance,
            Element::new(CardBody {
                label: labels.into(),
                control: self.control,
                padding: self.padding,
                gap: self.gap,
                stacked: self.stacked,
            }),
        )
        .role(SurfaceRole::SettingsCard)
        .into()
    }
}
impl<'a, M: 'a> From<SettingsCard<'a, M>> for Element<'a, M> {
    fn from(card: SettingsCard<'a, M>) -> Self {
        card.build()
    }
}
struct CardBody<'a, M> {
    label: Element<'a, M>,
    control: Element<'a, M>,
    padding: f32,
    gap: f32,
    stacked: bool,
}
impl<M> Widget<M, Theme, Renderer> for CardBody<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.label), Tree::new(&self.control)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.label, &self.control]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, limits: &layout::Limits) -> layout::Node {
        let width = limits.max().width;
        let available = (width - self.padding * 2.).max(0.);
        let mut control = self.control.as_widget_mut().layout(
            &mut t.children[1],
            r,
            &layout::Limits::new(Size::ZERO, Size::new(available, f32::INFINITY)),
        );
        let stacked = self.stacked || available < 440. || control.size().width > available * 0.55;
        let label_width = if stacked {
            available
        } else {
            (available - control.size().width - self.gap).max(0.)
        };
        let mut label = self.label.as_widget_mut().layout(
            &mut t.children[0],
            r,
            &layout::Limits::new(Size::ZERO, Size::new(label_width, f32::INFINITY)),
        );
        let height = if stacked {
            let h = label.size().height + self.gap + control.size().height;
            label = label.move_to(iced_core::Point::new(self.padding, self.padding));
            control = control.move_to(iced_core::Point::new(
                self.padding,
                self.padding + label.size().height + self.gap,
            ));
            h
        } else {
            let h = label.size().height.max(control.size().height);
            let ly = self.padding + (h - label.size().height) / 2.;
            let cy = self.padding + (h - control.size().height) / 2.;
            label = label.move_to(iced_core::Point::new(self.padding, ly));
            let x = width - self.padding - control.size().width;
            control = control.move_to(iced_core::Point::new(x, cy));
            h
        };
        layout::Node::with_children(
            Size::new(width, height + self.padding * 2.),
            vec![label, control],
        )
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut layouts = l.children();
        self.label.as_widget().draw(
            &t.children[0],
            r,
            theme,
            style,
            layouts.next().unwrap(),
            cursor,
            viewport,
        );
        self.control.as_widget().draw(
            &t.children[1],
            r,
            theme,
            style,
            layouts.next().unwrap(),
            cursor,
            viewport,
        );
    }
    fn update(
        &mut self,
        t: &mut Tree,
        event: &Event,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        r: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let mut layouts = l.children();
        self.label.as_widget_mut().update(
            &mut t.children[0],
            event,
            layouts.next().unwrap(),
            cursor,
            r,
            clipboard,
            shell,
            viewport,
        );
        self.control.as_widget_mut().update(
            &mut t.children[1],
            event,
            layouts.next().unwrap(),
            cursor,
            r,
            clipboard,
            shell,
            viewport,
        );
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, op: &mut dyn Operation) {
        let mut layouts = l.children();
        op.traverse(&mut |op| {
            self.label
                .as_widget_mut()
                .operate(&mut t.children[0], layouts.next().unwrap(), r, op);
            self.control.as_widget_mut().operate(
                &mut t.children[1],
                layouts.next().unwrap(),
                r,
                op,
            );
        });
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        let mut layouts = l.children();
        self.label
            .as_widget()
            .mouse_interaction(&t.children[0], layouts.next().unwrap(), cursor, viewport, r)
            .max(self.control.as_widget().mouse_interaction(
                &t.children[1],
                layouts.next().unwrap(),
                cursor,
                viewport,
                r,
            ))
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        let mut layouts = l.children();
        let (label_tree, control_tree) = t.children.split_at_mut(1);
        let overlays = vec![
            self.label.as_widget_mut().overlay(
                &mut label_tree[0],
                layouts.next().unwrap(),
                r,
                viewport,
                translation,
            ),
            self.control.as_widget_mut().overlay(
                &mut control_tree[0],
                layouts.next().unwrap(),
                r,
                viewport,
                translation,
            ),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if overlays.is_empty() {
            None
        } else {
            Some(overlay::Group::with_children(overlays).overlay())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn responsive_cards_keep_control_state_and_bounds() {
        let appearance = Appearance::fixed(
            iced_core::Font::DEFAULT,
            crate::palette::palette(true, false),
        );
        let make = || {
            SettingsCard::<u8>::new(
                appearance,
                "Directory sizes",
                iced_widget::text_input("Value", "Preserved").width(198),
            )
            .description("Choose how directory sizes are calculated.")
            .build()
        };
        let mut card = make();
        let mut tree = Tree::new(&card);
        let renderer = Renderer::new(iced_core::Font::DEFAULT, 13.0.into());
        type InputState =
            iced_widget::text_input::State<<Renderer as iced_core::text::Renderer>::Paragraph>;
        tree.children[0].children[1]
            .state
            .downcast_mut::<InputState>()
            .focus();
        let mut wide_height = 0.;
        for width in [580., 340., 580.] {
            card = make();
            tree.diff(card.as_widget());
            let node = card.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width, 1000.)),
            );
            assert_eq!(node.size().width, width);
            if width == 340. {
                assert!(node.size().height > wide_height);
            } else {
                wide_height = node.size().height;
            }
            assert_eq!(tree.children[0].children.len(), 2);
            assert!(
                tree.children[0].children[1]
                    .state
                    .downcast_ref::<InputState>()
                    .is_focused()
            );
            assert!(node.size().height < 200.);
        }
    }
}
