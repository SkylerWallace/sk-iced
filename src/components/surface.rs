// SPDX-License-Identifier: MIT
// Copyright (c) 2026 File Beholder contributors
//! Paint-only theme surfaces; child input, layout, operations and overlays stay native.
use super::Appearance;
use crate::animation::{Element, Renderer};
use iced_core::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced_core::{Event, Length, Rectangle, Size, Theme, Vector};
#[derive(Default)]
struct Pressed(bool);
#[derive(Clone, Copy)]
#[expect(
    clippy::large_enum_variant,
    reason = "Custom materials remain Copy and avoid a separate allocation per surface."
)]
pub enum SurfaceRole {
    Panel,
    SettingsCard,
    Disclosure,
    Badge,
    SearchField,
    Segment(bool),
    Button,
    Toggle(bool),
    Important,
    Danger,
    Flat,
    Input,
    Dropdown,
    Tab,
    Menu,
    Custom(crate::visual::SurfaceStyle),
}
impl SurfaceRole {
    pub fn resolve(
        self,
        t: crate::theme::Tokens,
        status: iced_widget::button::Status,
    ) -> crate::visual::SurfaceStyle {
        match self {
            Self::SettingsCard => t
                .visual
                .settings_card
                .unwrap_or(crate::visual::SurfaceStyle {
                    fill: Some(crate::visual::Fill::Solid(crate::visual::Rgba::new(
                        t.surface, 1.,
                    ))),
                    shape: Some(crate::visual::Shape::Rounded([8.; 4])),
                    border_width: Some(1.),
                    border_color: Some(crate::visual::Rgba::new(t.border, 0.65)),
                    shadows: Some([None; 4]),
                    ..Default::default()
                }),
            Self::Panel => t.visual.panel,
            Self::Disclosure => t.visual.disclosure,
            Self::Badge => t.visual.badge,
            Self::SearchField => t.visual.search_field.unwrap_or_default(),
            Self::Segment(selected) => (if selected {
                t.visual.selected_segment.unwrap_or(t.visual.important)
            } else {
                t.visual.segment.unwrap_or(t.visual.button)
            })
            .button(status),
            Self::Tab => t.visual.tab,
            Self::Menu => t.visual.menu,
            Self::Toggle(selected) => (if selected {
                t.visual.toggled_button.unwrap_or(t.visual.important)
            } else {
                t.visual.button
            })
            .button(status),
            Self::Button => t.visual.button.button(status),
            Self::Important => t.visual.important.button(status),
            Self::Danger => t.visual.danger.button(status),
            Self::Flat => t.visual.flat.button(status),
            Self::Dropdown => t.visual.dropdown.unwrap_or(t.visual.input).button(status),
            Self::Input => t.visual.input.button(status),
            Self::Custom(style) => style,
        }
    }
}
pub struct Surface<'a, M> {
    content: Element<'a, M>,
    role: SurfaceRole,
    enabled: bool,
    paint_base: bool,
    appearance: Appearance,
}
impl<M> Widget<M, Theme, Renderer> for Surface<'_, M> {
    fn tag(&self) -> iced_core::widget::tree::Tag {
        iced_core::widget::tree::Tag::of::<Pressed>()
    }
    fn state(&self) -> iced_core::widget::tree::State {
        iced_core::widget::tree::State::new(Pressed::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, t: &mut Tree) {
        t.diff_children(&[&self.content]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, t: &mut Tree, r: &Renderer, l: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut t.children[0], r, l)
    }
    fn draw(
        &self,
        t: &Tree,
        r: &mut Renderer,
        th: &Theme,
        s: &renderer::Style,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
    ) {
        let tokens = self.appearance.tokens(th);
        let status = if !self.enabled {
            iced_widget::button::Status::Disabled
        } else if t.state.downcast_ref::<Pressed>().0 && c.is_over(l.bounds()) {
            iced_widget::button::Status::Pressed
        } else if c.is_over(l.bounds()) {
            iced_widget::button::Status::Hovered
        } else {
            iced_widget::button::Status::Active
        };
        let surface = self.role.resolve(tokens, status);
        let radius = surface.shape.map(|s| s.radius()).unwrap_or(6.0.into());
        // The native child paints the first shadow. Decoration adds only the rest.

        for layer in surface
            .layers()
            .iter()
            .skip(
                if self.paint_base || matches!(self.role, SurfaceRole::Dropdown) {
                    0
                } else {
                    1
                },
            )
            .flatten()
        {
            r.fill_quad(
                renderer::Quad {
                    bounds: l.bounds(),
                    border: iced_core::Border {
                        radius,
                        ..Default::default()
                    },
                    shadow: layer.native(surface.opacity()),
                    ..Default::default()
                },
                iced_core::Color::TRANSPARENT,
            );
        }

        if self.paint_base {
            let mut style = tokens.card(false);
            surface.apply(&mut style.background, &mut style.border, &mut style.shadow);
            style.shadow = Default::default();
            r.fill_quad(
                renderer::Quad {
                    bounds: l.bounds(),
                    border: style.border,
                    ..Default::default()
                },
                style
                    .background
                    .unwrap_or(iced_core::Color::TRANSPARENT.into()),
            );
        }
        self.content
            .as_widget()
            .draw(&t.children[0], r, th, s, l, c, v);
        paint_inset(r, l.bounds(), surface);
    }
    fn update(
        &mut self,
        t: &mut Tree,
        e: &Event,
        l: Layout<'_>,
        c: mouse::Cursor,
        r: &Renderer,
        cb: &mut dyn Clipboard,
        s: &mut Shell<'_, M>,
        v: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(&mut t.children[0], e, l, c, r, cb, s, v);
        let pressed = t.state.downcast_mut::<Pressed>();
        match e {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if self.enabled && c.is_over(l.bounds()) && s.is_event_captured() =>
            {
                pressed.0 = true
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Window(iced_core::window::Event::Unfocused) => pressed.0 = false,
            _ => {}
        }
        if matches!(e, Event::Mouse(_)) {
            s.request_redraw();
        }
    }
    fn operate(&mut self, t: &mut Tree, l: Layout<'_>, r: &Renderer, o: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(&mut t.children[0], l, r, o);
    }
    fn mouse_interaction(
        &self,
        t: &Tree,
        l: Layout<'_>,
        c: mouse::Cursor,
        v: &Rectangle,
        r: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&t.children[0], l, c, v, r)
    }
    fn overlay<'b>(
        &'b mut self,
        t: &'b mut Tree,
        l: Layout<'b>,
        r: &Renderer,
        v: &Rectangle,
        tr: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut t.children[0], l, r, v, tr)
    }
}

impl<'a, M: 'a> Surface<'a, M> {
    pub fn new(appearance: Appearance, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            appearance,
            content: content.into(),
            role: SurfaceRole::Panel,
            enabled: true,
            paint_base: true,
        }
    }
    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }
    pub fn decorate(mut self) -> Self {
        self.paint_base = false;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn build(self) -> Element<'a, M> {
        Element::new(self)
    }
}
impl<'a, M: 'a> From<Surface<'a, M>> for Element<'a, M> {
    fn from(surface: Surface<'a, M>) -> Self {
        surface.build()
    }
}

pub(crate) use super::inset::paint as paint_inset;
