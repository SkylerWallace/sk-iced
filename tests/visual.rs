#![cfg(feature = "components")]
use iced_core::{
    Font,
    keyboard::{Key, key::Named},
};
use iced_widget::button;
use sk_iced::{components::*, motion::Policy, palette, visual::*};

#[test]
fn arbitrary_theme_roundtrips_without_palette_guessing_or_cross_window_state() {
    let mut custom = palette::palette(false, false);
    custom.background = 0x324455;
    custom.accent = 0x771122;
    custom.visual.button.normal = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 130.,
            start: Rgba::new(0xffffff, 0.3),
            end: Rgba::new(custom.accent, 0.8),
        }),
        shape: Some(Shape::Pill),
        border_width: Some(2.),
        ..Default::default()
    };
    let other = neumorphic(palette::palette(true, false)).tokens;
    let a = custom.theme();
    let b = other.theme();
    assert_eq!(palette::from_theme(&a), custom);
    assert_eq!(palette::from_theme(&b), other);
    assert_eq!(palette::from_theme(&a.clone()), custom);
    let foreign = iced_core::Theme::custom("External", a.palette());
    assert_eq!(palette::from_theme(&foreign).visual, VisualTheme::default());
}

#[test]
fn skins_are_data_and_optional_state_properties_fall_back_to_normal() {
    let base = palette::palette(false, false);
    let mut custom = base;
    custom.visual.button.normal.shape = Some(Shape::Rectangle);
    custom.visual.button.normal.opacity = Some(0.42);
    let theme = custom.theme();
    for status in [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
    ] {
        let style = custom.button(false, false)(&theme, status);
        assert_eq!(style.border.radius, 0f32.into());
        assert_eq!(
            custom.visual.button.button(status),
            custom.visual.button.normal
        );
    }
    let neo = neumorphic(base).tokens;
    assert_ne!(base, neo);
    assert_ne!(
        neo.visual.button.button(button::Status::Active),
        neo.visual.button.button(button::Status::Pressed)
    );
    let theme = neo.theme();
    // Tone and disabled action semantics remain independent of the visual preset.
    assert_eq!(
        format!(
            "{:?}",
            neo.button(true, true)(&theme, button::Status::Disabled)
        ),
        format!(
            "{:?}",
            neo.button(false, false)(&theme, button::Status::Disabled)
        )
    );
    assert_eq!(
        DialogCommands::new()
            .default_action(7)
            .busy(true)
            .action(&Key::Named(Named::Enter)),
        None
    );
}

#[test]
fn fonts_design_and_icon_packs_belong_to_the_theme_and_have_fallbacks() {
    fn pack(name: &str) -> Option<IconAsset> {
        (name == "save").then_some(IconAsset::Glyph {
            font: Font::MONOSPACE,
            character: '+',
        })
    }
    let theme = ThemeDefinition::new(palette::palette(false, false))
        .fonts(Font::MONOSPACE, Font::with_name("Example Mono"))
        .icons(pack);
    let ui = theme.context(Policy::DISABLED);
    assert_eq!(ui.appearance.font, Font::MONOSPACE);
    assert_eq!(ui.typography.monospace, Font::with_name("Example Mono"));
    assert!(
        ui.appearance
            .tokens(&theme.tokens.theme())
            .assets
            .icons
            .unwrap()("save")
        .is_some()
    );
    assert!(pack("unknown").is_none());
    let offset = ShadowLayer {
        color: Rgba::new(0, 0.3),
        angle: 90.,
        distance: 6.,
        blur: 8.,
    }
    .native(1.)
    .offset;
    assert!(offset.x.abs() < 0.001);
    assert!((offset.y - 6.).abs() < 0.001);
}

#[test]
fn material_and_metro_are_data_presets_with_distinct_geometry() {
    for dark in [false, true] {
        let base = palette::palette(dark, false);
        let material = sk_iced::themes::material(base).tokens;
        let metro = sk_iced::themes::metro(base).tokens;
        assert_eq!(material.accent, base.accent);
        assert_eq!(metro.accent, base.accent);
        assert_eq!(metro.visual.panel.shape, Some(Shape::Rectangle));
        assert!(metro.visual.panel.layers().iter().all(Option::is_none));
        assert_ne!(material.visual.panel.shape, metro.visual.panel.shape);
        assert_eq!(palette::from_theme(&metro.theme()), metro);
        assert_eq!(palette::from_theme(&material.theme()), material);
    }
}

#[test]
fn toggled_buttons_and_selected_segments_use_persistent_recessed_material() {
    use iced_widget::button::Status;
    for dark in [false, true] {
        let t = sk_iced::themes::neumorphic(palette::palette(dark, false)).tokens;
        for status in [Status::Active, Status::Hovered, Status::Pressed] {
            let a = sk_iced::components::SurfaceRole::Toggle(true).resolve(t, status);
            let b = sk_iced::components::SurfaceRole::Segment(true).resolve(t, status);
            assert!(a.inset.unwrap().width > 0.);
            assert_eq!(a.inset, b.inset);
            assert_ne!(a.fill, b.fill);
            assert!(b.inset.unwrap().width > 0.);
            assert!(t.visual.dropdown.unwrap().normal.inset.is_none());
            assert!(
                t.visual
                    .dropdown
                    .unwrap()
                    .normal
                    .layers()
                    .iter()
                    .any(Option::is_some)
            );
            assert!(a.layers().iter().all(Option::is_none));
            assert_eq!(
                t.toggle_button(true)(&t.theme(), status).text_color,
                sk_iced::theme::color(t.text)
            );
        }
        assert_eq!(palette::from_theme(&t.theme()), t);
    }
    let t = palette::palette(true, false);
    assert_eq!(
        t.toggle_button(true)(&t.theme(), Status::Active),
        t.button(true, false)(&t.theme(), Status::Active)
    );
}

#[test]
fn material_and_metro_segment_groups_have_distinct_theme_owned_surfaces() {
    for dark in [false, true] {
        let base = palette::palette(dark, false);
        let material = sk_iced::themes::material(base).tokens;
        let metro = sk_iced::themes::metro(base).tokens;
        assert_eq!(
            material.visual.segment_group.unwrap().shape,
            Some(Shape::Pill)
        );
        assert_eq!(
            metro.visual.segment_group.unwrap().shape,
            Some(Shape::Rectangle)
        );
        for t in [material, metro] {
            assert_ne!(
                t.visual.segment.unwrap().normal.fill,
                t.visual.selected_segment.unwrap().normal.fill
            );
            assert_eq!(palette::from_theme(&t.theme()), t);
        }
    }
}
