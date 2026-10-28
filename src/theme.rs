// SPDX-License-Identifier: Apache-2.0
//! Semantic styles shared by all Iced pages, using the owned Rust palette.
use iced_core::{Color, Theme};
use iced_widget::{button, checkbox, container, progress_bar, scrollable};
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Tokens {
    #[cfg_attr(feature = "serde", serde(default))]
    pub visual: crate::visual::VisualTheme,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub assets: crate::visual::ThemeAssets,
    pub background: u32,
    pub surface: u32,
    pub text: u32,
    pub secondary: u32,
    pub border: u32,
    pub accent: u32,
    pub success: u32,
    pub failure: u32,
    pub warning: u32,
    pub navigation_text: u32,
    pub selected_icon: u32,
    pub accent_text: u32,
    pub success_text: u32,
    pub failure_text: u32,
    pub warning_text: u32,
    pub navigation_background: u32,
    pub navigation_overlay: u32,
    pub meter_start: u32,
    pub meter_end: u32,
    pub info: u32,
    pub light_text: u32,
    pub dark_text: u32,
    pub opacity_disabled: f64,
    pub opacity_hover: f64,
    pub opacity_pressed: f64,
    pub opacity_focused: f64,
    pub opacity_selected: f64,
    pub opacity_border: f64,
    pub opacity_dragged: f64,
    pub opacity_divider: f64,
}
pub fn color(rgb: u32) -> Color {
    Color::from_rgb8(rgb as u8, (rgb >> 8) as u8, (rgb >> 16) as u8)
}
pub fn blend(base: u32, ink: u32, amount: f64) -> Color {
    let channel = |shift: u32| {
        (((base >> shift) & 255) as f64 * (1.0 - amount) + ((ink >> shift) & 255) as f64 * amount)
            as u8
    };
    Color::from_rgb8(channel(0), channel(8), channel(16))
}
impl Tokens {
    pub fn flat(self) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |_, status| {
            self.visual.flat.button(status).button_style(button::Style {
                background: Some(
                    blend(
                        self.surface,
                        self.text,
                        match status {
                            button::Status::Hovered => self.opacity_hover,
                            button::Status::Pressed => self.opacity_pressed,
                            _ => 0.0,
                        },
                    )
                    .into(),
                ),
                text_color: color(self.text),
                border: iced_core::Border {
                    radius: crate::design::radius::CONTROL.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        }
    }
    pub fn flat_dimmed(self) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |_, status| {
            self.visual.flat.button(status).button_style(button::Style {
                background: Some(
                    blend(
                        self.surface,
                        self.text,
                        match status {
                            button::Status::Hovered => self.opacity_hover,
                            button::Status::Pressed => self.opacity_pressed,
                            _ => 0.0,
                        },
                    )
                    .into(),
                ),
                text_color: color(self.text).scale_alpha(if status == button::Status::Disabled {
                    self.opacity_disabled as f32
                } else {
                    1.0
                }),
                border: iced_core::Border {
                    radius: crate::design::radius::CONTROL.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        }
    }
    pub fn header(self, hovered: bool, expanded: bool) -> container::Style {
        self.visual.header.container_style(container::Style {
            background: Some(
                blend(
                    self.surface,
                    self.text,
                    if hovered { self.opacity_hover } else { 0.0 },
                )
                .into(),
            ),
            border: iced_core::Border {
                radius: iced_core::border::Radius {
                    top_left: 6.0,
                    top_right: 6.0,
                    bottom_left: if expanded { 0.0 } else { 6.0 },
                    bottom_right: if expanded { 0.0 } else { 6.0 },
                },
                ..Default::default()
            },
            ..Default::default()
        })
    }
    pub fn contrast(self, background: u32) -> Color {
        let linear = |shift: u32| {
            let v = ((background >> shift) & 255) as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let luminance = 0.2126 * linear(0) + 0.7152 * linear(8) + 0.0722 * linear(16);
        color(if 1.05 / (luminance + 0.05) >= 3.0 {
            self.light_text
        } else {
            self.dark_text
        })
    }
    pub fn card(self, selected: bool) -> container::Style {
        self.visual.card.container_style(container::Style {
            background: Some(color(self.surface).into()),
            text_color: Some(color(self.text)),
            border: iced_core::Border {
                color: color(if selected { self.accent } else { self.border }),
                width: 1.0,
                radius: crate::design::radius::CONTROL.into(),
            },
            ..Default::default()
        })
    }
    pub fn panel(self, selected: bool) -> container::Style {
        (if selected {
            self.visual
                .selected_panel
                .unwrap_or_default()
                .inherit(self.visual.panel)
        } else {
            self.visual.panel
        })
        .container_style(self.card(selected))
    }
    pub fn toggle_button(self, selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |theme, status| {
            let style = self.button(selected, false)(theme, status);
            if selected {
                self.visual
                    .toggled_button
                    .map(|s| s.button(status).button_style(style))
                    .unwrap_or(style)
            } else {
                style
            }
        }
    }
    pub fn button(
        self,
        primary: bool,
        danger: bool,
    ) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |_, status| {
            let disabled = status == button::Status::Disabled;
            let base = if disabled {
                self.surface
            } else if danger {
                self.failure
            } else if primary {
                self.accent
            } else {
                self.surface
            };
            let ink = if disabled {
                self.text
            } else if danger {
                self.failure_text
            } else if primary {
                self.accent_text
            } else {
                self.text
            };
            let amount = match status {
                button::Status::Hovered => self.opacity_hover,
                button::Status::Pressed => self.opacity_pressed,
                _ => 0.0,
            };
            (if danger {
                self.visual.danger
            } else if primary {
                self.visual.important
            } else {
                self.visual.button
            })
            .button(status)
            .button_style(button::Style {
                background: Some(blend(base, ink, amount).into()),
                text_color: if disabled {
                    blend(base, ink, self.opacity_disabled)
                } else {
                    color(ink)
                },
                border: iced_core::Border {
                    color: blend(base, ink, self.opacity_border),
                    width: 1.0,
                    radius: crate::design::radius::CONTROL.into(),
                },
                ..Default::default()
            })
        }
    }
    pub fn check(self) -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
        move |_, status| {
            let (checked, disabled, hover) = match status {
                checkbox::Status::Active { is_checked } => (is_checked, false, false),
                checkbox::Status::Hovered { is_checked } => (is_checked, false, true),
                checkbox::Status::Disabled { is_checked } => (is_checked, true, false),
            };
            let ink = if disabled {
                blend(self.surface, self.text, self.opacity_disabled)
            } else {
                color(self.text)
            };
            {
                let mut style = checkbox::Style {
                    background: if checked {
                        color(self.accent).into()
                    } else {
                        blend(
                            self.surface,
                            self.text,
                            if hover { self.opacity_hover } else { 0.0 },
                        )
                        .into()
                    },
                    icon_color: color(self.accent_text),
                    text_color: Some(ink),
                    border: iced_core::Border {
                        color: if checked {
                            color(self.accent)
                        } else if disabled {
                            ink
                        } else {
                            color(self.border)
                        },
                        width: 1.0,
                        radius: crate::design::radius::CHECKBOX.into(),
                    },
                };
                let surface = (if disabled {
                    self.visual.checkbox.disabled
                } else if hover {
                    self.visual.checkbox.hovered
                } else {
                    None
                })
                .unwrap_or_default()
                .inherit(self.visual.checkbox.normal);
                let mut bg = Some(style.background);
                surface.apply(&mut bg, &mut style.border, &mut Default::default());
                style.background = bg.unwrap();
                style
            }
        }
    }
    pub fn progress(self, tone: u32) -> impl Fn(&Theme) -> progress_bar::Style {
        move |_| {
            let mut style = progress_bar::Style {
                background: color(self.border).into(),
                bar: color(tone).into(),
                border: iced_core::Border {
                    radius: crate::design::radius::CONTROL.into(),
                    ..Default::default()
                },
            };
            let surface = self.visual.progress;
            let mut bar = Some(style.bar);
            surface.apply(&mut bar, &mut style.border, &mut Default::default());
            style.bar = bar.unwrap();
            style
        }
    }
    pub fn segment(
        self,
        selected: bool,
        warning: bool,
    ) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |_, status| {
            let disabled = status == button::Status::Disabled;
            let bg = if selected {
                if disabled {
                    self.border
                } else if warning {
                    self.warning
                } else {
                    self.accent
                }
            } else if status == button::Status::Hovered {
                self.surface
            } else {
                self.background
            };
            let ink = if disabled {
                self.secondary
            } else if selected {
                if warning {
                    self.warning_text
                } else {
                    self.accent_text
                }
            } else {
                self.text
            };
            (if selected {
                self.visual
                    .selected_segment
                    .unwrap_or(self.visual.important)
            } else {
                self.visual.segment.unwrap_or(self.visual.button)
            })
            .button(status)
            .button_style(button::Style {
                background: Some(color(bg).into()),
                text_color: color(ink),
                border: iced_core::Border {
                    radius: crate::design::radius::SMALL.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
        }
    }
    pub fn scroll(self) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
        move |theme, status| {
            let mut style = scrollable::default(theme, status);
            style.container = container::Style::default();
            style.vertical_rail.background = Some(color(self.border).into());
            style.vertical_rail.border = iced_core::Border::default();
            style.vertical_rail.scroller.background = color(self.secondary).into();
            style.vertical_rail.scroller.border = iced_core::Border {
                radius: crate::design::radius::TOOL.into(),
                ..Default::default()
            };
            let surface = self.visual.scrollbar;
            let mut bg = Some(style.vertical_rail.scroller.background);
            surface.apply(
                &mut bg,
                &mut style.vertical_rail.scroller.border,
                &mut Default::default(),
            );
            style.vertical_rail.scroller.background = bg.unwrap();
            style.horizontal_rail = style.vertical_rail;
            style
        }
    }
    pub fn outcome(self, status: &str) -> u32 {
        match status {
            "passed" => self.success,
            "failed" | "error" => self.failure,
            "warning" | "skipped" | "unsupported" | "cancelled" | "stopped" => self.warning,
            "testing" | "running" => self.accent,
            _ => self.secondary,
        }
    }
    pub fn theme(self) -> Theme {
        let theme = Theme::custom_with_fn(
            "sk-iced",
            iced_core::theme::Palette {
                background: color(self.background),
                text: color(self.text),
                primary: color(self.accent),
                success: color(self.success),
                danger: color(self.failure),
                warning: color(self.warning),
            },
            move |palette| {
                let mut extended = iced_core::theme::palette::Extended::generate(palette);
                let pair = |bg, fg| iced_core::theme::palette::Pair {
                    color: color(bg),
                    text: color(fg),
                };
                // Keep derived shades aligned with the semantic surface roles.
                extended.background.base = pair(self.background, self.text);
                for p in [
                    &mut extended.background.weakest,
                    &mut extended.background.weaker,
                    &mut extended.background.weak,
                    &mut extended.background.neutral,
                ] {
                    *p = pair(self.surface, self.text);
                }
                for p in [
                    &mut extended.background.strong,
                    &mut extended.background.stronger,
                    &mut extended.background.strongest,
                ] {
                    *p = pair(self.border, self.secondary);
                }
                for p in [
                    &mut extended.secondary.base,
                    &mut extended.secondary.weak,
                    &mut extended.secondary.strong,
                ] {
                    *p = pair(self.surface, self.text);
                }
                for p in [
                    &mut extended.primary.base,
                    &mut extended.primary.weak,
                    &mut extended.primary.strong,
                ] {
                    *p = pair(self.accent, self.accent_text);
                }
                for p in [
                    &mut extended.success.base,
                    &mut extended.success.weak,
                    &mut extended.success.strong,
                ] {
                    *p = pair(self.success, self.success_text);
                }
                for p in [
                    &mut extended.danger.base,
                    &mut extended.danger.weak,
                    &mut extended.danger.strong,
                ] {
                    *p = pair(self.failure, self.failure_text);
                }
                for p in [
                    &mut extended.warning.base,
                    &mut extended.warning.weak,
                    &mut extended.warning.strong,
                ] {
                    *p = pair(self.warning, self.warning_text);
                }
                extended
            },
        );
        crate::visual::remember(&theme, self);
        theme
    }
    pub fn surface(self) -> container::Style {
        self.visual.background.container_style(container::Style {
            background: Some(color(self.navigation_background).into()),
            text_color: Some(color(self.navigation_text)),
            ..Default::default()
        })
    }
}
