// SPDX-License-Identifier: Apache-2.0
// Palette extracted from PC Tonic native_services/mod.rs.
pub fn palette(dark: bool, contrast: bool) -> crate::theme::Tokens {
    let rgb = |r: u32, g: u32, b: u32| r | g << 8 | b << 16;
    let p = if dark {
        [
            [17, 22, 31],
            [27, 34, 46],
            [233, 239, 249],
            [163, 177, 197],
            [55, 68, 87],
            [55, 139, 250],
            [50, 195, 125],
            [245, 90, 105],
            [240, 181, 70],
        ]
    } else {
        [
            [244, 246, 250],
            [255, 255, 255],
            [26, 33, 45],
            [88, 100, 117],
            [208, 215, 225],
            [0, 105, 220],
            [20, 135, 80],
            [200, 45, 60],
            [160, 105, 0],
        ]
    };
    let p = p.map(|[r, g, b]| rgb(r, g, b));
    let [
        background,
        surface,
        mut text,
        mut secondary,
        border,
        accent,
        success,
        failure,
        warning,
    ] = p;
    let white = rgb(255, 255, 255);
    if contrast {
        text = if dark { white } else { 0 };
        secondary = text;
    }
    crate::theme::Tokens {
        visual: Default::default(),
        assets: Default::default(),
        background,
        surface,
        text,
        secondary,
        border,
        accent,
        success,
        failure,
        warning,
        navigation_text: if dark { white } else { text },
        selected_icon: if dark { white } else { 0 },
        accent_text: white,
        success_text: if dark { 0 } else { white },
        failure_text: white,
        warning_text: if dark { background } else { white },
        navigation_background: if dark {
            rgb(12, 16, 23)
        } else {
            rgb(232, 237, 244)
        },
        navigation_overlay: accent,
        meter_start: accent,
        meter_end: if dark {
            rgb(175, 110, 250)
        } else {
            rgb(115, 65, 195)
        },
        info: accent,
        light_text: white,
        dark_text: 0,
        opacity_disabled: 0.38,
        opacity_hover: 0.08,
        opacity_pressed: 0.10,
        opacity_focused: 0.10,
        opacity_selected: 0.12,
        opacity_border: 0.12,
        opacity_dragged: 0.16,
        opacity_divider: 0.38,
    }
}

/// Six traditional color families, three tonal variants per family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Accent(u8);
impl Accent {
    pub const ALL: [Self; 18] = [
        Self(0),
        Self(1),
        Self(2),
        Self(3),
        Self(4),
        Self(5),
        Self(6),
        Self(7),
        Self(8),
        Self(9),
        Self(10),
        Self(11),
        Self(12),
        Self(13),
        Self(14),
        Self(15),
        Self(16),
        Self(17),
    ];
    pub fn key(self) -> &'static str {
        Self::NAMES[self.0 as usize]
    }
    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|a| a.key() == key)
            .unwrap_or_default()
    }
    const NAMES: [&'static str; 18] = [
        "Blue",
        "Red",
        "Yellow",
        "Orange",
        "Green",
        "Purple",
        "Light blue",
        "Light red",
        "Light yellow",
        "Light orange",
        "Light green",
        "Light purple",
        "Deep blue",
        "Deep red",
        "Deep yellow",
        "Deep orange",
        "Deep green",
        "Deep purple",
    ];
    pub fn rgb(self) -> u32 {
        let [r, g, b]: [u32; 3] = [
            [0, 105, 220],
            [200, 45, 60],
            [185, 145, 0],
            [210, 95, 15],
            [30, 140, 80],
            [135, 70, 195],
            [85, 165, 250],
            [245, 120, 130],
            [245, 210, 75],
            [250, 165, 85],
            [95, 200, 140],
            [185, 140, 235],
            [35, 70, 155],
            [145, 30, 50],
            [125, 95, 0],
            [150, 65, 10],
            [20, 95, 55],
            [90, 40, 145],
        ][self.0 as usize];
        r | g << 8 | b << 16
    }
}
pub(crate) fn packed(c: iced_core::Color) -> u32 {
    (c.r * 255.).round() as u32
        | ((c.g * 255.).round() as u32) << 8
        | ((c.b * 255.).round() as u32) << 16
}
pub fn with_tint(dark: bool, accent: Accent, tinted: bool) -> crate::theme::Tokens {
    let mut t = palette(dark, false);
    let ink = accent.rgb();
    // Low-chroma neutral surfaces keep content legible and let the accent lead.
    let tint = |gray, amount| {
        if tinted {
            packed(crate::theme::blend(gray, ink, amount))
        } else {
            gray
        }
    };
    t.background = tint(if dark { 0x181818 } else { 0xf7f7f7 }, 0.035);
    t.surface = tint(if dark { 0x242424 } else { 0xffffff }, 0.04);
    t.navigation_background = tint(if dark { 0x121212 } else { 0xeeeeee }, 0.04);
    t.border = tint(if dark { 0x484848 } else { 0xd6d6d6 }, 0.08);
    t.accent = if dark {
        packed(crate::theme::blend(ink, 0xffffff, 0.28))
    } else {
        ink
    };
    let c = crate::theme::color(t.accent);
    let linear = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b);
    t.accent_text = if luminance > 0.179 { 0 } else { 0xffffff };
    t.navigation_overlay = t.accent;
    t.meter_start = t.accent;
    t.meter_end = packed(crate::theme::blend(
        t.accent,
        if dark { 0xffffff } else { 0 },
        0.22,
    ));
    t.info = t.accent;
    t
}
/// Recover custom tokens for reusable controls that receive only an Iced theme.
pub fn from_theme(theme: &iced_core::Theme) -> crate::theme::Tokens {
    if let Some(tokens) = crate::visual::recover(theme) {
        return tokens;
    }
    let p = theme.palette();
    let dark = p.background.r + p.background.g + p.background.b < 1.5;
    let accent = Accent::ALL
        .into_iter()
        .min_by(|a, b| {
            let distance = |a: Accent| {
                let ink = a.rgb();
                let c = if dark {
                    crate::theme::blend(ink, 0xffffff, 0.28)
                } else {
                    crate::theme::color(ink)
                };
                (c.r - p.primary.r).powi(2)
                    + (c.g - p.primary.g).powi(2)
                    + (c.b - p.primary.b).powi(2)
            };
            distance(*a).total_cmp(&distance(*b))
        })
        .unwrap_or_default();
    let tinted = p.background.r != p.background.g || p.background.g != p.background.b;
    with_tint(dark, accent, tinted)
}
