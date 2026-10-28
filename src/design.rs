// SPDX-License-Identifier: Apache-2.0
//! Logical-DIP design tokens. Display scaling belongs to Iced; density is an
//! explicit layout choice, never an automatic font or system-DPI multiplier.
pub mod space {
    pub const NONE: f32 = 0.0;
    pub const TINY: f32 = 2.0;
    pub const SMALL: f32 = 4.0;
    pub const COMPACT: f32 = 6.0;
    pub const MEDIUM: f32 = 8.0;
    pub const CONTROL: f32 = 10.0;
    pub const GUTTER: f32 = 12.0;
    pub const SECTION: f32 = 16.0;
    pub const CARD: f32 = 20.0;
    pub const DIALOG: f32 = 24.0;
    pub const LARGE: f32 = 32.0;
}
pub mod radius {
    pub const CHECKBOX: f32 = 2.0;
    pub const SMALL: f32 = 3.0;
    pub const TOOL: f32 = 4.0;
    pub const GROUP: f32 = 5.0;
    pub const CONTROL: f32 = 6.0;
    pub const CARD: f32 = 8.0;
    pub const BADGE: f32 = 11.0;
}
pub mod size {
    pub const BUTTON: f32 = 32.0;
    pub const COMPACT_BUTTON: f32 = 26.0;
    pub const SMALL_BUTTON: f32 = 28.0;
    pub const TAB: f32 = 30.0;
    pub const CHECKBOX: f32 = 15.0;
    pub const ICON: f32 = 16.0;
    pub const DISCLOSURE: f32 = 32.0;
    pub const SEGMENT: f32 = 28.0;
    pub const SEGMENT_INSET: f32 = 2.0;
    pub const BADGE: f32 = 22.0;
}
pub mod text {
    pub const TINY: f32 = 10.0;
    pub const BADGE: f32 = 10.667;
    pub const SMALL: f32 = 11.0;
    pub const CAPTION: f32 = 12.0;
    pub const CONTROL: f32 = 13.0;
    pub const BODY: f32 = 13.333;
    pub const EMPHASIS: f32 = 14.667;
    pub const HEADING: f32 = 16.0;
    pub const SUBTITLE: f32 = 19.0;
    pub const TITLE: f32 = 22.0;
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextScale {
    pub tiny: f32,
    pub badge: f32,
    pub small: f32,
    pub caption: f32,
    pub control: f32,
    pub body: f32,
    pub emphasis: f32,
    pub heading: f32,
    pub subtitle: f32,
    pub title: f32,
}
impl Default for TextScale {
    fn default() -> Self {
        Self {
            tiny: text::TINY,
            badge: text::BADGE,
            small: text::SMALL,
            caption: text::CAPTION,
            control: text::CONTROL,
            body: text::BODY,
            emphasis: text::EMPHASIS,
            heading: text::HEADING,
            subtitle: text::SUBTITLE,
            title: text::TITLE,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlProfile {
    pub height: f32,
    pub text_size: f32,
    pub horizontal_padding: f32,
    pub icon_gap: f32,
}
impl ControlProfile {
    pub const STANDARD: Self = Self {
        height: size::BUTTON,
        text_size: text::BODY,
        horizontal_padding: space::CONTROL,
        icon_gap: 5.0,
    };
    pub const COMPACT: Self = Self {
        height: size::COMPACT_BUTTON,
        text_size: text::CAPTION,
        horizontal_padding: space::COMPACT,
        icon_gap: 5.0,
    };
    pub const NAVIGATION: Self = Self {
        height: size::SMALL_BUTTON,
        ..Self::COMPACT
    };
}
impl Default for ControlProfile {
    fn default() -> Self {
        Self::STANDARD
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DesignTokens {
    pub text: TextScale,
    pub button: ControlProfile,
    pub panel_inset: f32,
    pub scroll_gap: f32,
    pub checkbox_size: f32,
    pub checkbox_gap: f32,
    pub segment_height: f32,
    pub segment_inset: f32,
}
impl Default for DesignTokens {
    fn default() -> Self {
        Self {
            text: TextScale::default(),
            button: ControlProfile::default(),
            panel_inset: space::GUTTER,
            scroll_gap: space::GUTTER,
            checkbox_size: size::CHECKBOX,
            checkbox_gap: space::MEDIUM,
            segment_height: size::SEGMENT,
            segment_inset: size::SEGMENT_INSET,
        }
    }
}

/// Shared semantic color names. Warning remains the palette's warning role;
/// destructive button compatibility aliases retain their existing red behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorRole {
    #[default]
    Text,
    Secondary,
    Accent,
    Success,
    Warning,
    Danger,
    Border,
    NavigationText,
    SelectedIcon,
    OnAccent,
    OnSuccess,
    OnWarning,
    OnDanger,
    Explicit(u32),
}
impl ColorRole {
    #[cfg(feature = "theme")]
    pub fn resolve(self, t: crate::theme::Tokens) -> u32 {
        match self {
            Self::Text => t.text,
            Self::Secondary => t.secondary,
            Self::Accent => t.accent,
            Self::Success => t.success,
            Self::Warning => t.warning,
            Self::Danger => t.failure,
            Self::Border => t.border,
            Self::NavigationText => t.navigation_text,
            Self::SelectedIcon => t.selected_icon,
            Self::OnAccent => t.accent_text,
            Self::OnSuccess => t.success_text,
            Self::OnWarning => t.warning_text,
            Self::OnDanger => t.failure_text,
            Self::Explicit(color) => color,
        }
    }
}
