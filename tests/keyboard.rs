// SPDX-License-Identifier: Apache-2.0
#![cfg(feature = "components")]
use iced_core::{
    Event, Font, Layout, Rectangle, Shell, Size,
    keyboard::{
        self, Key,
        key::{Named, NativeCode, Physical},
    },
    layout, mouse,
    widget::{Operation, Tree, operation::Focusable},
};
use iced_widget::{column, row};
use sk_iced::{
    animation::{Element, Renderer},
    components::*,
    motion::Policy,
    palette,
};
fn ui() -> UiContext {
    UiContext::new(
        Appearance::fixed(Font::DEFAULT, palette::palette(true, false)),
        Policy::DISABLED,
    )
}
fn key(key: Named, modifiers: keyboard::Modifiers, repeat: bool) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(key),
        modified_key: Key::Named(key),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: keyboard::Location::Standard,
        modifiers,
        text: None,
        repeat,
    })
}
fn send<M>(w: &mut Element<'_, M>, t: &mut Tree, r: &Renderer, e: Event) -> (Vec<M>, bool) {
    let bounds = Rectangle::with_size(Size::new(600., 400.));
    let n = w
        .as_widget_mut()
        .layout(t, r, &layout::Limits::new(Size::ZERO, bounds.size()));
    let mut messages = vec![];
    let mut shell = Shell::new(&mut messages);
    w.as_widget_mut().update(
        t,
        &e,
        Layout::new(&n),
        mouse::Cursor::Unavailable,
        r,
        &mut iced_core::clipboard::Null,
        &mut shell,
        &bounds,
    );
    let captured = shell.is_event_captured();
    (messages, captured)
}
#[derive(Default)]
struct Focused {
    count: usize,
    current: Option<usize>,
}
impl Operation for Focused {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self)
    }
    fn focusable(
        &mut self,
        _: Option<&iced_core::widget::Id>,
        _: Rectangle,
        s: &mut dyn Focusable,
    ) {
        if s.is_focused() {
            self.current = Some(self.count);
        }
        self.count += 1;
    }
}
fn focused<M>(w: &mut Element<'_, M>, t: &mut Tree, r: &Renderer) -> Focused {
    let n = w.as_widget_mut().layout(
        t,
        r,
        &layout::Limits::new(Size::ZERO, Size::new(600., 400.)),
    );
    let mut op = Focused::default();
    w.as_widget_mut().operate(t, Layout::new(&n), r, &mut op);
    op
}
#[test]
fn tab_order_skips_disabled_and_preserves_focus_on_rebuild() {
    let make = || {
        FocusScope::new(row![
            ui().button("First").on_press(1u8),
            ui().button("Disabled").enabled(false).on_press(2u8),
            ui().button("Last").on_press(3u8)
        ])
        .build()
    };
    let mut w = make();
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    assert_eq!(focused(&mut w, &mut t, &r).count, 2);
    for (modifiers, expected) in [
        (keyboard::Modifiers::empty(), 0),
        (keyboard::Modifiers::empty(), 1),
        (keyboard::Modifiers::empty(), 0),
        (keyboard::Modifiers::SHIFT, 1),
    ] {
        assert!(send(&mut w, &mut t, &r, key(Named::Tab, modifiers, false)).1);
        assert_eq!(focused(&mut w, &mut t, &r).current, Some(expected));
        w = make();
        t.diff(&w);
        assert_eq!(focused(&mut w, &mut t, &r).current, Some(expected));
    }
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::Enter, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![3]
    );
    assert!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::Enter, keyboard::Modifiers::empty(), true)
        )
        .0
        .is_empty()
    );
    assert!(
        !send(
            &mut w,
            &mut t,
            &r,
            key(Named::Tab, keyboard::Modifiers::CTRL, false)
        )
        .1
    );
    assert!(
        !send(
            &mut w,
            &mut t,
            &r,
            key(Named::Enter, keyboard::Modifiers::CTRL, false)
        )
        .1
    );
}
#[test]
fn switch_and_checkbox_activate_once_under_every_motion_policy() {
    for policy in [
        Policy::DISABLED,
        Policy {
            system_enabled: true,
        },
        Policy::DISABLED,
    ] {
        let mut w = FocusScope::new(column![
            ToggleSwitch::new(ui().appearance, policy, false).on_toggle(|v| v),
            ui().checkbox(false).on_toggle(|v| v)
        ])
        .build();
        let mut t = Tree::new(&w);
        let r = Renderer::new(Font::DEFAULT, 13.0.into());
        for _ in 0..2 {
            send(
                &mut w,
                &mut t,
                &r,
                key(Named::Tab, keyboard::Modifiers::empty(), false),
            );
            assert_eq!(
                send(
                    &mut w,
                    &mut t,
                    &r,
                    key(Named::Space, keyboard::Modifiers::empty(), false)
                )
                .0,
                vec![true]
            );
        }
    }
}
#[test]
fn segmented_arrows_skip_unavailable_choices_and_keep_one_focus_stop() {
    let mut w = FocusScope::new(
        ui().segmented()
            .options([
                Segment::new("First"),
                Segment::new("Disabled").enabled(false),
                Segment::new("Last"),
            ])
            .selected(0)
            .on_select(|i| i),
    )
    .build();
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    assert_eq!(focused(&mut w, &mut t, &r).count, 1);
    send(
        &mut w,
        &mut t,
        &r,
        key(Named::Tab, keyboard::Modifiers::empty(), false),
    );
    for keyname in [Named::ArrowRight, Named::End, Named::ArrowLeft] {
        assert_eq!(
            send(
                &mut w,
                &mut t,
                &r,
                key(keyname, keyboard::Modifiers::empty(), false)
            )
            .0,
            vec![2]
        );
    }
}
#[test]
fn dropdown_arrow_keys_select_without_delaying_or_leaving_a_menu_open() {
    let mut w = FocusScope::new(
        Dropdown::new(ui().appearance, vec![1u8, 2, 3], Some(1), |v| v)
            .on_open(9)
            .on_close(8),
    )
    .build();
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    send(
        &mut w,
        &mut t,
        &r,
        key(Named::Tab, keyboard::Modifiers::empty(), false),
    );
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::Space, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![9]
    );
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::ArrowDown, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![8, 2]
    );
    let bounds = Rectangle::with_size(Size::new(600., 400.));
    let n = w
        .as_widget_mut()
        .layout(&mut t, &r, &layout::Limits::new(Size::ZERO, bounds.size()));
    assert!(
        w.as_widget_mut()
            .overlay(
                &mut t,
                Layout::new(&n),
                &r,
                &bounds,
                iced_core::Vector::ZERO
            )
            .is_none()
    );
}
#[test]
fn ids_support_entry_and_focus_restoration_without_replacing_native_text_state() {
    let make = |active| {
        FocusScope::new(row![
            ui().button("Trigger").id("trigger").on_press(1u8),
            ui().text_input("Text", "Keep me")
                .id("editor")
                .on_input(|_| 2u8)
        ])
        .active(active)
        .autofocus("editor")
        .build()
    };
    let mut w = make(true);
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    assert_eq!(focused(&mut w, &mut t, &r).current, Some(1));
    let n = w.as_widget_mut().layout(
        &mut t,
        &r,
        &layout::Limits::new(Size::ZERO, Size::new(600., 400.)),
    );
    w.as_widget_mut().operate(
        &mut t,
        Layout::new(&n),
        &r,
        &mut iced_core::widget::operation::focusable::focus::<()>("trigger".into()),
    );
    assert_eq!(focused(&mut w, &mut t, &r).current, Some(0));
    w = make(false);
    t.diff(&w);
    assert_eq!(focused(&mut w, &mut t, &r).count, 0);
    w = make(true);
    t.diff(&w);
    assert_eq!(focused(&mut w, &mut t, &r).current, Some(1));
}

#[test]
fn slider_arrows_use_native_steps_and_tab_reveals_scrolled_controls() {
    let mut w = FocusScope::new(Slider::new(0u8..=100u8, 50u8, |v| v).step(10u8)).build();
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    send(
        &mut w,
        &mut t,
        &r,
        key(Named::Tab, keyboard::Modifiers::empty(), false),
    );
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::ArrowRight, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![60]
    );
    struct Offset(f32);
    impl Operation for Offset {
        fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
            f(self)
        }
        fn scrollable(
            &mut self,
            _: Option<&iced_core::widget::Id>,
            _: Rectangle,
            _: Rectangle,
            tr: iced_core::Vector,
            _: &mut dyn iced_core::widget::operation::Scrollable,
        ) {
            self.0 = tr.y;
        }
    }
    let mut controls = column![].spacing(8);
    for i in 0..12u8 {
        controls = controls.push(ui().button(format!("Action {i}")).on_press(i));
    }
    let mut w = FocusScope::new(ScrollArea::new(ui().appearance, controls).height(80)).build();
    let mut t = Tree::new(&w);
    for _ in 0..9 {
        assert!(
            send(
                &mut w,
                &mut t,
                &r,
                key(Named::Tab, keyboard::Modifiers::empty(), false)
            )
            .1
        );
    }
    let n = w.as_widget_mut().layout(
        &mut t,
        &r,
        &layout::Limits::new(Size::ZERO, Size::new(600., 400.)),
    );
    let mut offset = Offset(0.);
    w.as_widget_mut()
        .operate(&mut t, Layout::new(&n), &r, &mut offset);
    assert!(offset.0 > 0.);
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::Enter, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![8]
    );
}

#[test]
fn leaving_dropdown_closes_pointer_menu_without_consuming_tab() {
    let mut w = FocusScope::new(row![
        Dropdown::new(ui().appearance, vec![1u8, 2], Some(1), |v| v).on_close(8),
        ui().button("Next").on_press(9u8)
    ])
    .build();
    let mut t = Tree::new(&w);
    let r = Renderer::new(Font::DEFAULT, 13.0.into());
    send(
        &mut w,
        &mut t,
        &r,
        key(Named::Tab, keyboard::Modifiers::empty(), false),
    );
    send(
        &mut w,
        &mut t,
        &r,
        key(Named::Space, keyboard::Modifiers::empty(), false),
    );
    assert_eq!(
        send(
            &mut w,
            &mut t,
            &r,
            key(Named::Tab, keyboard::Modifiers::empty(), false)
        )
        .0,
        vec![8]
    );
    assert_eq!(focused(&mut w, &mut t, &r).current, Some(1));
}

#[test]
fn keyboard_focus_is_visible_on_primary_buttons_without_changing_layout() {
    use iced_core::{Renderer as _, renderer, renderer::Headless as _};
    let mut w = ui()
        .button("")
        .tone(ButtonTone::Important)
        .width(80)
        .height(32)
        .id("primary")
        .on_press(1u8)
        .build();
    let mut t = Tree::new(&w);
    let mut r = Renderer::new(Font::DEFAULT, 13.0.into());
    let bounds = Rectangle::with_size(Size::new(80., 32.));
    let n = w
        .as_widget_mut()
        .layout(&mut t, &r, &layout::Limits::new(Size::ZERO, bounds.size()));
    let _ = send(
        &mut w,
        &mut t,
        &r,
        Event::Window(iced_core::window::Event::RedrawRequested(
            std::time::Instant::now(),
        )),
    );
    let mut images = vec![];
    for focus in [false, true] {
        if focus {
            w.as_widget_mut().operate(
                &mut t,
                Layout::new(&n),
                &r,
                &mut iced_core::widget::operation::focusable::focus::<()>("primary".into()),
            );
        }
        r.reset(bounds);
        w.as_widget().draw(
            &t,
            &mut r,
            &iced_core::Theme::Dark,
            &renderer::Style::default(),
            Layout::new(&n),
            mouse::Cursor::Unavailable,
            &bounds,
        );
        images.push(r.screenshot(Size::new(80, 32), 1., iced_core::Color::BLACK));
    }
    let count = |pixels: &Vec<u8>| {
        (10..22)
            .flat_map(|y| (1..5).map(move |x| (y * 80 + x) * 4))
            .filter(|i| pixels[*i..*i + 3] == [255, 255, 255])
            .count()
    };
    assert_eq!(count(&images[0]), 0);
    assert!(count(&images[1]) > 0);
    assert_eq!(n.size(), Size::new(80., 32.));
}
