// SPDX-License-Identifier: Apache-2.0
//! Compile and construct the README examples without opening a window.
#[cfg(feature = "components")]
fn main() {
    use iced_core::Font;
    use sk_iced::{
        animation::Element,
        components::{Appearance, ButtonTone, UiContext},
        motion::Policy,
    };

    let ui = UiContext::new(
        Appearance::fixed(Font::DEFAULT, sk_iced::palette::palette(true, false)),
        Policy::detect(),
    );
    let save: Element<'_, u8> = ui
        .button("Save")
        .tone(ButtonTone::Important)
        .width(120)
        .on_press(1)
        .build();

    let enabled = ui.toggle_switch(true).on_toggle(|_| 4u8);
    let card = ui
        .settings_card("Notifications", enabled)
        .description("Show a notification when a transfer completes.");
    let _: Element<'_, u8> = card.into();
    let scope: Element<'_, u8> =
        sk_iced::components::FocusScope::new(ui.button("Save").id("save").on_press(1))
            .autofocus("save")
            .into();
    let _ = scope;
    let name = ui.text_input("Name", "Example").on_input(|_| 2u8);
    let check = ui.checkbox(true).label("Enabled").on_toggle(|_| 3u8);
    let mode = ui
        .segmented()
        .labels(["Light", "Dark", "Auto"])
        .selected(1)
        .on_select(|index| index as u8);

    use sk_iced::components::Navigation;

    let navigation = Navigation::vertical()
        .spacing(4)
        .push(ui.button("Home").borderless().on_press(1u8))
        .push(ui.button("Settings").borderless().on_press(2u8));

    use sk_iced::components::{MenuItem, ScrollArea, Scrollbar};

    let item = MenuItem::new(ui.appearance, ui.motion, "Open")
        .shortcut("Ctrl+O")
        .on_press(1u8);
    let menu_body = ScrollArea::new(ui.appearance, item)
        .height(200)
        .vertical(Scrollbar::embedded());

    let _: Element<'_, u8> = name.into();
    let _: Element<'_, u8> = check.into();
    let _: Element<'_, u8> = mode.into();
    let _: Element<'_, u8> = navigation.into();
    let _: Element<'_, u8> = menu_body.into();
    let _ = save;
}
#[cfg(not(feature = "components"))]
fn main() {
    eprintln!("Enable the components feature to run this example.");
}
