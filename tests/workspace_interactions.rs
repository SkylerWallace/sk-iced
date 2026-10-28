// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "components")]
use iced_core::{
    Event, Font, Layout, Length, Point, Rectangle, Renderer as _, Shell, Size, Vector, layout,
    mouse, renderer,
    renderer::Headless as _,
    widget::{Operation, Tree},
};
use iced_widget::{
    Space,
    pane_grid::{Axis, Configuration},
};
use sk_iced::{
    animation::{Element, Renderer},
    components::*,
    motion::Policy,
};
#[derive(Clone)]
enum Model {
    Leaf(u8),
    Branch(Axis, Vec<Model>, Vec<f32>),
}
impl SplitModel for Model {
    type Key = u8;
    fn panel(&self) -> Option<u8> {
        if let Self::Leaf(key) = self {
            Some(*key)
        } else {
            None
        }
    }
    fn branch(&self) -> Option<(Axis, &[Self], &[f32])> {
        if let Self::Branch(a, c, w) = self {
            Some((*a, c, w))
        } else {
            None
        }
    }
}
#[test]
fn split_constraints_rounding_and_resize_preserve_keys_and_weight_sum() {
    let model = Model::Branch(
        Axis::Vertical,
        vec![Model::Leaf(7), Model::Leaf(9)],
        vec![0.25, 0.75],
    );
    assert_eq!(split::split_minimum_width(&model, 320.0, 6.0), 1286.0);
    let mut spec = Vec::new();
    let config = split::split_configuration(&model, Size::new(1286.0, 700.0), 6.0, &[], &mut spec);
    assert_eq!(spec, vec![(vec![], 0)]);
    let Configuration::Split { a, b, ratio, .. } = config else {
        panic!()
    };
    assert!(matches!(*a, Configuration::Pane(7)));
    assert!(matches!(*b, Configuration::Pane(9)));
    assert_eq!(ratio, 323.0 / 1286.0);
    let mut shares = [0.25, 0.75];
    split::resize_split_shares(&mut shares, 0, 1000.0, 0.5, 6.0);
    assert_eq!(shares, [0.5, 0.5]);
}
#[test]
fn dock_edges_leave_the_center_neutral_and_keep_corner_priority() {
    let bounds = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 400.0,
        height: 200.0,
    };
    assert_eq!(DockEdge::at(bounds, bounds.center()), None);
    assert_eq!(
        DockEdge::at(bounds, Point::new(10.0, 20.0)),
        Some(DockEdge::Top)
    );
    assert_eq!(
        DockEdge::at(bounds, Point::new(409.0, 120.0)),
        Some(DockEdge::Right)
    );
    assert_eq!(DockEdge::at(bounds, Point::new(9.0, 20.0)), None);
}
#[test]
fn gestures_retain_distinct_existing_thresholds_and_edge_scroll_is_tick_driven() {
    let mut gesture = DragGesture::default();
    gesture.begin(Point::ORIGIN);
    gesture.advance(Point::new(6.0, 0.0), 6.0);
    assert!(gesture.lifted);
    assert!(!DragGesture::crossed(
        Point::ORIGIN,
        Point::new(5.0, 0.0),
        5.0
    ));
    assert!(DragGesture::crossed(
        Point::ORIGIN,
        Point::new(5.01, 0.0),
        5.0
    ));
    gesture.reset();
    assert!(!gesture.lifted);
    let b = Rectangle {
        x: 20.0,
        y: 40.0,
        width: 100.0,
        height: 200.0,
    };
    assert_eq!(
        drag::edge_scroll(b, Point::new(30.0, 50.0), 24.0, 24.0),
        -24.0
    );
    assert_eq!(
        drag::edge_scroll(b, Point::new(30.0, 140.0), 24.0, 24.0),
        0.0
    );
    assert!(drag::cancel_drag(
        &Event::Window(iced_core::window::Event::Unfocused),
        false
    ));
    assert!(!drag::cancel_drag(
        &Event::Mouse(mouse::Event::CursorLeft),
        false
    ));
    assert!(drag::cancel_drag(
        &Event::Mouse(mouse::Event::CursorLeft),
        true
    ));
}
#[test]
fn nested_disjoint_scroll_clips_do_not_expose_invisible_drop_targets() {
    use iced_core::widget::operation::scrollable::{AbsoluteOffset, RelativeOffset, Scrollable};
    struct Mock;
    impl Scrollable for Mock {
        fn snap_to(&mut self, _: RelativeOffset<Option<f32>>) {}
        fn scroll_to(&mut self, _: AbsoluteOffset<Option<f32>>) {}
        fn scroll_by(&mut self, _: AbsoluteOffset, _: Rectangle, _: Rectangle) {}
    }
    let mut collect = DragRegions::<u8>::new();
    let outer = Rectangle::with_size(Size::new(100.0, 100.0));
    collect.scrollable(None, outer, outer, Vector::ZERO, &mut Mock);
    collect.traverse(&mut |op| {
        let hidden = Rectangle {
            x: 200.0,
            y: 200.0,
            ..outer
        };
        op.scrollable(None, hidden, hidden, Vector::ZERO, &mut Mock);
        op.traverse(&mut |op| op.custom(None, hidden, &mut 1u8));
        op.custom(
            None,
            Rectangle {
                x: 5.0,
                y: 5.0,
                width: 20.0,
                height: 20.0,
            },
            &mut 2u8,
        );
    });
    assert_eq!(collect.regions.len(), 1);
    assert_eq!(collect.regions[0].marker, 2);
}
#[test]
fn drag_handles_collect_geometry_without_delaying_child_actions() {
    let mut view: Element<'_, u8> = DragHandle::new(
        iced_widget::button(Space::new().width(40).height(30))
            .padding(0)
            .on_press(7),
        11u8,
    )
    .into();
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(200.0, 40.0)),
    );
    let mut regions = DragRegions::<u8>::new();
    view.as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut regions);
    assert_eq!(regions.regions[0].marker, 11);
    assert_eq!(regions.regions[0].bounds, node.bounds());
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        view.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
            &node.bounds(),
        );
    }
    assert_eq!(messages, vec![7]);
}
#[test]
fn toolbar_switches_at_measured_width_and_reclaims_height() {
    let mut view: Element<'_, ()> =
        AdaptiveToolbar::new(Space::new().width(300).height(20).into(), |compact| {
            Space::new()
                .width(Length::Fill)
                .height(if compact { 40 } else { 20 })
                .into()
        })
        .into();
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut tree = Tree::new(&view);
    for (width, height) in [(299.0, 40.0), (300.0, 20.0), (500.0, 20.0), (100.0, 40.0)] {
        let node = view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, 100.0)),
        );
        assert_eq!(node.size(), Size::new(width, height));
    }
}
#[test]
fn navigation_drawer_dismisses_input_immediately_for_every_policy() {
    for policy in [
        Policy::DISABLED,
        Policy {
            system_enabled: true,
        },
    ] {
        let make = |open| {
            NavigationDrawer::new(Space::new().width(100).height(40), open, policy, || {
                iced_widget::button(Space::new().width(40).height(30))
                    .on_press(9u8)
                    .into()
            })
            .build()
        };
        let mut view: Element<'_, u8> = make(true).into();
        let mut tree = Tree::new(&view);
        let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
        view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(100.0, 40.0)),
        );
        view = make(false).into();
        tree.diff(view.as_widget());
        let node = view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(100.0, 40.0)),
        );
        let mut messages = Vec::new();
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            view.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(10.0, 10.0)),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(&mut messages),
                &node.bounds(),
            );
        }
        assert!(messages.is_empty());
    }
}
#[test]
fn drop_indicator_matches_native_pixels_at_fractional_scale() {
    let mut renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let frame = Rectangle::with_size(Size::new(120.0, 80.0));
    let bounds = Rectangle {
        x: 10.25,
        y: 15.5,
        width: 70.0,
        height: 40.0,
    };
    let color = iced_core::Color::from_rgb8(40, 120, 210);
    renderer.reset(frame);
    DropIndicator::new(bounds, color.scale_alpha(0.18))
        .border(color, 2.0, 6.0)
        .draw(&mut renderer);
    let actual = renderer.screenshot(Size::new(150, 100), 1.25, iced_core::Color::BLACK);
    renderer.reset(frame);
    renderer.with_layer(bounds.expand(1.0), |r| {
        r.fill_quad(
            renderer::Quad {
                bounds,
                border: iced_core::Border {
                    color,
                    width: 2.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
            color.scale_alpha(0.18),
        )
    });
    assert_eq!(
        actual,
        renderer.screenshot(Size::new(150, 100), 1.25, iced_core::Color::BLACK)
    );
    assert_eq!(TabSizing::new().reserved(134.0).width(1000.0, 2), 188.0);
    assert_eq!(TabSizing::new().reserved(134.0).width(320.0, 20), 72.0);
}

#[test]
fn passive_drag_markers_do_not_schedule_frames_on_pointer_motion() {
    let mut view: Element<'_, u8> = DragHandle::new(Space::new().width(40).height(30), 11u8).into();
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(200.0, 40.0)),
    );
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(10.0, 10.0),
        }),
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
    assert!(messages.is_empty());
}
