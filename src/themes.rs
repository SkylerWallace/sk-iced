// SPDX-License-Identifier: Apache-2.0
//! Optional visual presets expressed entirely through public theme properties.
use crate::{
    theme::{Tokens, color},
    visual::*,
};
pub fn neumorphic(mut tokens: Tokens) -> ThemeDefinition {
    let dark =
        color(tokens.background).r + color(tokens.background).g + color(tokens.background).b < 1.5;
    let rgb = |r: u32, g: u32, b: u32| r | g << 8 | b << 16;
    tokens.background = if dark {
        rgb(41, 47, 56)
    } else {
        rgb(228, 233, 239)
    };
    tokens.surface = tokens.background;
    tokens.navigation_background = tokens.background;
    tokens.border = if dark {
        rgb(51, 59, 70)
    } else {
        rgb(206, 214, 224)
    };
    let light = Rgba::new(
        if dark { rgb(67, 76, 90) } else { 0xffffff },
        if dark { 0.6 } else { 0.95 },
    );
    let shade = Rgba::new(
        if dark {
            rgb(17, 21, 28)
        } else {
            rgb(169, 181, 198)
        },
        0.65,
    );
    let mut raised = SurfaceStyle {
        shape: Some(Shape::Rounded([9.; 4])),
        border_width: Some(0.),
        shadows: Some([
            Some(ShadowLayer {
                color: shade,
                angle: 45.,
                distance: 3.5,
                blur: 6.,
            }),
            Some(ShadowLayer {
                color: light,
                angle: 225.,
                distance: 3.5,
                blur: 6.,
            }),
            None,
            None,
        ]),
        ..Default::default()
    };
    raised.fill = Some(Fill::Solid(Rgba::new(tokens.surface, 1.)));
    let pressed = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 315.,
            start: Rgba::new(tokens.surface, 1.),
            end: Rgba::new(tokens.border, 1.),
        }),
        shadows: Some([None; 4]),
        border_width: Some(1.),
        border_color: Some(Rgba::new(tokens.border, 1.)),
        ..raised
    };
    let disabled = SurfaceStyle {
        shadows: Some([None; 4]),
        opacity: Some(0.65),
        ..raised
    };
    let button = States {
        normal: raised,
        hovered: Some(SurfaceStyle {
            border_width: Some(1.),
            border_color: Some(Rgba::new(tokens.accent, 0.55)),
            ..raised
        }),
        pressed: Some(pressed),
        disabled: Some(disabled),
        focused: Some(pressed),
    };
    let colored = States {
        normal: SurfaceStyle {
            fill: None,
            ..raised
        },
        pressed: Some(SurfaceStyle {
            fill: None,
            ..pressed
        }),
        disabled: Some(disabled),
        ..button
    };
    tokens.visual = VisualTheme {
        toggled_button: None,
        dropdown: None,
        button,
        important: colored,
        danger: colored,
        flat: States {
            normal: SurfaceStyle {
                shape: raised.shape,
                ..Default::default()
            },
            hovered: Some(raised),
            pressed: Some(pressed),
            ..Default::default()
        },
        input: States {
            normal: pressed,
            focused: Some(SurfaceStyle {
                border_color: Some(Rgba::new(tokens.accent, 1.)),
                ..pressed
            }),
            ..Default::default()
        },
        checkbox: States {
            normal: SurfaceStyle {
                shape: Some(Shape::Rounded([4.; 4])),
                ..Default::default()
            },
            ..Default::default()
        },
        panel: SurfaceStyle {
            shadows: Some([None; 4]),
            shape: Some(Shape::Rounded([12.; 4])),
            ..raised
        },
        selected_panel: Some(SurfaceStyle {
            shadows: Some([None; 4]),
            border_width: Some(1.),
            border_color: Some(Rgba::new(tokens.accent, 0.6)),
            shape: Some(Shape::Rounded([12.; 4])),
            ..raised
        }),
        header: SurfaceStyle {
            shadows: Some([None; 4]),
            ..raised
        },
        tab: SurfaceStyle {
            shape: Some(Shape::Rounded([10., 10., 0., 0.])),
            ..pressed
        },
        selected_tab: Some(SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
            shape: Some(Shape::Rounded([10., 10., 0., 0.])),
            border_width: Some(1.),
            border_color: Some(Rgba::new(tokens.accent, 0.5)),
            ..Default::default()
        }),
        badge: SurfaceStyle {
            shape: Some(Shape::Pill),
            ..Default::default()
        },
        menu: raised,
        ..Default::default()
    };
    let flat = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
        shape: Some(Shape::Rounded([8.; 4])),
        border_width: Some(1.),
        shadows: Some([None; 4]),
        ..Default::default()
    };
    let bevel = InsetBevel {
        shade: Rgba::new(
            if dark {
                rgb(13, 17, 23)
            } else {
                rgb(151, 165, 184)
            },
            0.65 * 0.35,
        ),
        light: Rgba::new(light.rgb, light.alpha * 0.35),
        width: 2.5,
    };
    let recessed = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(
            if dark {
                rgb(32, 38, 47)
            } else {
                rgb(215, 223, 234)
            },
            1.,
        ))),
        inset: Some(bevel),
        border_color: Some(Rgba::new(tokens.border, 0.8)),
        border_width: Some(1.),
        shadows: Some([None; 4]),
        shape: Some(Shape::Rounded([7.; 4])),
        ..Default::default()
    };
    let unavailable = SurfaceStyle {
        inset: Some(InsetBevel { width: 0., ..bevel }),
        text_color: Some(Rgba::new(tokens.secondary, 0.55)),
        fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
        border_color: Some(Rgba::new(tokens.border, 0.5)),
        border_width: Some(1.),
        opacity: Some(0.65),
        shadows: Some([
            Some(ShadowLayer {
                color: shade,
                angle: 45.,
                distance: 1.,
                blur: 2.,
            }),
            None,
            None,
            None,
        ]),
        ..flat
    };
    tokens.visual.flat.hovered = Some(SurfaceStyle {
        shadows: Some([None; 4]),
        border_width: Some(0.),
        ..Default::default()
    });
    tokens.visual.flat.pressed = Some(recessed);
    tokens.visual.card = flat;
    tokens.visual.disclosure = SurfaceStyle {
        shape: Some(Shape::Rounded([10.; 4])),
        ..raised
    };
    let toggled = States {
        normal: SurfaceStyle {
            text_color: Some(Rgba::new(tokens.text, 1.)),
            ..recessed
        },
        hovered: Some(SurfaceStyle {
            text_color: Some(Rgba::new(tokens.text, 1.)),
            border_color: Some(Rgba::new(tokens.accent, 0.6)),
            ..recessed
        }),
        pressed: Some(recessed),
        disabled: Some(unavailable),
        ..Default::default()
    };
    tokens.visual.toggled_button = Some(toggled);
    tokens.visual.button.disabled = Some(unavailable);
    tokens.visual.button.pressed = Some(recessed);
    tokens.visual.important.hovered = Some(SurfaceStyle {
        fill: None,
        ..raised
    });
    tokens.visual.important.pressed = Some(SurfaceStyle {
        fill: None,
        ..recessed
    });
    tokens.visual.important.disabled = Some(unavailable);
    tokens.visual.danger = tokens.visual.important;
    tokens.visual.input = States {
        normal: recessed,
        focused: Some(SurfaceStyle {
            border_color: Some(Rgba::new(tokens.accent, 0.8)),
            ..recessed
        }),
        disabled: Some(unavailable),
        ..Default::default()
    };
    tokens.visual.panel = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
        shape: Some(Shape::Rounded([10.; 4])),
        shadows: Some([
            Some(ShadowLayer {
                color: shade,
                angle: 45.,
                distance: 1.,
                blur: 3.,
            }),
            Some(ShadowLayer {
                color: light,
                angle: 225.,
                distance: 1.,
                blur: 3.,
            }),
            None,
            None,
        ]),
        ..raised
    };
    tokens.visual.selected_panel = Some(SurfaceStyle {
        border_color: Some(Rgba::new(tokens.accent, 0.6)),
        border_width: Some(1.),
        ..tokens.visual.panel
    });
    tokens.visual.search_field = Some(recessed);
    tokens.visual.tab_bar = Some(SurfaceStyle {
        shape: Some(Shape::Rounded([8., 8., 0., 0.])),
        inset: None,
        ..recessed
    });
    tokens.visual.tab = SurfaceStyle {
        fill: recessed.fill,
        border_width: Some(0.),
        shadows: Some([None; 4]),
        shape: Some(Shape::Rounded([8., 8., 0., 0.])),
        ..Default::default()
    };
    tokens.visual.selected_tab = Some(SurfaceStyle {
        fill: flat.fill,
        border_width: Some(0.),
        shadows: Some([None; 4]),
        ..tokens.visual.tab
    });
    tokens.visual.badge = SurfaceStyle {
        fill: None,
        shape: Some(Shape::Pill),
        border_width: Some(1.),
        border_color: Some(Rgba::new(tokens.border, 0.8)),
        inset: Some(InsetBevel { width: 1., ..bevel }),
        ..Default::default()
    };
    tokens.visual.slider = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 135.,
            start: Rgba::new(tokens.surface, 1.),
            end: Rgba::new(tokens.border, 1.),
        }),
        border_color: Some(Rgba::new(tokens.secondary, 0.7)),
        border_width: Some(2.),
        shape: Some(Shape::Pill),
        ..Default::default()
    };
    tokens.visual.slider_rail = SurfaceStyle {
        fill: recessed.fill,
        shape: Some(Shape::Pill),
        border_width: Some(6.),
        ..Default::default()
    };
    tokens.visual.checkbox = States {
        normal: SurfaceStyle {
            border_color: Some(Rgba::new(tokens.secondary, 0.85)),
            border_width: Some(1.5),
            shape: Some(Shape::Rounded([4.; 4])),
            inset: Some(InsetBevel {
                width: 1.5,
                ..bevel
            }),
            shadows: Some([None; 4]),
            ..Default::default()
        },
        hovered: Some(SurfaceStyle {
            border_color: Some(Rgba::new(tokens.accent, 0.8)),
            ..Default::default()
        }),
        ..Default::default()
    };
    tokens.visual.segment = Some(States {
        normal: SurfaceStyle {
            border_color: Some(Rgba::new(tokens.border, 0.8)),
            border_width: Some(1.),
            shape: Some(Shape::Rounded([6.; 4])),
            ..raised
        },
        pressed: Some(recessed),
        hovered: Some(SurfaceStyle {
            border_color: Some(Rgba::new(tokens.accent, 0.5)),
            ..flat
        }),
        disabled: Some(unavailable),
        ..Default::default()
    });
    let selected_fill = Fill::Solid(Rgba::new(
        crate::palette::packed(crate::theme::blend(tokens.surface, tokens.accent, 0.18)),
        1.,
    ));
    let selected_material = SurfaceStyle {
        fill: Some(selected_fill),
        border_color: Some(Rgba::new(tokens.accent, 0.65)),
        text_color: Some(Rgba::new(tokens.text, 1.)),
        ..toggled.normal
    };
    tokens.visual.selected_segment = Some(States {
        normal: selected_material,
        hovered: Some(selected_material),
        pressed: Some(selected_material),
        ..toggled
    });
    tokens.visual.settings_card = Some(SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(
            crate::palette::packed(crate::theme::blend(tokens.surface, tokens.text, 0.025)),
            1.,
        ))),
        shape: Some(Shape::Rounded([10.; 4])),
        border_width: Some(1.),
        border_color: Some(Rgba::new(tokens.border, 0.75)),
        shadows: Some([None; 4]),
        ..Default::default()
    });
    tokens.visual.switch_off = Some(States {
        normal: recessed,
        disabled: Some(unavailable),
        ..Default::default()
    });
    tokens.visual.switch_on = Some(States {
        normal: selected_material,
        disabled: Some(unavailable),
        ..Default::default()
    });
    tokens.visual.switch_thumb = Some(SurfaceStyle {
        shape: Some(Shape::Pill),
        fill: Some(Fill::Solid(Rgba::new(tokens.text, 1.))),
        border_width: Some(0.),
        ..Default::default()
    });
    tokens.visual.dropdown = Some(States {
        normal: raised,
        hovered: Some(raised),
        pressed: Some(raised),
        disabled: Some(unavailable),
        ..Default::default()
    });
    let soften = |c: u32| {
        let c = crate::theme::blend(c, 0xffffff, 0.28);
        rgb(
            (c.r * 255.) as u32,
            (c.g * 255.) as u32,
            (c.b * 255.) as u32,
        )
    };
    tokens.visual.progress = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 90.,
            start: Rgba::new(soften(tokens.accent), 1.),
            end: Rgba::new(tokens.accent, 1.),
        }),
        shape: Some(Shape::Pill),
        ..Default::default()
    };
    tokens.visual.scrollbar = SurfaceStyle {
        fill: Some(Fill::Linear {
            angle: 90.,
            start: Rgba::new(tokens.secondary, 0.65),
            end: Rgba::new(tokens.secondary, 0.9),
        }),
        shape: Some(Shape::Pill),
        ..Default::default()
    };
    tokens.visual.grip = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.secondary, 0.5))),
        shape: Some(Shape::Pill),
        ..Default::default()
    };
    ThemeDefinition::new(tokens)
}

/// Material-inspired surfaces: rounded controls, clear semantic colors and modest elevation.
pub fn material(mut tokens: Tokens) -> ThemeDefinition {
    let dark =
        color(tokens.background).r + color(tokens.background).g + color(tokens.background).b < 1.5;
    let rgb = |r: u32, g: u32, b: u32| r | g << 8 | b << 16;
    tokens.background = if dark {
        rgb(20, 22, 27)
    } else {
        rgb(244, 246, 250)
    };
    tokens.surface = if dark { rgb(35, 38, 45) } else { 0xffffff };
    tokens.navigation_background = tokens.background;
    tokens.border = if dark {
        rgb(72, 77, 88)
    } else {
        rgb(202, 208, 218)
    };
    let flat = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
        shape: Some(Shape::Rounded([6.; 4])),
        border_width: Some(0.),
        shadows: Some([None; 4]),
        ..Default::default()
    };
    let elevated = SurfaceStyle {
        shadows: Some([
            Some(ShadowLayer {
                color: Rgba::new(0x000000, if dark { 0.3 } else { 0.16 }),
                angle: 90.,
                distance: 1.,
                blur: 3.,
            }),
            None,
            None,
            None,
        ]),
        ..flat
    };
    let outlined = SurfaceStyle {
        border_width: Some(1.),
        border_color: Some(Rgba::new(tokens.border, 1.)),
        ..flat
    };
    let focus = SurfaceStyle {
        border_width: Some(2.),
        border_color: Some(Rgba::new(tokens.accent, 1.)),
        ..outlined
    };
    let disabled = SurfaceStyle {
        opacity: Some(0.5),
        ..flat
    };
    let button = States {
        normal: elevated,
        hovered: Some(focus),
        pressed: Some(outlined),
        disabled: Some(disabled),
        focused: Some(focus),
    };
    let toned = States {
        normal: SurfaceStyle {
            fill: None,
            ..elevated
        },
        hovered: Some(SurfaceStyle {
            fill: None,
            ..focus
        }),
        pressed: Some(SurfaceStyle {
            fill: None,
            ..outlined
        }),
        disabled: Some(disabled),
        ..Default::default()
    };
    let panel = SurfaceStyle {
        shape: Some(Shape::Rounded([10.; 4])),
        ..elevated
    };
    tokens.visual = VisualTheme {
        toggled_button: None,
        dropdown: None,
        card: outlined,
        disclosure: panel,
        panel,
        selected_panel: Some(SurfaceStyle {
            border_width: Some(1.),
            border_color: Some(Rgba::new(tokens.accent, 0.6)),
            ..panel
        }),
        button,
        important: toned,
        danger: toned,
        flat: States {
            hovered: Some(SurfaceStyle {
                fill: Some(Fill::Solid(Rgba::new(tokens.accent, 0.12))),
                ..flat
            }),
            pressed: Some(focus),
            ..Default::default()
        },
        input: States {
            normal: outlined,
            focused: Some(focus),
            disabled: Some(disabled),
            ..Default::default()
        },
        search_field: Some(outlined),
        tab_bar: Some(SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.background, 1.))),
            ..Default::default()
        }),
        tab: SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.background, 1.))),
            shape: Some(Shape::Rounded([6., 6., 0., 0.])),
            border_width: Some(0.),
            ..Default::default()
        },
        selected_tab: Some(SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.surface, 1.))),
            border_width: Some(0.),
            ..Default::default()
        }),
        checkbox: States {
            normal: SurfaceStyle {
                shape: Some(Shape::Rounded([3.; 4])),
                border_width: Some(1.5),
                border_color: Some(Rgba::new(tokens.secondary, 1.)),
                ..Default::default()
            },
            ..Default::default()
        },
        segment: Some(States {
            normal: outlined,
            disabled: Some(disabled),
            ..Default::default()
        }),
        selected_segment: Some(toned),
        progress: SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.accent, 1.))),
            shape: Some(Shape::Pill),
            ..Default::default()
        },
        badge: SurfaceStyle {
            shape: Some(Shape::Pill),
            ..Default::default()
        },
        menu: panel,
        slider: SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.accent, 1.))),
            shape: Some(Shape::Pill),
            border_width: Some(0.),
            ..Default::default()
        },
        slider_rail: SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.secondary, 0.22))),
            shape: Some(Shape::Pill),
            border_width: Some(4.),
            ..Default::default()
        },
        scrollbar: SurfaceStyle {
            shape: Some(Shape::Pill),
            ..Default::default()
        },
        ..Default::default()
    };
    let segment_base = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.background, 0.))),
        shape: Some(Shape::Pill),
        border_width: Some(0.),
        text_color: Some(Rgba::new(tokens.text, 1.)),
        shadows: Some([None; 4]),
        ..Default::default()
    };
    let selection = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(
            crate::palette::packed(crate::theme::blend(tokens.surface, tokens.accent, 0.22)),
            1.,
        ))),
        ..segment_base
    };
    tokens.visual.segment = Some(States {
        normal: segment_base,
        hovered: Some(SurfaceStyle {
            fill: Some(Fill::Solid(Rgba::new(tokens.accent, 0.08))),
            ..segment_base
        }),
        pressed: Some(selection),
        disabled: Some(disabled),
        ..Default::default()
    });
    tokens.visual.selected_segment = Some(States {
        normal: selection,
        hovered: Some(selection),
        pressed: Some(selection),
        disabled: Some(disabled),
        ..Default::default()
    });
    tokens.visual.segment_group = Some(SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(tokens.background, 1.))),
        shape: Some(Shape::Pill),
        border_width: Some(1.),
        border_color: Some(Rgba::new(tokens.secondary, 0.45)),
        ..Default::default()
    });
    tokens.visual.settings_card = Some(SurfaceStyle {
        shape: Some(Shape::Rounded([12.; 4])),
        shadows: Some([None; 4]),
        ..outlined
    });
    ThemeDefinition::new(tokens)
}
/// Metro-inspired flat surfaces, crisp rectangular controls and accent-led selection.
pub fn metro(tokens: Tokens) -> ThemeDefinition {
    let mut definition = material(tokens);
    let t = &mut definition.tokens;
    let square = |mut s: SurfaceStyle| {
        s.shape = Some(Shape::Rectangle);
        s.shadows = Some([None; 4]);
        s
    };
    let states = |s: States| States {
        normal: square(s.normal),
        hovered: s.hovered.map(square),
        pressed: s.pressed.map(square),
        disabled: s.disabled.map(square),
        focused: s.focused.map(square),
    };
    t.visual.button = states(t.visual.button);
    t.visual.important = states(t.visual.important);
    t.visual.danger = states(t.visual.danger);
    t.visual.flat = states(t.visual.flat);
    t.visual.input = states(t.visual.input);
    t.visual.checkbox = states(t.visual.checkbox);
    t.visual.panel = square(t.visual.panel);
    t.visual.selected_panel = t.visual.selected_panel.map(square);
    t.visual.card = square(t.visual.card);
    t.visual.disclosure = square(t.visual.disclosure);
    t.visual.menu = square(t.visual.menu);
    t.visual.tab = square(t.visual.tab);
    t.visual.selected_tab = t.visual.selected_tab.map(square);
    t.visual.search_field = t.visual.search_field.map(square);
    t.visual.segment = t.visual.segment.map(states);
    t.visual.selected_segment = t.visual.selected_segment.map(states);
    t.visual.badge = square(t.visual.badge);
    t.visual.progress = square(t.visual.progress);
    t.visual.scrollbar = square(t.visual.scrollbar);
    t.visual.slider = square(t.visual.slider);
    t.visual.slider_rail = square(t.visual.slider_rail);
    // Adjust tile brightness until both labels and icons have enough contrast.
    let luminance = |rgb| {
        let c = color(rgb);
        let linear = |v: f32| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    };
    let text_luminance = luminance(t.text);
    let mut tile_color = t.accent;
    for step in 0..=20 {
        let c = crate::theme::blend(
            t.accent,
            if text_luminance > 0.5 { 0 } else { 0xffffff },
            step as f64 / 20.,
        );
        tile_color = crate::palette::packed(c);
        let tile_luminance = luminance(tile_color);
        if (tile_luminance.max(text_luminance) + 0.05) / (tile_luminance.min(text_luminance) + 0.05)
            >= 5.5
        {
            break;
        }
    }
    let tile = |rgb: u32| SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(rgb, 1.))),
        text_color: Some(Rgba::new(t.text, 1.)),
        shape: Some(Shape::Rectangle),
        shadows: Some([None; 4]),
        border_width: Some(0.),
        ..Default::default()
    };
    t.visual.button.normal = tile(tile_color);
    t.visual.button.hovered = Some(SurfaceStyle {
        border_width: Some(2.),
        border_color: Some(Rgba::new(t.text, 0.55)),
        ..tile(tile_color)
    });
    t.visual.button.pressed = Some(SurfaceStyle {
        opacity: Some(0.8),
        ..tile(tile_color)
    });
    t.visual.button.focused = t.visual.button.hovered;
    t.visual.button.disabled = Some(SurfaceStyle {
        text_color: Some(Rgba::new(t.secondary, 0.55)),
        ..t.visual.button.disabled.unwrap_or_default()
    });
    t.visual.important = t.visual.button;
    t.visual.settings_card = t.visual.settings_card.map(square);
    t.visual.switch_off = Some(States {
        normal: SurfaceStyle {
            shape: Some(Shape::Rectangle),
            fill: Some(Fill::Solid(Rgba::new(t.border, 1.))),
            ..Default::default()
        },
        ..Default::default()
    });
    t.visual.switch_on = Some(States {
        normal: SurfaceStyle {
            shape: Some(Shape::Rectangle),
            fill: Some(Fill::Solid(Rgba::new(t.accent, 1.))),
            ..Default::default()
        },
        ..Default::default()
    });
    t.visual.switch_thumb = Some(SurfaceStyle {
        shape: Some(Shape::Rectangle),
        ..Default::default()
    });
    let unselected = SurfaceStyle {
        fill: Some(Fill::Solid(Rgba::new(t.background, 1.))),
        text_color: Some(Rgba::new(t.text, 1.)),
        shape: Some(Shape::Rectangle),
        border_width: Some(1.),
        border_color: Some(Rgba::new(t.secondary, 0.4)),
        shadows: Some([None; 4]),
        ..Default::default()
    };
    t.visual.segment = Some(States {
        normal: unselected,
        hovered: Some(SurfaceStyle {
            border_color: Some(Rgba::new(t.text, 0.75)),
            ..unselected
        }),
        pressed: Some(t.visual.button.normal),
        disabled: t.visual.button.disabled,
        ..Default::default()
    });
    t.visual.selected_segment = Some(t.visual.button);
    t.visual.segment_group = Some(SurfaceStyle {
        shape: Some(Shape::Rectangle),
        fill: Some(Fill::Solid(Rgba::new(t.background, 1.))),
        border_width: Some(1.),
        border_color: Some(Rgba::new(t.secondary, 0.45)),
        ..Default::default()
    });
    definition.fonts(
        iced_core::Font::with_name("Segoe UI"),
        iced_core::Font::MONOSPACE,
    )
}
