// SPDX-License-Identifier: Apache-2.0
//! Run with --features components --example catalog -- <output-directory>.
//! Writes portable PPM contact sheets without adding a window/runtime dependency.
#[cfg(not(feature = "components"))]
fn main() {
    eprintln!("Enable the components feature to render the catalog.");
}
#[cfg(feature = "components")]
fn main() -> std::io::Result<()> {
    use iced_core::{
        Font, Layout, Length, Rectangle, Renderer as _, Size, layout, mouse, renderer,
        renderer::Headless as _, widget::Tree,
    };
    use iced_widget::{column, container, row, text_editor};
    use sk_iced::{
        animation::{Element, Renderer},
        components::*,
        design::ColorRole,
        motion::Policy,
        palette,
        theme::color,
    };
    use std::io::Write;
    let output = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "target/catalog".into());
    std::fs::create_dir_all(&output)?;
    for dark in [false, true] {
        let t = palette::palette(dark, false);
        let t = match std::env::args().nth(2).as_deref() {
            Some("material") => sk_iced::themes::material(t).tokens,
            Some("metro") => sk_iced::themes::metro(t).tokens,
            Some("neumorphic") => sk_iced::themes::neumorphic(t).tokens,
            _ => t,
        };
        let appearance = Appearance::fixed(Font::DEFAULT, t);
        let content = text_editor::Content::<Renderer>::with_text(
            "Selectable read-only results\nSelect and copy text without editing it.",
        );
        let mut choices = ChoiceGrid::new(4).spacing(4);
        for label in ["One", "Two", "Three", "Four"] {
            choices = choices.push(
                Button::new(appearance, Policy::DISABLED)
                    .text(label)
                    .on_press(0u8)
                    .feedback(false),
            );
        }
        let body: Element<'_, u8> = container(
            column![
                iced_widget::text("sk-iced component catalog").size(24),
                row![
                    Button::new(appearance, Policy::DISABLED)
                        .text("Default")
                        .on_press(0),
                    Button::new(appearance, Policy::DISABLED)
                        .text("Primary")
                        .tone(ButtonTone::Important)
                        .on_press(0),
                    Button::new(appearance, Policy::DISABLED)
                        .text("Danger")
                        .tone(ButtonTone::Danger)
                        .on_press(0),
                    Button::new(appearance, Policy::DISABLED)
                        .text("Disabled")
                        .enabled(false)
                        .on_press(0)
                ]
                .spacing(8),
                row![
                    Checkbox::new(appearance, true)
                        .label("Checked")
                        .on_toggle(|_| 0),
                    Checkbox::with_state(appearance, CheckState::Mixed)
                        .label("Mixed")
                        .on_toggle(|_| 0),
                    Checkbox::new(appearance, false)
                        .label("Disabled")
                        .enabled(false)
                ]
                .spacing(20),
                Segmented::new(appearance, Policy::DISABLED)
                    .labels(["Light", "Dark", "Auto"])
                    .selected(if dark { 1 } else { 0 })
                    .width(300)
                    .on_select(|_| 0),
                FormField::new(
                    appearance,
                    "Name",
                    TextInput::new(appearance, "Name", "Example").on_input(|_| 0)
                )
                .hint("Enter a display name"),
                FormField::new(
                    appearance,
                    "Validation",
                    TextInput::new(appearance, "Value", "").on_input(|_| 0)
                )
                .error("This field is required"),
                Slider::new(0u16..=100, 40, |_| 0).step(10u16),
                row![
                    Dropdown::new(appearance, vec!["One", "Two"], Some("One"), |_| 0).enabled(true),
                    Dropdown::new(appearance, vec!["One", "Two"], Some("One"), |_| 0)
                        .enabled(false)
                ]
                .spacing(8),
                choices,
                Notice::new(appearance, "Information with an immediate action")
                    .tone(ColorRole::Text)
                    .action(
                        Button::new(appearance, Policy::DISABLED)
                            .text("Action")
                            .on_press(0)
                    )
                    .padding(8)
                    .surface(true),
                Notice::new(appearance, "Warning").tone(ColorRole::Warning),
                EmptyState::new(appearance, "Nothing to show yet").text_size(13),
                LabeledProgress::new(appearance, 0.45, t.accent, "45%").height(24),
                ShareMeter::new(appearance, 37.0).height(8.0),
                ReadOnlyText::new(appearance, &content)
                    .on_action(|_| 0)
                    .height(80),
                row![
                    StatusBadge::new(appearance, "Ready", t.success),
                    StatusBadge::new(appearance, "Warning", t.warning),
                    StatusBadge::new(appearance, "Failed", t.failure)
                ]
                .spacing(8),
                row![
                    Tab::new(appearance, Policy::DISABLED, "Overview", true, Some(0)),
                    Tab::new(appearance, Policy::DISABLED, "Details", false, Some(0))
                ]
                .spacing(4),
                Panel::new(
                    appearance,
                    MenuItem::new(appearance, Policy::DISABLED, "Open preview")
                        .shortcut("Ctrl+O")
                        .on_press(0)
                        .feedback(false)
                ),
                PreviewSurface::new(
                    Panel::new(appearance, iced_widget::text("Preview content")).padding(10)
                )
                .toolbar(iced_widget::text("Preview toolbar"))
                .footer(iced_widget::text("Preview status"))
                .spacing(8)
            ]
            .spacing(16),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| iced_widget::container::Style {
            background: Some(color(t.background).into()),
            text_color: Some(color(t.text)),
            ..Default::default()
        })
        .into();
        let mut view = body;
        let mut renderer = Renderer::new(Font::DEFAULT, 13.333.into());
        let mut tree = Tree::new(&view);
        let size = Size::new(900.0, 1000.0);
        let bounds = Rectangle::with_size(size);
        let node =
            view.as_widget_mut()
                .layout(&mut tree, &renderer, &layout::Limits::new(size, size));
        let mut ignored = Vec::new();
        view.as_widget_mut().update(
            &mut tree,
            &iced_core::Event::Window(iced_core::window::Event::RedrawRequested(
                std::time::Instant::now(),
            )),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut iced_core::clipboard::Null,
            &mut iced_core::Shell::new(&mut ignored),
            &bounds,
        );
        renderer.reset(bounds);
        view.as_widget().draw(
            &tree,
            &mut renderer,
            &t.theme(),
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &bounds,
        );
        let rgba = renderer.screenshot(Size::new(900, 1000), 1.0, color(t.background));
        let mut file =
            std::fs::File::create(output.join(if dark { "dark.ppm" } else { "light.ppm" }))?;
        file.write_all(b"P6\n900 1000\n255\n")?;
        let rgb = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| p[..3].iter().copied())
            .collect::<Vec<_>>();
        file.write_all(&rgb)?;
    }
    println!("Catalog written to {}", output.display());
    Ok(())
}
