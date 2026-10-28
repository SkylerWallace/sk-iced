// SPDX-License-Identifier: Apache-2.0
//! Controlled UI components for Iced.
//!
//! Applications own values, stable keys, commands and domain models. `UiContext`
//! supplies explicit appearance, typography, design defaults and motion policy.
//! Builders finish with `build()` or conversion to an `Element`; native adapters
//! preserve widget state, operations and custom styling.
//!
//! Enablement gates actions independently of visual tone. Pass the current motion
//! policy to animated components; scrolling and native control actions are immediate.
use crate::{
    animation::{Element, Renderer},
    motion::Policy,
    theme::{Tokens, blend, color},
};
use iced_core::{Border, Font, Length, Padding, Theme};
use iced_widget::{Container, button, container, row};
use std::borrow::Cow;
mod redraw_boundary;
pub use redraw_boundary::RedrawBoundary;
mod keyboard;
pub use keyboard::{FocusScope, KeyboardControl};
mod button_builder;
pub use button_builder::{Button, ButtonBorder, ButtonTone};
mod segmented;
pub use segmented::{Segment, Segmented};
mod inputs;
pub use inputs::{Checkbox, TextInput};
mod tooltips;
pub use tooltips::{Tooltip, tooltip};
mod settings_card;
pub use settings_card::SettingsCard;
mod toggle_switch;
pub use toggle_switch::ToggleSwitch;
mod settings_row;
pub use settings_row::SettingsRow;
#[derive(Clone, Copy)]
#[expect(
    clippy::large_enum_variant,
    reason = "Inline tokens preserve the Copy API without adding allocations to view construction."
)]
pub enum Palette {
    Fixed(Tokens),
    Themed(fn(&Theme) -> Tokens),
}
#[derive(Clone, Copy)]
pub struct Appearance {
    pub font: Font,
    pub palette: Palette,
}
impl Appearance {
    pub fn context(self, policy: Policy) -> UiContext {
        UiContext::new(self, policy)
    }
    pub fn fixed(font: Font, tokens: Tokens) -> Self {
        Self {
            font: tokens.assets.body.unwrap_or(font),
            palette: Palette::Fixed(tokens),
        }
    }
    pub fn themed(font: Font, tokens: fn(&Theme) -> Tokens) -> Self {
        Self {
            font,
            palette: Palette::Themed(tokens),
        }
    }
    pub fn tokens(self, theme: &Theme) -> Tokens {
        match self.palette {
            Palette::Fixed(t) => t,
            Palette::Themed(resolve) => resolve(theme),
        }
    }
}
#[derive(Clone, Copy)]
pub struct ButtonLayout {
    pub height: f32,
    pub width: Length,
    pub padding: Padding,
    /// None preserves normal left alignment; Some(Shrink) and Some(Fill) are explicit.
    pub center_x: Option<Length>,
}
impl Default for ButtonLayout {
    fn default() -> Self {
        Self {
            height: crate::design::size::BUTTON,
            width: Length::Shrink,
            padding: Padding::from([0.0, crate::design::space::CONTROL]),
            center_x: None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct ActionLayout {
    pub button: ButtonLayout,
    pub text_size: f32,
    pub wrapping: iced_core::text::Wrapping,
}
impl Default for ActionLayout {
    fn default() -> Self {
        Self {
            button: ButtonLayout::default(),
            text_size: crate::design::text::BODY,
            wrapping: iced_core::text::Wrapping::Word,
        }
    }
}
pub fn action<'a, M: Clone + 'a>(
    appearance: Appearance,
    policy: Policy,
    label: impl Into<Cow<'a, str>>,
    primary: bool,
    danger: bool,
    layout: ActionLayout,
) -> Button<'a, M> {
    Button::new(appearance, policy)
        .text(label)
        .action_layout(layout)
        .tone(if danger {
            ButtonTone::Danger
        } else if primary {
            ButtonTone::Important
        } else {
            ButtonTone::Default
        })
}

#[derive(Clone, Copy)]
pub struct CardLayout {
    pub padding: Padding,
    pub width: Length,
    pub height: Option<Length>,
    pub clip: bool,
    pub selected: bool,
}
impl Default for CardLayout {
    fn default() -> Self {
        Self {
            padding: 1.into(),
            width: Length::Fill,
            height: None,
            clip: true,
            selected: false,
        }
    }
}
pub fn card<'a, M: 'a>(
    appearance: Appearance,
    content: impl Into<Element<'a, M>>,
    layout: CardLayout,
) -> Container<'a, M, Theme, Renderer> {
    let content = container(content)
        .padding(layout.padding)
        .width(layout.width)
        .clip(layout.clip)
        .style(move |theme| appearance.tokens(theme).panel(layout.selected));
    if let Some(height) = layout.height {
        content.height(height)
    } else {
        content
    }
}
pub fn text<'a>(
    font: Font,
    content: impl iced_widget::text::IntoFragment<'a>,
) -> iced_widget::Text<'a> {
    crate::typography::Typography::new(font, iced_core::Font::MONOSPACE).raw(content)
}
pub fn checkbox<'a, M: 'a>(font: Font, checked: bool) -> iced_widget::Checkbox<'a, M> {
    iced_widget::checkbox(checked).font(font)
}
#[derive(Clone, Copy)]
pub enum CheckState {
    Unchecked,
    Checked,
    Mixed,
}
pub fn tri_state_checkbox<'a, M: 'a>(
    font: Font,
    state: CheckState,
) -> iced_widget::Checkbox<'a, M> {
    let control = checkbox(font, !matches!(state, CheckState::Unchecked));
    if matches!(state, CheckState::Mixed) {
        control.icon(iced_widget::checkbox::Icon {
            font,
            code_point: '\u{2212}',
            size: Some(14.0.into()),
            line_height: Default::default(),
            shaping: iced_core::text::Shaping::Basic,
        })
    } else {
        control
    }
}
pub fn text_input<'a, M: Clone + 'a>(
    font: Font,
    placeholder: &str,
    value: &str,
) -> iced_widget::TextInput<'a, M> {
    iced_widget::text_input(placeholder, value).font(font)
}
pub fn themed_text_input<'a, M: Clone + 'a>(
    appearance: Appearance,
    placeholder: &str,
    value: &str,
) -> iced_widget::TextInput<'a, M> {
    text_input(appearance.font, placeholder, value).style(move |theme: &Theme, status| {
        let t = appearance.tokens(theme);
        let mut style = iced_widget::text_input::default(theme, status);
        style.background = color(t.background).into();
        style.border.color = color(
            if matches!(status, iced_widget::text_input::Status::Focused { .. }) {
                t.accent
            } else {
                t.border
            },
        );
        style.value = color(t.text);
        style.placeholder = color(t.secondary);
        style.selection = color(t.accent);
        let surface = match status {
            iced_widget::text_input::Status::Focused { .. } => t.visual.input.focused,
            iced_widget::text_input::Status::Hovered => t.visual.input.hovered,
            iced_widget::text_input::Status::Disabled => t.visual.input.disabled,
            _ => None,
        }
        .unwrap_or_default()
        .inherit(t.visual.input.normal);
        let mut background = Some(style.background);
        surface.apply(&mut background, &mut style.border, &mut Default::default());
        style.background = background.unwrap();
        if let Some(color) = surface.text_color {
            style.value = color.color();
        }
        style
    })
}
pub fn icon_label<'a, M: 'a>(
    font: Font,
    graphic: impl Into<Element<'a, M>>,
    label: Option<Cow<'a, str>>,
    text_size: f32,
    spacing: f32,
) -> iced_widget::Row<'a, M> {
    let mut content = row![graphic.into()]
        .spacing(spacing)
        .align_y(iced_core::Alignment::Center);
    if let Some(label) = label {
        content = content.push(text(font, label).size(text_size));
    }
    content
}
pub fn shortcut_hint<'a, M: 'a>(
    appearance: Appearance,
    label: impl Into<Cow<'a, str>>,
    shortcut: &str,
) -> Element<'a, M> {
    let mut parts = row![text(appearance.font, label.into()).size(12)]
        .spacing(5)
        .align_y(iced_core::Alignment::Center);
    if !shortcut.is_empty() {
        parts = parts.push(
            text(appearance.font, format!("({shortcut})"))
                .size(12)
                .style(move |theme: &Theme| iced_widget::text::Style {
                    color: Some(color(appearance.tokens(theme).info)),
                }),
        );
    }
    parts.into()
}
/// Unframed tool controls: no app-specific icon or command knowledge.
pub fn panel_control_style(t: Tokens, status: button::Status) -> button::Style {
    button::Style {
        text_color: color(if status == button::Status::Hovered {
            t.text
        } else {
            t.secondary
        }),
        background: if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(blend(t.surface, t.text, t.opacity_hover).into())
        } else {
            None
        },
        border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

mod layouts;
pub use layouts::{ExpandableCard, Navigation, NavigationAxis, Panel, Tab, TabBar, TabSizing};

mod indicators;
pub use indicators::{Progress, StatusBadge, progress};

mod selectors;
pub use selectors::{Dropdown, SearchableDropdown, dropdown};

mod popup;
pub use popup::Popup;

mod shells;
pub use shells::{Dialog, Menu};

pub mod scroll;
pub use scroll::{
    HorizontalScrollRail, LocalScroll, RailVisual, ScrollAction, ScrollArea, ScrollRail,
    ScrollReplay, Scrollbar, scroll_area,
};

mod context;
pub use context::UiContext;
pub mod virtual_list;
pub use virtual_list::VirtualList;
mod table_header;
pub use table_header::{HeaderResize, TableHeader};
mod column_resize;
pub use column_resize::ColumnResize;
mod data;
pub use data::{DataTable, TableColumns, TableRow, adjacent, select};
mod tree;
pub use tree::{TreeNavigation, TreeRow, TreeView, tree_navigation};
mod tab_viewport;
pub use tab_viewport::TabViewport;
mod toolbar_fit;
pub use toolbar_fit::AdaptiveToolbar;
mod tab_surface;
pub use tab_surface::TabSurface;
pub mod drag;
pub use drag::{DragGesture, DragRegion, DragRegions};
pub mod split;
pub use split::{DockEdge, SplitLayout, SplitModel};
mod chrome;
pub use chrome::{AppShell, NavigationDrawer, NavigationItem, Toolbar};

mod drag_handle;
pub use drag_handle::{DragHandle, DragVisibility};
mod drop_indicator;
pub use drop_indicator::DropIndicator;
mod gate;
pub use gate::InteractionGate;
mod forms;
pub use forms::{ChoiceGrid, FormField, FormSection, KeyValueRow, Slider};
mod feedback;
pub use feedback::{DialogContent, EmptyState, Notice, PreviewSurface};
mod share_meter;
pub use share_meter::ShareMeter;
mod fill_clip;
pub use fill_clip::FillClip;
mod labeled_progress;
pub use labeled_progress::LabeledProgress;
mod read_only;
pub use read_only::{ReadOnlyAction, ReadOnlyText, perform_read_only};
mod resize_grip;
pub use resize_grip::ResizeGrip;
mod menu_item;
pub use menu_item::MenuItem;
mod dialog_support;
pub use dialog_support::{DialogBackdrop, DialogCommands};

mod surface;
pub use surface::{Surface, SurfaceRole};

impl Default for Appearance {
    fn default() -> Self {
        Self::themed(Font::DEFAULT, crate::palette::from_theme)
    }
}

mod native_surface;

mod inset;
