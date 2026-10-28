// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "components")]
use iced_core::{
    Event, Font, Layout, Length, Point, Rectangle, Renderer as _, Shell, Size, Theme, Vector,
    layout, mouse, renderer, renderer::Headless as _, widget::Tree,
};
use iced_widget::{Space, column, container, row, stack, text_editor};
use sk_iced::{
    animation::{Element, Renderer},
    components::*,
    palette,
    theme::color,
};
fn appearance() -> Appearance {
    Appearance::fixed(Font::DEFAULT, palette::palette(true, false))
}
fn paint(mut view: Element<'_, u8>) -> (Vec<u8>, Size) {
    let mut tree = Tree::new(&view);
    let mut renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let bounds = Rectangle::with_size(Size::new(300.0, 100.0));
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, bounds.size()),
    );
    let mut ignored = Vec::new();
    view.as_widget_mut().update(
        &mut tree,
        &Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut ignored),
        &bounds,
    );
    renderer.reset(bounds);
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &appearance().tokens(&Theme::Dark).theme(),
        &renderer::Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    (
        renderer.screenshot(Size::new(300, 100), 1.0, iced_core::Color::BLACK),
        node.size(),
    )
}
#[test]
fn form_fields_and_sections_keep_native_geometry_and_pixels() {
    let field = FormField::new(appearance(), "Name", Space::new().width(120).height(28))
        .text_size(13)
        .into();
    let native = row![
        iced_widget::text("Name")
            .font(Font::DEFAULT)
            .size(13)
            .width(115),
        Space::new().width(120).height(28)
    ]
    .spacing(8)
    .align_y(iced_core::Alignment::Center)
    .into();
    assert_eq!(paint(field), paint(native));
    let shared = FormSection::new()
        .push(Space::new().height(12))
        .push(Space::new().height(22))
        .spacing(6)
        .into();
    let native = column![Space::new().height(12), Space::new().height(22)]
        .spacing(6)
        .into();
    assert_eq!(paint(shared), paint(native));
}
#[test]
fn feedback_and_preview_shells_keep_native_topology() {
    let t = appearance().tokens(&Theme::Dark);
    assert_eq!(
        paint(
            EmptyState::new(appearance(), "Loading…")
                .text_size(13)
                .center(Length::Fill)
                .into()
        ),
        paint(
            container(
                iced_widget::text("Loading…")
                    .font(Font::DEFAULT)
                    .size(13)
                    .color(color(t.secondary))
            )
            .center(Length::Fill)
            .into()
        )
    );
    let a: Element<'_, u8> = PreviewSurface::new(Space::new().height(24))
        .toolbar(Space::new().height(16))
        .footer(Space::new().height(10))
        .spacing(8)
        .into();
    let b: Element<'_, u8> = column![
        Space::new().height(16),
        Space::new().height(24),
        Space::new().height(10)
    ]
    .spacing(8)
    .into();
    assert_eq!(Tree::new(&a).tag, Tree::new(&b).tag);
    assert_eq!(paint(a), paint(b));
}
#[test]
fn labeled_progress_retains_split_text_geometry_and_contrast() {
    let t = appearance().tokens(&Theme::Dark);
    for value in [0.0, 0.001, 0.45, 1.0] {
        let face = |ink| {
            container(
                iced_widget::text("45%")
                    .font(Font::DEFAULT)
                    .wrapping(iced_core::text::Wrapping::None)
                    .size(10.667)
                    .color(ink),
            )
            .padding([0, 8])
            .center_y(18)
            .width(Length::Fill)
            .clip(true)
        };
        let mut native = stack![
            iced_widget::progress_bar(0.0..=1.0, value)
                .girth(18)
                .style(t.progress(t.accent)),
            face(t.contrast(t.border))
        ];
        if value > 0.0 {
            let fraction = (value * 10000.0).round() as u16;
            native = native.push(row![
                container(face(t.contrast(t.accent)))
                    .width(Length::FillPortion(fraction.max(1)))
                    .clip(true),
                Space::new().width(Length::FillPortion((10000 - fraction).max(1)))
            ]);
        }
        assert_eq!(
            paint(
                LabeledProgress::new(appearance(), value, t.accent, "45%")
                    .height(18)
                    .into()
            ),
            paint(native.width(Length::Fill).height(18).into())
        );
    }
}
#[test]
fn read_only_actions_preserve_selection_reject_edits_and_route_scroll() {
    let mut content = text_editor::Content::<Renderer>::with_text("first\nsecond");
    assert!(matches!(
        perform_read_only(&mut content, text_editor::Action::SelectAll),
        ReadOnlyAction::Applied
    ));
    assert_eq!(content.selection().as_deref(), Some("first\nsecond"));
    assert!(matches!(
        perform_read_only(
            &mut content,
            text_editor::Action::Edit(text_editor::Edit::Insert('X'))
        ),
        ReadOnlyAction::IgnoredEdit
    ));
    assert_eq!(content.text(), "first\nsecond");
    assert!(matches!(
        perform_read_only(&mut content, text_editor::Action::Scroll { lines: 4 }),
        ReadOnlyAction::Scroll(4)
    ));
}
fn dropdown(enabled: bool) -> Element<'static, u8> {
    Dropdown::new(appearance(), vec!["One", "Two"], Some("One"), |_| 7)
        .enabled(enabled)
        .width(150)
        .into()
}
#[test]
fn disabling_dropdown_closes_popup_without_replacing_child_state() {
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let limits = layout::Limits::new(Size::ZERO, Size::new(200.0, 40.0));
    let mut view = dropdown(true);
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let child_tag = tree.children[0].tag;
    let mut messages = Vec::new();
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &node.bounds(),
    );
    assert!(
        view.as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &Rectangle::with_size(Size::new(300.0, 300.0)),
                Vector::ZERO
            )
            .is_some()
    );
    view = dropdown(false);
    tree.diff(view.as_widget());
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    assert_eq!(tree.children[0].tag, child_tag);
    assert!(
        view.as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &node.bounds(),
                Vector::ZERO
            )
            .is_none()
    );
    let mut shell = Shell::new(&mut messages);
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut shell,
        &node.bounds(),
    );
    assert_eq!(
        shell.redraw_request(),
        iced_core::window::RedrawRequest::Wait
    );
    view = dropdown(true);
    tree.diff(view.as_widget());
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    assert!(
        view.as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &node.bounds(),
                Vector::ZERO
            )
            .is_none()
    );
    assert!(messages.is_empty());
}
#[test]
fn choice_grid_keeps_order_and_native_row_heights() {
    let mut grid = ChoiceGrid::new(2).spacing(4);
    for _ in 0..5 {
        grid = grid.push(Space::new().width(20).height(18));
    }
    let mut view: Element<'_, u8> = grid.into();
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(200.0, 100.0)),
    );
    assert_eq!(node.children().len(), 3);
    assert_eq!(node.children()[2].children().len(), 1);
    assert_eq!(node.size().height, 62.0);
}
#[test]
fn slider_retains_native_child_and_disabled_slider_emits_nothing() {
    let shared: Element<'_, u8> = Slider::new(0u16..=100, 50, |_| 7).step(5u16).into();
    let native: Element<'_, u8> = iced_widget::slider(0u16..=100, 50, |_| 7).step(5u16).into();
    assert_eq!(Tree::new(&shared).children[0].tag, Tree::new(&native).tag);
    assert_eq!(paint(shared), paint(native));
    let mut view: Element<'_, u8> = Slider::new(0u16..=100, 50, |_| 7).enabled(false).into();
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(200.0, 40.0)),
    );
    let mut messages = Vec::new();
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(10.0, 10.0)),
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &node.bounds(),
    );
    assert!(messages.is_empty());
}

#[test]
fn resize_grip_retains_capture_on_rebuild_and_disabling_cancels_it() {
    let make = |height, enabled| {
        ResizeGrip::new(appearance(), height)
            .limits(60.0, 400.0)
            .enabled(enabled)
            .on_resize(|_| 7u8)
            .build()
    };
    let mut view = make(150.0, true);
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let limits = layout::Limits::new(Size::ZERO, Size::new(200.0, 6.0));
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let mut messages = Vec::new();
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(20.0, 3.0)),
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &node.bounds(),
    );
    view = make(150.0, false);
    tree.diff(view.as_widget());
    view = make(150.0, true);
    tree.diff(view.as_widget());
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(20.0, -100.0),
        }),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &node.bounds(),
    );
    assert!(messages.is_empty());
}
