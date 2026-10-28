// SPDX-License-Identifier: Apache-2.0
//! Theme-defined surfaces and assets, constructed without opening a window.
#[cfg(feature = "icons")]
fn main() {
    use iced_core::Font;
    use sk_iced::{motion::Policy, palette, visual::*};

    let mut visual = VisualTheme::default();
    visual.button.normal = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 135.,
            start: Rgba::rgb(255, 255, 255, 0.35),
            end: Rgba::rgb(180, 210, 255, 0.15),
        }),
        shape: Some(Shape::Rounded([16.; 4])),
        border_color: Some(Rgba::rgb(255, 255, 255, 0.5)),
        border_width: Some(1.),
        ..Default::default()
    };
    visual.button.hovered = Some(SurfaceStyle {
        border_width: Some(2.), // Other properties inherit from normal.
        ..Default::default()
    });
    let theme = ThemeDefinition::new(palette::palette(false, false))
        .visuals(visual)
        .fonts(Font::DEFAULT, Font::MONOSPACE);
    let ui = theme.context(Policy::detect());
    let save = ui.button("Save").on_press(1u8).build();
    let iced_theme = theme.tokens.theme();

    fn pack(name: &str) -> Option<IconAsset> {
        (name == "save").then_some(IconAsset::Glyph {
            font: Font::MONOSPACE,
            character: '+',
        })
    }
    let theme = theme.icons(pack);
    let ui = theme.context(Policy::DISABLED);
    let icon = sk_iced::icons::Icon::from_asset(
        ui.appearance,
        IconAsset::Glyph {
            font: Font::DEFAULT,
            character: '-',
        },
    )
    .unwrap()
    .semantic("save")
    .build::<u8>();
    let _button = ui.button("Save").icon(icon).on_press(1u8).build();
    assert_eq!(palette::from_theme(&iced_theme).visual, visual);
    let _ = save;
}
#[cfg(not(feature = "icons"))]
fn main() {
    eprintln!("Enable icons to run the theming example.");
}
