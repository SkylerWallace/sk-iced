#![cfg(feature = "components")]
use iced_core::{
    Event, Font, Layout, Length, Point, Rectangle, Renderer as _, Shell, Size, layout, mouse,
    renderer, renderer::Headless as _, widget::Tree,
};
use iced_widget::{Space, button, container};
use sk_iced::{
    animation::{Element, Renderer},
    components::*,
    motion::Policy,
    palette,
};
fn paint(mut widget: Element<'_, u8>, theme: &iced_core::Theme, hover: bool) -> (String, Vec<u8>) {
    let mut tree = Tree::new(&widget);
    let mut renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let size = Size::new(240.0, 80.0);
    let node =
        widget
            .as_widget_mut()
            .layout(&mut tree, &renderer, &layout::Limits::new(Size::ZERO, size));
    let cursor = if hover {
        mouse::Cursor::Available(Point::new(12.0, 12.0))
    } else {
        mouse::Cursor::Unavailable
    };
    let mut messages = vec![];
    widget.as_widget_mut().update(
        &mut tree,
        &Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
        Layout::new(&node),
        cursor,
        &renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(&mut messages),
        &Rectangle::with_size(size),
    );
    renderer.reset(Rectangle::with_size(size));
    widget.as_widget().draw(
        &tree,
        &mut renderer,
        theme,
        &renderer::Style::default(),
        Layout::new(&node),
        cursor,
        &Rectangle::with_size(size),
    );
    (
        format!("{node:?}"),
        renderer.screenshot(Size::new(240, 80), 1.0, iced_core::Color::BLACK),
    )
}
#[test]
fn cards_preserve_inherited_fill_height_and_clipping() {
    let t = palette::palette(true, false);
    let appearance = Appearance::fixed(Font::DEFAULT, t);
    let theme = t.theme();
    let new = card(
        appearance,
        Space::new().height(Length::Fill).width(Length::Fill),
        CardLayout::default(),
    )
    .into();
    let old = container(Space::new().height(Length::Fill).width(Length::Fill))
        .padding(1)
        .clip(true)
        .width(Length::Fill)
        .style(move |_| t.card(false))
        .into();
    assert_eq!(paint(new, &theme, false), paint(old, &theme, false));
}
#[test]
fn shared_actions_dispatch_immediately_and_preserve_pressed_state_on_rebuild() {
    for policy in [
        Policy {
            system_enabled: true,
        },
        Policy::DISABLED,
    ] {
        let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
        let make = || -> Element<'static, u8> {
            action(
                appearance,
                policy,
                "Action",
                false,
                false,
                ActionLayout::default(),
            )
            .on_press(7)
            .into()
        };
        let mut widget = make();
        let mut tree = Tree::new(&widget);
        let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
        let node = widget.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
        );
        let mut messages = vec![];
        widget.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(12.0, 12.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
            &node.bounds(),
        );
        widget = make();
        tree.diff(&widget);
        let node = widget.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
        );
        widget.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(12.0, 12.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
            &node.bounds(),
        );
        assert_eq!(messages, vec![7]);
    }
}

fn segment_event(
    widget: &mut Element<'_, u8>,
    tree: &mut Tree,
    renderer: &Renderer,
    messages: &mut Vec<u8>,
    event: Event,
    point: Point,
) -> layout::Node {
    let node = widget.as_widget_mut().layout(
        tree,
        renderer,
        &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
    );
    widget.as_widget_mut().update(
        tree,
        &event,
        Layout::new(&node),
        mouse::Cursor::Available(point),
        renderer,
        &mut iced_core::clipboard::Null,
        &mut Shell::new(messages),
        &Rectangle::with_size(Size::new(240.0, 80.0)),
    );
    node
}

#[test]
fn segmented_commands_are_immediate_and_gated_across_rebuilds_and_motion_policies() {
    for policy in [
        Policy::DISABLED,
        Policy {
            system_enabled: true,
        },
        Policy::DISABLED,
    ] {
        for mode in 0..5 {
            let make = |enabled| -> Element<'static, u8> {
                let spec = Segmented::new(
                    Appearance::fixed(Font::DEFAULT, palette::palette(true, false)),
                    policy,
                )
                .option(Segment::new("First"))
                .option(Segment::new("Second").enabled(mode != 2))
                .width(198)
                .selected(1)
                .enabled(enabled);
                if mode == 4 {
                    spec.into()
                } else {
                    spec.on_select_maybe(move |i| (mode != 3).then_some(i as u8))
                        .into()
                }
            };
            let mut widget = make(mode != 1);
            let mut tree = Tree::new(&widget);
            let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
            let mut messages = vec![];
            let point = Point::new(140.0, 14.0);
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                point,
            );
            widget = make(mode != 1);
            tree.diff(&widget);
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                point,
            );
            assert_eq!(messages, if mode == 0 { vec![1] } else { vec![] });
            // A control disabled between press and release must not dispatch.
            widget = make(true);
            tree.diff(&widget);
            messages.clear();
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                point,
            );
            widget = make(false);
            tree.diff(&widget);
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                point,
            );
            assert!(messages.is_empty());
        }
    }
}

#[test]
fn segmented_defaults_sizing_and_empty_options_are_valid() {
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    for empty in [false, true] {
        for custom in [false, true] {
            let mut spec = Segmented::<u8>::new(appearance, Policy::DISABLED);
            if !empty {
                spec = spec.labels(["One", "Two"]);
            }
            if custom {
                spec = spec.width(150).height(36.0).inset(4.0).selection(None);
            }
            let mut widget: Element<'_, u8> = spec.into();
            let mut tree = Tree::new(&widget);
            let mut messages = vec![];
            let node = segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Point::new(10.0, 10.0),
            );
            assert_eq!(
                node.size(),
                if custom {
                    Size::new(150.0, 36.0)
                } else {
                    Size::new(240.0, 28.0)
                }
            );
            assert!(messages.is_empty());
        }
    }
}

#[test]
fn optional_segment_tooltip_uses_an_overlay_without_affecting_layout_or_clicks() {
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    for has_tooltip in [false, true] {
        let option = Segment::new("One");
        let option = if has_tooltip {
            option.tooltip("Helpful information")
        } else {
            option
        };
        let mut widget: Element<'_, u8> = Segmented::new(appearance, Policy::DISABLED)
            .option(option)
            .width(198)
            .tooltip_delay(std::time::Duration::ZERO)
            .on_select(|i| i as u8)
            .into();
        let mut tree = Tree::new(&widget);
        let mut messages = vec![];
        let now = std::time::Instant::now();
        let point = Point::new(50.0, 14.0);
        for instant in [now, now + std::time::Duration::from_millis(1)] {
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Window(iced_core::window::Event::RedrawRequested(instant)),
                point,
            );
        }
        let node = segment_event(
            &mut widget,
            &mut tree,
            &renderer,
            &mut messages,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            point,
        );
        segment_event(
            &mut widget,
            &mut tree,
            &renderer,
            &mut messages,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            point,
        );
        assert_eq!(node.size(), Size::new(198.0, 28.0));
        assert_eq!(messages, vec![0]);
        let viewport = Rectangle::with_size(Size::new(240.0, 80.0));
        let overlay = widget.as_widget_mut().overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &viewport,
            iced_core::Vector::ZERO,
        );
        assert_eq!(overlay.is_some(), has_tooltip);
    }
}

#[test]
fn shared_inputs_and_settings_rows_preserve_native_geometry_and_pixels() {
    for dark in [false, true] {
        let t = palette::palette(dark, false);
        let appearance = Appearance::fixed(Font::DEFAULT, t);
        for enabled in [false, true] {
            for hover in [false, true] {
                for checked in [false, true] {
                    let new = Checkbox::new(appearance, checked)
                        .label("Example")
                        .enabled(enabled)
                        .on_toggle(|_| 7)
                        .into();
                    let old = iced_widget::checkbox(checked)
                        .font(Font::DEFAULT)
                        .label("Example")
                        .size(15)
                        .spacing(8)
                        .text_size(13.333)
                        .style(t.check())
                        .on_toggle_maybe(enabled.then_some(|_| 7))
                        .into();
                    assert_eq!(paint(new, &t.theme(), hover), paint(old, &t.theme(), hover));
                }
                let new = TextInput::new(appearance, "Placeholder", "Example")
                    .padding(8)
                    .size(13)
                    .enabled(enabled)
                    .on_input(|_| 7)
                    .into();
                let old = themed_text_input(appearance, "Placeholder", "Example")
                    .padding(8)
                    .size(13)
                    .on_input_maybe(enabled.then_some(|_| 7))
                    .into();
                assert_eq!(paint(new, &t.theme(), hover), paint(old, &t.theme(), hover));
            }
        }
        let new = SettingsRow::new(appearance, "Theme", Space::new().width(80).height(28))
            .hint("Follow Windows appearance")
            .into();
        let old = iced_widget::column![
            iced_widget::row![
                iced_widget::text("Theme")
                    .font(Font::DEFAULT)
                    .size(13.333)
                    .width(Length::Fill),
                Space::new().width(80).height(28)
            ]
            .align_y(iced_core::Alignment::Center),
            iced_widget::text("Follow Windows appearance")
                .font(Font::DEFAULT)
                .size(11)
                .color(sk_iced::theme::color(t.secondary))
        ]
        .spacing(4)
        .into();
        assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        let new = SettingsRow::content(appearance, Space::new().width(100).height(16))
            .height(30)
            .into();
        let old = container(Space::new().width(100).height(16))
            .center_y(30)
            .into();
        assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        let new = Checkbox::with_state(appearance, CheckState::Mixed)
            .size(16)
            .into();
        let old = tri_state_checkbox(Font::DEFAULT, CheckState::Mixed)
            .size(16)
            .spacing(8)
            .text_size(13.333)
            .style(t.check())
            .into();
        assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
    }
}

#[test]
fn checkbox_actions_remain_immediate_and_enabled_is_independent_of_setter_order() {
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    for enabled in [false, true] {
        for state in [
            CheckState::Unchecked,
            CheckState::Checked,
            CheckState::Mixed,
        ] {
            let mut widget: Element<'_, u8> = Checkbox::with_state(appearance, state)
                .enabled(enabled)
                .on_toggle(|v| if v { 1 } else { 0 })
                .into();
            let mut tree = Tree::new(&widget);
            let mut messages = vec![];
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Point::new(7.0, 7.0),
            );
            let expected = if !enabled {
                vec![]
            } else if matches!(state, CheckState::Unchecked) {
                vec![1]
            } else {
                vec![0]
            };
            assert_eq!(messages, expected);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InputMessage {
    Edited(String),
    Pasted(String),
    Submitted,
}
struct TestClipboard;
impl iced_core::Clipboard for TestClipboard {
    fn read(&self, _: iced_core::clipboard::Kind) -> Option<String> {
        Some("paste".into())
    }
    fn write(&mut self, _: iced_core::clipboard::Kind, _: String) {}
}
fn input_key(key: iced_core::keyboard::Key, text: Option<&str>) -> Event {
    Event::Keyboard(iced_core::keyboard::Event::KeyPressed {
        modified_key: key.clone(),
        key,
        physical_key: iced_core::keyboard::key::Physical::Unidentified(
            iced_core::keyboard::key::NativeCode::Unidentified,
        ),
        location: iced_core::keyboard::Location::Standard,
        modifiers: iced_core::keyboard::Modifiers::default(),
        text: text.map(Into::into),
        repeat: false,
    })
}
fn input_update(
    widget: &mut Element<'_, InputMessage>,
    tree: &mut Tree,
    renderer: &Renderer,
    event: Event,
    messages: &mut Vec<InputMessage>,
) {
    let node = widget.as_widget_mut().layout(
        tree,
        renderer,
        &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
    );
    widget.as_widget_mut().update(
        tree,
        &event,
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        renderer,
        &mut TestClipboard,
        &mut Shell::new(messages),
        &node.bounds(),
    );
}

#[test]
fn text_input_retains_focus_selection_and_native_tag_and_gates_all_actions() {
    use iced_core::keyboard::{Key, key::Named};
    type InputState =
        iced_widget::text_input::State<<Renderer as iced_core::text::Renderer>::Paragraph>;
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    let make = |enabled| -> Element<'static, InputMessage> {
        TextInput::new(appearance, "Placeholder", "hello")
            .id("shared-input")
            .enabled(enabled)
            .on_input(InputMessage::Edited)
            .on_paste(InputMessage::Pasted)
            .on_submit(InputMessage::Submitted)
            .into()
    };
    for enabled in [false, true] {
        let mut widget = make(true);
        let mut tree = Tree::new(&widget);
        let native: Element<'_, InputMessage> =
            themed_text_input(appearance, "Placeholder", "hello").into();
        assert_eq!(tree.tag, Tree::new(&native).tag);
        let state = tree.state.downcast_mut::<InputState>();
        state.focus();
        state.select_range(1, 4);
        let cursor = state.cursor();
        widget = make(enabled);
        tree.diff(&widget);
        let state = tree.state.downcast_ref::<InputState>();
        assert!(state.is_focused());
        assert_eq!(state.cursor(), cursor);
        let mut messages = vec![];
        input_update(
            &mut widget,
            &mut tree,
            &renderer,
            input_key(Key::Character("X".into()), Some("X")),
            &mut messages,
        );
        input_update(
            &mut widget,
            &mut tree,
            &renderer,
            input_key(Key::Named(Named::Enter), None),
            &mut messages,
        );
        assert_eq!(
            messages,
            if enabled {
                vec![InputMessage::Edited("hXo".into()), InputMessage::Submitted]
            } else {
                vec![]
            }
        );
        // A focused input must stop paste and submit immediately when disabled.
        widget = make(false);
        tree.diff(&widget);
        messages.clear();
        input_update(
            &mut widget,
            &mut tree,
            &renderer,
            Event::Keyboard(iced_core::keyboard::Event::ModifiersChanged(
                iced_core::keyboard::Modifiers::CTRL,
            )),
            &mut messages,
        );
        input_update(
            &mut widget,
            &mut tree,
            &renderer,
            input_key(Key::Character("v".into()), None),
            &mut messages,
        );
        input_update(
            &mut widget,
            &mut tree,
            &renderer,
            input_key(Key::Named(Named::Enter), None),
            &mut messages,
        );
        assert!(messages.is_empty());
    }
}

#[test]
fn shared_tooltip_defaults_preserve_overlay_geometry_and_pixels() {
    let t = palette::palette(true, false);
    let appearance = Appearance::fixed(Font::DEFAULT, t);
    let mut baseline = None;
    for shared in [false, true] {
        // Store the comparison from the native implementation first.
        let mut widget: Element<'_, u8> = if shared {
            Tooltip::new(appearance, Space::new().width(80).height(28), "Hint")
                .position(iced_widget::tooltip::Position::Bottom)
                .delay(std::time::Duration::ZERO)
                .into()
        } else {
            iced_widget::tooltip(
                Space::new().width(80).height(28),
                container(iced_widget::text("Hint").font(Font::DEFAULT).size(12))
                    .padding(7)
                    .style(move |_| t.card(false)),
                iced_widget::tooltip::Position::Bottom,
            )
            .into()
        };
        let mut tree = Tree::new(&widget);
        let mut renderer = Renderer::new(Font::DEFAULT, 13.0.into());
        let mut messages = vec![];
        let node = segment_event(
            &mut widget,
            &mut tree,
            &renderer,
            &mut messages,
            Event::Window(iced_core::window::Event::RedrawRequested(
                std::time::Instant::now(),
            )),
            Point::new(10.0, 10.0),
        );
        let viewport = Rectangle::with_size(Size::new(240.0, 80.0));
        let mut overlay = widget
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &viewport,
                iced_core::Vector::ZERO,
            )
            .unwrap();
        let layout = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        renderer.reset(viewport);
        overlay.as_overlay().draw(
            &mut renderer,
            &t.theme(),
            &renderer::Style::default(),
            Layout::new(&layout),
            mouse::Cursor::Unavailable,
        );
        let pixels = renderer.screenshot(Size::new(240, 80), 1.0, iced_core::Color::BLACK);
        if shared {
            assert_eq!(baseline.as_ref().unwrap(), &(format!("{layout:?}"), pixels));
        } else {
            baseline = Some((format!("{layout:?}"), pixels));
        }
    }
}

#[test]
fn navigation_preserves_native_inherited_fill_sizing_on_both_axes() {
    for vertical in [false, true] {
        let child = || Space::new().width(Length::Fill).height(Length::Fill);
        let new: Element<'_, u8> = if vertical {
            Navigation::vertical()
                .push(Space::new().height(18))
                .push(child())
                .spacing(8)
                .into()
        } else {
            Navigation::horizontal()
                .push(Space::new().width(18))
                .push(child())
                .spacing(8)
                .into()
        };
        let old = if vertical {
            iced_widget::column![Space::new().height(18), child()]
                .spacing(8)
                .into()
        } else {
            iced_widget::row![Space::new().width(18), child()]
                .spacing(8)
                .into()
        };
        assert_eq!(
            paint(new, &iced_core::Theme::Dark, false),
            paint(old, &iced_core::Theme::Dark, false)
        );
    }
}

#[test]
fn panel_badge_progress_and_dropdown_builders_match_native_profiles() {
    for dark in [false, true] {
        let t = palette::palette(dark, false);
        let appearance = Appearance::fixed(Font::DEFAULT, t);
        let new = Panel::new(appearance, Space::new().height(30)).into();
        let old = card(appearance, Space::new().height(30), CardLayout::default()).into();
        assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        for width in [64, 72] {
            let new = StatusBadge::new(appearance, "Passed", t.success)
                .width(width)
                .into();
            let old = container(
                iced_widget::text("Passed")
                    .font(Font::DEFAULT)
                    .size(10.667)
                    .color(sk_iced::theme::color(t.success)),
            )
            .center_x(width)
            .center_y(22)
            .style(move |_| container::Style {
                background: Some(
                    sk_iced::theme::blend(t.surface, t.success, t.opacity_selected).into(),
                ),
                border: iced_core::Border {
                    radius: 11.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into();
            assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        }
        for value in [0.0, 45.0, 100.0] {
            let new = Progress::new(appearance, 0.0..=100.0, value, t.accent)
                .girth(20)
                .into();
            let old = iced_widget::progress_bar(0.0..=100.0, value)
                .girth(20)
                .style(t.progress(t.accent))
                .into();
            assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        }
        for selected in [None, Some("Light")] {
            let new = Dropdown::new(appearance, vec!["Light", "Dark"], selected, |_| 7)
                .width(150)
                .padding(8)
                .into();
            let old = iced_widget::pick_list(vec!["Light", "Dark"], selected, |_| 7)
                .font(Font::DEFAULT)
                .text_size(13.333)
                .width(150)
                .padding(8)
                .into();
            assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        }
    }
}

#[test]
fn tab_close_is_independent_from_selection_and_tabbar_uses_stable_keyed_layout() {
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    for close in [false, true] {
        let title = Button::new(appearance, Policy::DISABLED)
            .text("Folder")
            .width(80)
            .on_press(1);
        let mut widget: Element<'_, u8> = Tab::custom(title)
            .close(appearance, Policy::DISABLED, Some(2))
            .into();
        let mut tree = Tree::new(&widget);
        let mut messages = vec![];
        let point = Point::new(if close { 90.0 } else { 10.0 }, 15.0);
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            segment_event(
                &mut widget,
                &mut tree,
                &renderer,
                &mut messages,
                Event::Mouse(event),
                point,
            );
        }
        assert_eq!(messages, vec![if close { 2 } else { 1 }]);
    }
    let items = || {
        vec![sk_iced::animation::keyed::Item {
            key: sk_iced::animation::keyed::Key::Item(12),
            content: Space::new().width(80).height(30).into(),
            snapshot: Some(12u8),
        }]
    };
    let new = TabBar::new(Policy::DISABLED, |_, _| {
        Space::new().width(80).height(30).into()
    })
    .items(items())
    .into();
    let old = sk_iced::animation::keyed::view(items(), Policy::DISABLED, |_, _| {
        Space::new().width(80).height(30).into()
    });
    assert_eq!(
        paint(new, &iced_core::Theme::Dark, false),
        paint(old, &iced_core::Theme::Dark, false)
    );
}

#[test]
fn expandable_card_and_custom_dialog_preserve_original_structure() {
    let t = palette::palette(true, false);
    let appearance = Appearance::fixed(Font::DEFAULT, t);
    for open in [false, true] {
        let new = ExpandableCard::new(appearance, Policy::DISABLED, Space::new().height(28))
            .expanded(open)
            .body(|| Space::new().height(32).width(Length::Fill).into())
            .into();
        let old = card(
            appearance,
            iced_widget::column![
                Space::new().height(28),
                sk_iced::animation::reveal::vertical(open, Policy::DISABLED, || Space::new()
                    .height(32)
                    .width(Length::Fill)
                    .into())
            ],
            CardLayout::default(),
        )
        .into();
        assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
    }
    let backdrop = || container(Space::new().width(80).height(40)).center(Length::Fill);
    let new = Dialog::custom(Space::new().height(80), backdrop());
    let old = iced_widget::stack![Space::new().height(80), iced_widget::opaque(backdrop())].into();
    assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
}

#[test]
fn popup_keyboard_dismissal_pointer_placement_and_context_cursor_are_preserved() {
    let mut widget: Element<'_, u8> = Popup::new(
        Space::new().width(240).height(80),
        container(Space::new()).width(120).height(40),
    )
    .open(true)
    .at(Point::new(30.0, 20.0))
    .capture_outside(true)
    .on_dismiss(7)
    .on_key(|key| {
        (*key == iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::ArrowDown))
            .then_some(9)
    })
    .into();
    let mut tree = Tree::new(&widget);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let viewport = Rectangle::with_size(Size::new(240.0, 80.0));
    let node = widget.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    let mut overlay = widget
        .as_widget_mut()
        .overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &viewport,
            iced_core::Vector::ZERO,
        )
        .unwrap();
    let popup = overlay.as_overlay_mut().layout(&renderer, viewport.size());
    assert_eq!(
        popup.bounds(),
        Rectangle {
            x: 30.0,
            y: 20.0,
            width: 120.0,
            height: 40.0
        }
    );
    assert_eq!(
        overlay.as_overlay().mouse_interaction(
            Layout::new(&popup),
            mouse::Cursor::Available(Point::new(50.0, 30.0)),
            &renderer
        ),
        mouse::Interaction::Idle
    );
    for (event, expected) in [
        (
            input_key(
                iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::ArrowDown),
                None,
            ),
            9,
        ),
        (
            input_key(
                iced_core::keyboard::Key::Named(iced_core::keyboard::key::Named::Escape),
                None,
            ),
            7,
        ),
        (
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            7,
        ),
    ] {
        let mut messages = vec![];
        let mut shell = Shell::new(&mut messages);
        overlay.as_overlay_mut().update(
            &event,
            Layout::new(&popup),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
        );
        assert!(shell.is_event_captured());
        assert_eq!(messages, vec![expected]);
    }
}

#[test]
fn searchable_dropdown_retains_external_search_state_and_emits_input_immediately() {
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    let state = iced_widget::combo_box::State::new(vec!["Light".to_string(), "Dark".to_string()]);
    let mut widget: Element<'_, u8> =
        SearchableDropdown::new(appearance, &state, "Search", None, |_| 7)
            .on_input(|_| 9)
            .width(150)
            .into();
    let mut tree = Tree::new(&widget);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut messages = vec![];
    segment_event(
        &mut widget,
        &mut tree,
        &renderer,
        &mut messages,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Point::new(10.0, 10.0),
    );
    messages.clear();
    segment_event(
        &mut widget,
        &mut tree,
        &renderer,
        &mut messages,
        input_key(iced_core::keyboard::Key::Character("D".into()), Some("D")),
        Point::new(10.0, 10.0),
    );
    assert_eq!(messages, vec![9]);
    assert_eq!(state.options(), &["Light".to_string(), "Dark".to_string()]);
}

#[derive(Default)]
struct ScrollProbe {
    offset: Option<iced_core::Vector>,
    metrics: Option<(f32, f32)>,
}
impl iced_core::widget::Operation for ScrollProbe {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn iced_core::widget::Operation)) {
        f(self);
    }
    fn scrollable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        _: Rectangle,
        _: Rectangle,
        offset: iced_core::Vector,
        _: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        self.offset = Some(offset);
    }
    fn custom(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        _: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if let Some(rail) = state.downcast_ref::<sk_iced::components::scroll::ScrollRailState>() {
            self.metrics = rail.metrics();
        }
    }
}
#[test]
fn scroll_area_and_scrollbar_profiles_match_native_geometry_and_pixels() {
    let t = palette::palette(true, false);
    let appearance = Appearance::fixed(Font::DEFAULT, t);
    for hidden in [false, true] {
        for horizontal in [false, true] {
            let native_bar = if hidden {
                iced_widget::scrollable::Scrollbar::hidden()
            } else {
                iced_widget::scrollable::Scrollbar::new()
                    .width(4)
                    .scroller_width(6)
                    .margin(3)
                    .spacing(12)
            };
            let shared_bar = if hidden {
                Scrollbar::hidden()
            } else {
                Scrollbar::embedded()
            };
            let direction = if horizontal {
                iced_widget::scrollable::Direction::Horizontal(native_bar)
            } else {
                iced_widget::scrollable::Direction::Vertical(native_bar)
            };
            let content = || Space::new().width(600).height(600);
            let spec = ScrollArea::new(appearance, content()).width(200).height(70);
            let new = if horizontal {
                spec.horizontal(shared_bar)
            } else {
                spec.vertical(shared_bar)
            }
            .into();
            let old = iced_widget::scrollable(content())
                .direction(direction)
                .width(200)
                .height(70)
                .style(t.scroll())
                .into();
            assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
        }
    }
}

#[test]
fn native_scroll_offset_survives_rebuild_and_resizes_then_clamps_on_content_shrink() {
    let t = palette::palette(true, false);
    let make = |height, viewport_height| -> Element<'static, u8> {
        ScrollArea::new(
            Appearance::fixed(Font::DEFAULT, t),
            Space::new().width(100).height(height),
        )
        .id("shared-native")
        .width(200)
        .height(viewport_height)
        .into()
    };
    let mut widget = make(1000, 75);
    let mut tree = Tree::new(&widget);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut messages = vec![];
    let mut node = segment_event(
        &mut widget,
        &mut tree,
        &renderer,
        &mut messages,
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -240.0 },
        }),
        Point::new(20.0, 20.0),
    );
    let mut before = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut before);
    assert!(before.offset.unwrap().y > 200.0);
    let native: Element<'_, u8> = iced_widget::scrollable(Space::new()).into();
    assert_eq!(tree.tag, Tree::new(&native).tag);
    widget = make(1000, 75);
    tree.diff(&widget);
    node = segment_event(
        &mut widget,
        &mut tree,
        &renderer,
        &mut messages,
        Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
        Point::new(20.0, 20.0),
    );
    let mut after = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut after);
    assert_eq!(before.offset, after.offset);
    widget = make(30, 75);
    tree.diff(&widget);
    node = segment_event(
        &mut widget,
        &mut tree,
        &renderer,
        &mut messages,
        Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
        Point::new(20.0, 20.0),
    );
    let mut shrunk = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut shrunk);
    assert_eq!(shrunk.offset.unwrap().y, 0.0);
}

#[derive(Debug, Clone, PartialEq)]
enum ScrollMessage {
    Scrolled,
    Seek(f32),
    Button,
}
fn scroll_filter(message: &ScrollMessage) -> ScrollAction {
    match message {
        ScrollMessage::Scrolled => ScrollAction::Discard,
        ScrollMessage::Seek(f) => ScrollAction::Vertical(*f),
        _ => ScrollAction::Forward,
    }
}
#[test]
fn local_scroll_updates_stay_local_and_external_operations_update_the_matching_thumb() {
    let t = palette::palette(true, false);
    let make = || -> Element<'static, ScrollMessage> {
        LocalScroll::new(
            iced_widget::row![
                scroll_area(iced_widget::column![
                    button("Action")
                        .width(100)
                        .height(24)
                        .on_press(ScrollMessage::Button),
                    Space::new().width(150).height(2000)
                ])
                .id("local-scroll")
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical(Scrollbar::hidden())
                .on_scroll(|_| ScrollMessage::Scrolled),
                ScrollRail::new(t, "local-scroll", ScrollMessage::Seek),
            ]
            .height(70)
            .width(200)
            .spacing(12),
        )
        .track("local-scroll")
        .filter(scroll_filter)
        .into()
    };
    let mut widget = make();
    let mut tree = Tree::new(&widget);
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut node = widget.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
    );
    let mut messages = vec![];
    for _ in 0..100 {
        let mut shell = Shell::new(&mut messages);
        widget.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -10.0 },
            }),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(20.0, 20.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut shell,
            &node.bounds(),
        );
        assert!(!shell.is_layout_invalid());
        assert!(!shell.are_widgets_invalid());
    }
    assert!(messages.is_empty());
    let mut before = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut before);
    assert!(before.offset.unwrap().y > 900.0);
    assert!(before.metrics.unwrap().0 > 0.45);
    widget = make();
    tree.diff(&widget);
    node = widget.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(240.0, 80.0)),
    );
    let mut after = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut after);
    assert_eq!(before.offset, after.offset);
    assert_eq!(before.metrics, after.metrics);
    let mut reset = iced_core::widget::operation::scrollable::snap_to::<()>(
        "local-scroll".into(),
        iced_core::widget::operation::scrollable::RelativeOffset {
            x: None,
            y: Some(0.0),
        },
    );
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut reset);
    let mut probe = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
    assert_eq!(probe.metrics.unwrap().0, 0.0);
    let mut other = iced_core::widget::operation::scrollable::snap_to::<()>(
        "another-area".into(),
        iced_core::widget::operation::scrollable::RelativeOffset {
            x: None,
            y: Some(1.0),
        },
    );
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut other);
    let mut probe = ScrollProbe::default();
    widget
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
    assert_eq!(probe.metrics.unwrap().0, 0.0);
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        widget.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(20.0, 12.0)),
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut Shell::new(&mut messages),
            &node.bounds(),
        );
    }
    assert_eq!(messages, vec![ScrollMessage::Button]);
}

#[test]
fn typography_roles_preserve_native_geometry_pixels_and_family_choices() {
    use sk_iced::typography::{TextRole, Typography};
    for font in [
        Font::with_name("Fira Sans"),
        Font::with_name("Roboto"),
        Font::with_name("Open Sans"),
    ] {
        let typography = Typography::new(font, Font::MONOSPACE);
        for (role, size) in [
            (TextRole::Tiny, 10.0),
            (TextRole::Badge, 10.667),
            (TextRole::Small, 11.0),
            (TextRole::Caption, 12.0),
            (TextRole::Control, 13.0),
            (TextRole::Body, 13.333),
            (TextRole::Emphasis, 14.667),
            (TextRole::Heading, 16.0),
            (TextRole::Subtitle, 19.0),
            (TextRole::Title, 22.0),
        ] {
            let new = typography.text(role, "Example text").into();
            let old = iced_widget::text("Example text")
                .font(font)
                .size(size)
                .into();
            assert_eq!(
                paint(new, &iced_core::Theme::Dark, false),
                paint(old, &iced_core::Theme::Dark, false)
            );
        }
        assert_eq!(
            paint(
                typography.raw("Inherited size").into(),
                &iced_core::Theme::Dark,
                false
            ),
            paint(
                iced_widget::text("Inherited size").font(font).into(),
                &iced_core::Theme::Dark,
                false
            )
        );
    }
}

#[test]
fn explicit_context_profiles_and_api_aliases_keep_native_widget_state() {
    use sk_iced::{
        design::{ControlProfile, DesignTokens},
        typography::Typography,
    };
    let appearance = Appearance::fixed(Font::DEFAULT, palette::palette(true, false));
    let design = DesignTokens {
        button: ControlProfile {
            height: 36.0,
            text_size: 14.0,
            horizontal_padding: 12.0,
            icon_gap: 6.0,
        },
        ..Default::default()
    };
    let context = UiContext::new(appearance, Policy::DISABLED)
        .with_typography(Typography::new(Font::DEFAULT, Font::MONOSPACE))
        .with_design(design);
    let new = context.button("Action").on_press(7).build();
    let old = Button::new(appearance, Policy::DISABLED)
        .text("Action")
        .height(36)
        .text_size(14)
        .padding([0, 12])
        .spacing(6)
        .on_press(7)
        .build();
    assert_eq!(
        paint(new, &iced_core::Theme::Dark, false),
        paint(old, &iced_core::Theme::Dark, false)
    );
    let a: Element<'_, u8> = TextInput::new(appearance, "", "Value")
        .text_size(13)
        .on_input(|_| 7)
        .build()
        .into();
    let b: Element<'_, u8> = TextInput::new(appearance, "", "Value")
        .size(13)
        .on_input(|_| 7)
        .into();
    assert_eq!(Tree::new(&a).tag, Tree::new(&b).tag);
    assert_eq!(
        paint(a, &iced_core::Theme::Dark, false),
        paint(b, &iced_core::Theme::Dark, false)
    );
    assert_eq!(
        paint(
            Progress::new(appearance, 0.0..=1.0, 0.5, 0x00ff00)
                .height(10)
                .build()
                .into(),
            &iced_core::Theme::Dark,
            false
        ),
        paint(
            Progress::new(appearance, 0.0..=1.0, 0.5, 0x00ff00)
                .girth(10)
                .into(),
            &iced_core::Theme::Dark,
            false
        )
    );
}

#[test]
fn semantic_roles_preserve_warning_and_destructive_color_distinction() {
    use sk_iced::design::ColorRole;
    for dark in [false, true] {
        let t = palette::palette(dark, false);
        assert_eq!(ColorRole::Warning.resolve(t), t.warning);
        assert_eq!(ColorRole::Danger.resolve(t), t.failure);
        assert_eq!(ColorRole::OnDanger.resolve(t), t.failure_text);
        let text = sk_iced::typography::Text::new(
            sk_iced::typography::Typography::new(Font::DEFAULT, Font::MONOSPACE),
            sk_iced::typography::TextRole::Body,
            "Warning",
        )
        .tone(ColorRole::Warning)
        .build();
        let old = iced_widget::text("Warning")
            .font(Font::DEFAULT)
            .size(13.333)
            .color(sk_iced::theme::color(t.warning));
        assert_eq!(
            paint(text.into(), &t.theme(), false),
            paint(old.into(), &t.theme(), false)
        );
    }
}

#[cfg(feature = "icons")]
#[test]
fn alpha_icon_cache_is_bounded_reuses_handles_and_preserves_rgba() {
    use sk_iced::icons::AlphaIconCache;
    let mut cache = AlphaIconCache::new(2);
    let a = cache.get_or_insert(1, 0x00332211, 2, 1, &[0, 128]).unwrap();
    let b = cache.get_or_insert(1, 0x00332211, 2, 1, &[0, 128]).unwrap();
    assert_eq!(a.id(), b.id());
    match &a {
        iced_core::image::Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => {
            assert_eq!((*width, *height), (2, 1));
            assert_eq!(pixels.as_ref(), &[17, 34, 51, 0, 17, 34, 51, 128]);
        }
        _ => panic!("Expected raw RGBA"),
    }
    assert!(cache.get_or_insert(2, 0, 0, 1, &[]).is_none());
    assert!(cache.get_or_insert(2, 0, 2, 2, &[1]).is_none());
    cache.get_or_insert(2, 0, 1, 1, &[255]).unwrap();
    cache.get_or_insert(3, 0, 1, 1, &[255]).unwrap();
    assert!(cache.len() <= 2);
}

#[cfg(feature = "raster-icons")]
#[test]
fn raster_icon_mips_keep_pixels_alpha_and_cache_identity_until_scale_changes() {
    use sk_iced::icons::RasterIconSet;
    let source = image::RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 128]));
    let mut cache = RasterIconSet::new(vec![source]);
    cache.prepare(Some(0x00332211), 4);
    let a = cache.get(0, Some(0x00332211), 4).unwrap();
    cache.prepare(Some(0x00332211), 4);
    assert_eq!(a.id(), cache.get(0, Some(0x00332211), 4).unwrap().id());
    assert!(!cache.set_scale(1.005));
    assert_eq!(a.id(), cache.get(0, Some(0x00332211), 4).unwrap().id());
    assert!(cache.set_scale(2.0));
    assert!(cache.get(0, Some(0x00332211), 4).is_none());
    cache.prepare(Some(0x00332211), 4);
    match cache.get(0, Some(0x00332211), 4).unwrap() {
        iced_core::image::Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } => {
            assert_eq!((width, height), (8, 8));
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [17, 34, 51, 128])
            );
        }
        _ => panic!("Expected raw RGBA"),
    }
    cache.prepare(None, 2);
    match cache.get(0, None, 2).unwrap() {
        iced_core::image::Handle::Rgba { pixels, .. } => {
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [10, 20, 30, 128])
            )
        }
        _ => panic!("Expected raw RGBA"),
    }
}

#[cfg(feature = "icons")]
#[test]
fn raster_icon_presentation_preserves_native_pixels_and_themed_disabled_opacity() {
    fn resolve(theme: &iced_core::Theme) -> sk_iced::theme::Tokens {
        let mut t = palette::from_theme(theme);
        t.opacity_disabled = 0.7;
        t
    }
    let appearance = Appearance::themed(Font::DEFAULT, resolve);
    let handle = iced_core::image::Handle::from_rgba(2, 2, vec![255u8; 16]);
    for enabled in [false, true] {
        let new = sk_iced::icons::Icon::raster(appearance, handle.clone())
            .size(16)
            .enabled(enabled)
            .build();
        let old = iced_widget::image(handle.clone())
            .width(16)
            .height(16)
            .opacity(if enabled { 1.0_f32 } else { 0.7_f32 })
            .into();
        assert_eq!(
            paint(new, &iced_core::Theme::Dark, false),
            paint(old, &iced_core::Theme::Dark, false)
        );
    }
}

#[cfg(all(feature = "icons", feature = "svg"))]
#[test]
fn vector_icon_tones_and_disabled_opacity_preserve_the_original_material_style() {
    use sk_iced::icons::{Icon, IconTone};
    let handle = iced_widget::svg::Handle::from_memory(br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M2 2h12v12H2z"/></svg>"#);
    for dark in [false, true] {
        for enabled in [false, true] {
            for danger in [false, true] {
                let t = palette::palette(dark, false);
                let new = Icon::svg(Appearance::fixed(Font::DEFAULT, t), handle.clone())
                    .enabled(enabled)
                    .tone(if danger {
                        IconTone::OnDanger
                    } else {
                        IconTone::Text
                    })
                    .build();
                let old = iced_widget::svg(handle.clone())
                    .width(16)
                    .height(16)
                    .style(move |_, _| iced_widget::svg::Style {
                        color: Some(
                            sk_iced::theme::color(if !enabled {
                                t.secondary
                            } else if danger {
                                t.failure_text
                            } else {
                                t.text
                            })
                            .scale_alpha(if enabled {
                                1.0
                            } else {
                                t.opacity_disabled as f32
                            }),
                        ),
                    })
                    .into();
                assert_eq!(paint(new, &t.theme(), false), paint(old, &t.theme(), false));
            }
        }
    }
}

#[test]
fn menu_item_matches_native_row_and_keeps_action_enablement() {
    for dark in [false, true] {
        let t = palette::palette(dark, false);
        let theme = t.theme();
        let appearance = Appearance::fixed(Font::DEFAULT, t);
        for enabled in [false, true] {
            for hover in [false, true] {
                let shared = MenuItem::new(appearance, Policy::DISABLED, "Open")
                    .icon(Space::new().width(16).height(16))
                    .on_press(1u8)
                    .enabled(enabled)
                    .feedback(false);
                let native = button(
                    iced_widget::row![
                        Space::new().width(16).height(16),
                        text(Font::DEFAULT, "Open").size(13)
                    ]
                    .spacing(8)
                    .align_y(iced_core::Alignment::Center),
                )
                .padding([8, 10])
                .width(Length::Fill)
                .on_press_maybe(enabled.then_some(1u8))
                .style(t.flat_dimmed());
                assert_eq!(
                    paint(shared.into(), &theme, hover),
                    paint(native.into(), &theme, hover)
                );
            }
        }
    }
}
#[test]
fn dialog_backdrop_preserves_native_container_geometry_and_pixels() {
    let t = palette::palette(true, false);
    let theme = t.theme();
    let shared = DialogBackdrop::new(Space::new().width(100).height(40)).padding(12);
    let native = container(Space::new().width(100).height(40))
        .center(Length::Fill)
        .padding(12)
        .style(|_| iced_widget::container::Style {
            background: Some(iced_core::Color::from_rgba(0., 0., 0., 0.55).into()),
            ..Default::default()
        });
    assert_eq!(
        paint(shared.into(), &theme, false),
        paint(native.into(), &theme, false)
    );
}
#[test]
fn dialog_commands_gate_acceptance_without_blocking_cancel() {
    use iced_core::keyboard::{Key, key::Named};
    let commands = DialogCommands::new().default_action(1u8).cancel_action(2);
    assert_eq!(commands.action(&Key::Named(Named::Enter)), Some(1));
    assert_eq!(commands.action(&Key::Named(Named::Escape)), Some(2));
    assert_eq!(commands.action(&Key::Character("x".into())), None);
    let busy = commands.busy(true);
    assert_eq!(busy.action_named(Named::Enter), None);
    assert_eq!(busy.action_named(Named::Escape), Some(2));
    assert_eq!(
        DialogCommands::<u8>::default().action_named(Named::Enter),
        None
    );
}

#[test]
fn theme_switch_preserves_an_accepted_button_press() {
    let make = |tokens| -> Element<'static, u8> {
        Button::new(Appearance::fixed(Font::DEFAULT, tokens), Policy::DISABLED)
            .text("Save")
            .on_press(7)
            .feedback(false)
            .into()
    };
    let renderer = Renderer::new(Font::DEFAULT, 13.0.into());
    let mut widget = make(palette::palette(false, false));
    let mut tree = Tree::new(&widget);
    let mut messages = vec![];
    let send =
        |widget: &mut Element<'_, u8>, tree: &mut Tree, event: Event, messages: &mut Vec<u8>| {
            let node = widget.as_widget_mut().layout(
                tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(240., 80.)),
            );
            widget.as_widget_mut().update(
                tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(12., 12.)),
                &renderer,
                &mut iced_core::clipboard::Null,
                &mut Shell::new(messages),
                &Rectangle::with_size(Size::new(240., 80.)),
            );
        };
    send(
        &mut widget,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        &mut messages,
    );
    widget = make(sk_iced::themes::neumorphic(palette::palette(false, false)).tokens);
    tree.diff(&widget);
    send(
        &mut widget,
        &mut tree,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        &mut messages,
    );
    assert_eq!(messages, vec![7]);
}
#[test]
fn themed_input_shadows_keep_native_focus_selection_and_geometry() {
    type InputState =
        iced_widget::text_input::State<<Renderer as iced_core::text::Renderer>::Paragraph>;
    let tokens = palette::palette(false, false);
    let appearance = Appearance::fixed(Font::DEFAULT, tokens);
    let mut styled = tokens;
    styled.visual.input.normal.shadows = Some([
        Some(sk_iced::visual::ShadowLayer {
            color: sk_iced::visual::Rgba::new(0xffffff, 0.6),
            angle: 90.,
            distance: 4.,
            blur: 5.,
        }),
        None,
        None,
        None,
    ]);
    let make = |appearance| -> Element<'static, u8> {
        TextInput::new(appearance, "Name", "hello")
            .on_input(|_| 1)
            .into()
    };
    let mut widget = make(appearance);
    let mut tree = Tree::new(&widget);
    let state = tree.state.downcast_mut::<InputState>();
    state.focus();
    state.select_range(1, 4);
    let cursor = state.cursor();
    widget = make(Appearance::fixed(Font::DEFAULT, styled));
    tree.diff(&widget);
    let state = tree.state.downcast_ref::<InputState>();
    assert!(state.is_focused());
    assert_eq!(state.cursor(), cursor);
    let plain = paint(make(appearance), &tokens.theme(), false);
    let decorated = paint(
        make(Appearance::fixed(Font::DEFAULT, styled)),
        &styled.theme(),
        false,
    );
    assert_eq!(plain.0, decorated.0);
    assert!(
        plain.1 != decorated.1,
        "themed shadow must change rendered pixels"
    );
}
#[cfg(feature = "icons")]
#[test]
fn semantic_icon_pack_changes_pixels_and_missing_names_use_fallbacks() {
    use sk_iced::{icons::Icon, visual::IconAsset};
    fn pack(name: &str) -> Option<IconAsset> {
        (name == "save").then_some(IconAsset::Glyph {
            font: Font::DEFAULT,
            character: '+',
        })
    }
    let base = palette::palette(false, false);
    let mut themed = base;
    themed.assets.icons = Some(pack);
    let make = |tokens, name| -> Element<'static, u8> {
        Icon::from_asset(
            Appearance::fixed(Font::DEFAULT, tokens),
            IconAsset::Glyph {
                font: Font::DEFAULT,
                character: '-',
            },
        )
        .unwrap()
        .semantic(name)
        .build()
    };
    let fallback = paint(make(base, "save"), &base.theme(), false);
    let missing = paint(make(themed, "missing"), &themed.theme(), false);
    let themed = paint(make(themed, "save"), &themed.theme(), false);
    assert_eq!(fallback, missing);
    assert_eq!(fallback.0, themed.0);
    assert_ne!(fallback.1, themed.1);
}

#[test]
fn custom_flat_rows_do_not_inherit_standard_button_elevation() {
    let t = sk_iced::themes::neumorphic(palette::palette(true, false)).tokens;
    let appearance = Appearance::fixed(Font::DEFAULT, t);
    let raw = || {
        button(Space::new().width(180).height(20))
            .padding(0)
            .on_press(1u8)
            .style(move |_, _| iced_widget::button::Style {
                background: Some(sk_iced::theme::color(t.surface).into()),
                ..Default::default()
            })
    };
    let native = paint(raw().into(), &t.theme(), false);
    let shared = paint(
        Button::custom(appearance, Policy::DISABLED, raw())
            .feedback(false)
            .into(),
        &t.theme(),
        false,
    );
    assert!(
        native == shared,
        "custom file rows must remain exactly flat"
    );
    let normal = t.button(false, false)(&t.theme(), iced_widget::button::Status::Active);
    let disabled = t.button(false, false)(&t.theme(), iced_widget::button::Status::Disabled);
    let pressed = t.button(false, false)(&t.theme(), iced_widget::button::Status::Pressed);

    assert!(normal.shadow.color.a > 0.);

    assert!(disabled.shadow.color.a > 0.);
    assert!(disabled.shadow.blur_radius < normal.shadow.blur_radius);

    assert!(t.visual.button.disabled.unwrap().opacity() < 1.);
    assert_eq!(pressed.shadow.color.a, 0.);
    assert!(t.visual.input.normal.inset.is_some());
    assert!(t.visual.progress.fill.is_some());
}
