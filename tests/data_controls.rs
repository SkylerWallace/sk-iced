// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "components")]
use iced_core::{Event, Font, Layout, Point, Rectangle, Shell, Size, layout, mouse, widget::Tree};
use sk_iced::{
    animation::{Element, Renderer},
    components::*,
};
fn appearance() -> Appearance {
    Appearance::themed(Font::DEFAULT, sk_iced::palette::from_theme)
}

#[test]
fn selection_tracks_keys_through_reordering_ranges_and_toggle() {
    let mut selected = std::collections::BTreeSet::new();
    let mut anchor = None;
    select(
        &mut selected,
        &mut anchor,
        &[30, 10, 20, 40],
        10,
        false,
        false,
    );
    select(
        &mut selected,
        &mut anchor,
        &[30, 10, 20, 40],
        40,
        true,
        false,
    );
    assert_eq!(selected.into_iter().collect::<Vec<_>>(), vec![10, 20, 40]);
    let mut selected = [10, 20, 40].into_iter().collect();
    select(
        &mut selected,
        &mut anchor,
        &[40, 20, 10, 30],
        30,
        true,
        true,
    );
    assert_eq!(selected.len(), 4);
    select(
        &mut selected,
        &mut anchor,
        &[40, 20, 10, 30],
        30,
        false,
        true,
    );
    assert_eq!(anchor, Some(10));
    assert_eq!(adjacent(0, None, -1), None);
    assert_eq!(adjacent(4, Some(0), -1), Some(0));
    assert_eq!(adjacent(4, Some(3), 1), Some(3));
}
#[test]
fn tree_arrows_toggle_select_parent_and_handle_empty_loading_rows() {
    assert_eq!(
        tree_navigation(0, None, 1, false, None),
        TreeNavigation::None
    );
    assert_eq!(
        tree_navigation(4, Some(1), 2, false, Some(0)),
        TreeNavigation::Toggle(1)
    );
    assert_eq!(
        tree_navigation(4, Some(1), -2, true, Some(0)),
        TreeNavigation::Toggle(1)
    );
    assert_eq!(
        tree_navigation(4, Some(1), -2, false, Some(0)),
        TreeNavigation::Select(0)
    );
    assert_eq!(
        tree_navigation(4, Some(0), -2, false, None),
        TreeNavigation::None
    );
    assert_eq!(
        tree_navigation(4, Some(1), 2, true, Some(0)),
        TreeNavigation::Select(2)
    );
}
#[derive(Debug, PartialEq)]
enum Action {
    Sort(usize),
    Move(usize, usize),
    Context(Point),
    Width(f32),
}
fn header(enabled: bool) -> Element<'static, Action> {
    let row = TableRow::new()
        .push(iced_widget::Space::new().width(100).height(26))
        .push(iced_widget::Space::new().width(100).height(26));
    TableHeader::new(appearance(), row)
        .columns([HeaderResize::Trailing(6.0), HeaderResize::Leading(6.0)])
        .enabled(enabled)
        .on_sort(Action::Sort)
        .on_move(Action::Move)
        .on_context(Action::Context)
        .into()
}
fn events(
    view: &mut Element<'_, Action>,
    tree: &mut Tree,
    events: &[(Event, Point)],
) -> Vec<Action> {
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let size = Size::new(200.0, 26.0);
    let node = view
        .as_widget_mut()
        .layout(tree, &renderer, &layout::Limits::new(size, size));
    let mut messages = Vec::new();
    for (event, point) in events {
        view.as_widget_mut().update(
            tree,
            event,
            Layout::new(&node),
            mouse::Cursor::Available(*point),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(size),
        );
    }
    messages
}
fn press() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}
fn release() -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}
#[test]
fn header_rebuild_retains_drag_origin_and_disabled_header_has_no_actions() {
    let mut view = header(true);
    let mut tree = Tree::new(&view);
    assert!(events(&mut view, &mut tree, &[(press(), Point::new(50.0, 10.0))]).is_empty());
    view = header(true);
    tree.diff(view.as_widget());
    assert_eq!(
        events(
            &mut view,
            &mut tree,
            &[
                (
                    Event::Mouse(mouse::Event::CursorMoved {
                        position: Point::new(150.0, 10.0)
                    }),
                    Point::new(150.0, 10.0)
                ),
                (release(), Point::new(150.0, 10.0))
            ]
        ),
        vec![Action::Move(0, 1)]
    );
    assert_eq!(
        events(
            &mut view,
            &mut tree,
            &[
                (press(), Point::new(50.0, 10.0)),
                (release(), Point::new(50.0, 10.0))
            ]
        ),
        vec![Action::Sort(0)]
    );
    view = header(false);
    tree.diff(view.as_widget());
    assert!(
        events(
            &mut view,
            &mut tree,
            &[
                (press(), Point::new(50.0, 10.0)),
                (release(), Point::new(50.0, 10.0))
            ]
        )
        .is_empty()
    );
}
#[test]
fn header_divider_and_focus_loss_do_not_trigger_sort_or_reorder() {
    let mut view = header(true);
    let mut tree = Tree::new(&view);
    let p = Point::new(98.0, 10.0);
    assert!(events(&mut view, &mut tree, &[(press(), p), (release(), p)]).is_empty());
    assert!(
        events(
            &mut view,
            &mut tree,
            &[
                (press(), Point::new(50.0, 10.0)),
                (Event::Window(iced_core::window::Event::Unfocused), p),
                (release(), p)
            ]
        )
        .is_empty()
    );
}
#[test]
fn tree_rows_and_tables_keep_native_layout_and_widget_tags() {
    let shared: Element<'_, ()> = TreeRow::new(
        iced_widget::Space::new().width(24),
        iced_widget::Space::new().width(100),
    )
    .depth(100)
    .height(21)
    .into();
    let native: Element<'_, ()> = iced_widget::row![
        iced_widget::Space::new().width(168),
        iced_widget::Space::new().width(24),
        iced_widget::Space::new().width(100)
    ]
    .height(21)
    .into();
    assert_eq!(Tree::new(&shared).tag, Tree::new(&native).tag);
    let mut shared = shared;
    let mut native = native;
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let limits = layout::Limits::new(Size::ZERO, Size::new(500.0, 100.0));
    let mut ta = Tree::new(&shared);
    let a = shared.as_widget_mut().layout(&mut ta, &renderer, &limits);
    let mut tb = Tree::new(&native);
    let b = native.as_widget_mut().layout(&mut tb, &renderer, &limits);
    assert_eq!(a.size(), b.size());
    assert_eq!(
        a.children().iter().map(|n| n.bounds()).collect::<Vec<_>>(),
        b.children().iter().map(|n| n.bounds()).collect::<Vec<_>>()
    );
}

#[test]
fn resize_retains_origin_on_rebuild_and_clamps_before_publishing() {
    let make = |width| {
        ColumnResize::new(appearance(), width)
            .limits(40.0, 180.0)
            .leading(true)
            .on_resize(Action::Width)
            .build()
    };
    let mut view = make(100.0);
    let mut tree = Tree::new(&view);
    events(&mut view, &mut tree, &[(press(), Point::new(3.0, 10.0))]);
    let p = Point::new(-37.0, 10.0);
    assert_eq!(
        events(
            &mut view,
            &mut tree,
            &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]
        ),
        vec![Action::Width(140.0)]
    );
    view = make(140.0);
    tree.diff(view.as_widget());
    let p = Point::new(-237.0, 10.0);
    assert_eq!(
        events(
            &mut view,
            &mut tree,
            &[(Event::Mouse(mouse::Event::CursorMoved { position: p }), p)]
        ),
        vec![Action::Width(180.0)]
    );
}

#[test]
fn hidden_columns_keep_identity_and_position_when_reordered_and_restored() {
    let mut columns = TableColumns::new([1, 2, 3, 4]).hidden([2]);
    columns.move_to(2, 4);
    assert_eq!(columns.visible(), vec![1, 3, 4]);
    columns.toggle(2);
    assert_eq!(columns.visible(), vec![1, 3, 4, 2]);
    columns.move_to(99, 1);
    assert_eq!(columns.order, vec![1, 3, 4, 2]);
    let mut empty = TableColumns::<u8>::new([]);
    empty.move_to(1, 2);
    assert!(empty.order.is_empty());
}
