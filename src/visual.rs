// SPDX-License-Identifier: Apache-2.0
//! Theme-owned surface descriptions. No component knows a preset's name.
use crate::theme::{Tokens, color};
use iced_core::{Background, Border, Color, Font, Shadow, Vector, border::Radius, gradient};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rgba {
    pub rgb: u32,
    pub alpha: f32,
}
impl Rgba {
    pub const fn rgb(red: u8, green: u8, blue: u8, alpha: f32) -> Self {
        Self::new(
            red as u32 | (green as u32) << 8 | (blue as u32) << 16,
            alpha,
        )
    }
    pub const fn new(rgb: u32, alpha: f32) -> Self {
        Self { rgb, alpha }
    }
    pub fn color(self) -> Color {
        color(self.rgb).scale_alpha(self.alpha.clamp(0., 1.))
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fill {
    Solid(Rgba),
    Linear { angle: f32, start: Rgba, end: Rgba },
}
impl Fill {
    pub fn background(self, opacity: f32) -> Background {
        match self {
            Self::Solid(c) => c.color().scale_alpha(opacity).into(),
            Self::Linear { angle, start, end } => {
                gradient::Linear::new(iced_core::Radians(angle.to_radians()))
                    .add_stop(0., start.color().scale_alpha(opacity))
                    .add_stop(1., end.color().scale_alpha(opacity))
                    .into()
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Shape {
    Rectangle,
    Rounded([f32; 4]),
    Pill,
}
impl Shape {
    pub fn radius(self) -> Radius {
        match self {
            Self::Rectangle => 0.0.into(),
            Self::Pill => 10000.0.into(),
            Self::Rounded([top_left, top_right, bottom_right, bottom_left]) => Radius {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            },
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShadowLayer {
    pub color: Rgba,
    /// Degrees clockwise from the positive X axis.
    pub angle: f32,
    pub distance: f32,
    pub blur: f32,
}
impl ShadowLayer {
    pub fn native(self, opacity: f32) -> Shadow {
        let radians = self.angle.to_radians();
        Shadow {
            color: self.color.color().scale_alpha(opacity),
            offset: Vector::new(radians.cos() * self.distance, radians.sin() * self.distance),
            blur_radius: self.blur.max(0.),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[derive(Default)]
pub struct SurfaceStyle {
    pub fill: Option<Fill>,
    pub shape: Option<Shape>,
    pub border_color: Option<Rgba>,
    pub text_color: Option<Rgba>,
    pub border_width: Option<f32>,
    /// At most four shadow layers, in back-to-front order.
    pub shadows: Option<[Option<ShadowLayer>; 4]>,
    /// Surface opacity; text and content retain their independent contrast.
    pub opacity: Option<f32>,
    pub inset: Option<InsetBevel>,
}
impl SurfaceStyle {
    /// Layout space needed to contain every visible shadow at any display scale.
    /// Reserve this inside a scroll viewport instead of allowing paint to escape it.
    pub fn shadow_padding(self) -> iced_core::Padding {
        let mut padding = iced_core::Padding::ZERO;
        if self.opacity() == 0. {
            return padding;
        }
        for layer in self
            .layers()
            .iter()
            .flatten()
            .filter(|layer| layer.color.alpha > 0.)
        {
            let shadow = layer.native(self.opacity());
            let blur = shadow.blur_radius;
            padding.left = padding.left.max((blur - shadow.offset.x).max(0.).ceil());
            padding.right = padding.right.max((blur + shadow.offset.x).max(0.).ceil());
            padding.top = padding.top.max((blur - shadow.offset.y).max(0.).ceil());
            padding.bottom = padding.bottom.max((blur + shadow.offset.y).max(0.).ceil());
        }
        padding
    }
    pub fn opacity(self) -> f32 {
        self.opacity.unwrap_or(1.).clamp(0., 1.)
    }
    pub fn layers(self) -> [Option<ShadowLayer>; 4] {
        self.shadows.unwrap_or([None; 4])
    }
    pub fn inherit(self, base: Self) -> Self {
        Self {
            fill: self.fill.or(base.fill),
            shape: self.shape.or(base.shape),
            border_color: self.border_color.or(base.border_color),
            text_color: self.text_color.or(base.text_color),
            border_width: self.border_width.or(base.border_width),
            shadows: self.shadows.or(base.shadows),
            opacity: self.opacity.or(base.opacity),
            inset: self.inset.or(base.inset),
        }
    }
    pub fn button_style(self, mut style: iced_widget::button::Style) -> iced_widget::button::Style {
        self.apply(&mut style.background, &mut style.border, &mut style.shadow);
        if let Some(color) = self.text_color {
            style.text_color = color.color().scale_alpha(self.opacity());
        }
        style
    }
    pub fn container_style(
        self,
        mut style: iced_widget::container::Style,
    ) -> iced_widget::container::Style {
        self.apply(&mut style.background, &mut style.border, &mut style.shadow);
        if let Some(color) = self.text_color {
            style.text_color = Some(color.color().scale_alpha(self.opacity()));
        }
        style
    }
    pub fn apply(
        self,
        background: &mut Option<Background>,
        border: &mut Border,
        shadow: &mut Shadow,
    ) {
        if let Some(fill) = self.fill {
            *background = Some(fill.background(self.opacity()));
        } else if let Some(fill) = background {
            *fill = fill.scale_alpha(self.opacity());
        }
        if let Some(shape) = self.shape {
            border.radius = shape.radius();
        }
        if let Some(color) = self.border_color {
            border.color = color.color().scale_alpha(self.opacity());
        }
        if let Some(width) = self.border_width {
            border.width = width.max(0.);
        }
        if let Some(layers) = self.shadows {
            *shadow = layers[0]
                .map(|layer| layer.native(self.opacity()))
                .unwrap_or_default();
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct States {
    pub normal: SurfaceStyle,
    pub hovered: Option<SurfaceStyle>,
    pub pressed: Option<SurfaceStyle>,
    pub disabled: Option<SurfaceStyle>,
    pub focused: Option<SurfaceStyle>,
}
impl States {
    pub fn button(self, status: iced_widget::button::Status) -> SurfaceStyle {
        use iced_widget::button::Status::*;
        match status {
            Hovered => self.hovered,
            Pressed => self.pressed,
            Disabled => self.disabled,
            Active => None,
        }
        .unwrap_or_default()
        .inherit(self.normal)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VisualTheme {
    pub card: SurfaceStyle,
    pub settings_card: Option<SurfaceStyle>,
    /// Keyboard-only focus decoration; None uses a two-DIP accent outline.
    pub focus_ring: Option<SurfaceStyle>,
    pub switch_off: Option<States>,
    pub switch_on: Option<States>,
    pub switch_thumb: Option<SurfaceStyle>,
    pub disclosure: SurfaceStyle,
    /// Composite input surface, including leading and trailing adornments.
    pub search_field: Option<SurfaceStyle>,
    /// Background of a draggable tab strip.
    pub tab_bar: Option<SurfaceStyle>,
    pub grip: SurfaceStyle,
    pub segment: Option<States>,
    pub segment_group: Option<SurfaceStyle>,
    pub selected_segment: Option<States>,
    pub background: SurfaceStyle,
    pub button: States,
    /// Persistent toggle state, independent of action importance.
    pub toggled_button: Option<States>,
    pub dropdown: Option<States>,
    pub important: States,
    pub danger: States,
    pub flat: States,
    pub input: States,
    pub checkbox: States,
    pub panel: SurfaceStyle,
    pub selected_panel: Option<SurfaceStyle>,
    pub header: SurfaceStyle,
    pub tab: SurfaceStyle,
    pub selected_tab: Option<SurfaceStyle>,
    pub badge: SurfaceStyle,
    pub progress: SurfaceStyle,
    pub menu: SurfaceStyle,
    pub slider: SurfaceStyle,
    pub slider_rail: SurfaceStyle,
    pub scrollbar: SurfaceStyle,
}

/// Prepared assets are supplied by the application; no files are loaded during paint.
#[derive(Debug, Clone, Copy)]
pub enum IconAsset {
    Svg(&'static [u8]),
    Rgba {
        width: u32,
        height: u32,
        pixels: &'static [u8],
    },
    Glyph {
        font: Font,
        character: char,
    },
}
pub type IconPack = fn(&str) -> Option<IconAsset>;
#[derive(Debug, Clone, Copy, Default)]
pub struct ThemeAssets {
    pub body: Option<Font>,
    pub monospace: Option<Font>,
    pub icons: Option<IconPack>,
}
impl PartialEq for ThemeAssets {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
            && self.monospace == other.monospace
            && match (self.icons, other.icons) {
                (None, None) => true,
                (Some(a), Some(b)) => std::ptr::fn_addr_eq(a, b),
                _ => false,
            }
    }
}
#[derive(Clone, Copy)]
pub struct ThemeDefinition {
    pub tokens: Tokens,
    pub design: crate::design::DesignTokens,
}
impl ThemeDefinition {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens,
            design: Default::default(),
        }
    }
    pub fn fonts(mut self, body: Font, monospace: Font) -> Self {
        self.tokens.assets.body = Some(body);
        self.tokens.assets.monospace = Some(monospace);
        self
    }
    pub fn icons(mut self, icons: IconPack) -> Self {
        self.tokens.assets.icons = Some(icons);
        self
    }
    /// Apply a subtle accent wash to surface materials, preserving text and semantic inks.
    pub fn tint_surfaces(mut self, ink: u32, amount: f32) -> Self {
        let t = &mut self.tokens;
        let semantic = [t.accent, t.failure, t.success, t.info, t.warning];
        let tint = |rgb: u32| {
            if semantic.contains(&rgb) {
                rgb
            } else {
                crate::palette::packed(crate::theme::blend(rgb, ink, amount.clamp(0., 0.15).into()))
            }
        };
        t.background = tint(t.background);
        t.surface = tint(t.surface);
        t.navigation_background = tint(t.navigation_background);
        t.border = tint(t.border);
        let rgba = |mut c: Rgba| {
            c.rgb = tint(c.rgb);
            c
        };
        let surface = |s: &mut SurfaceStyle| {
            s.fill = s.fill.map(|f| match f {
                Fill::Solid(c) => Fill::Solid(rgba(c)),
                Fill::Linear { angle, start, end } => Fill::Linear {
                    angle,
                    start: rgba(start),
                    end: rgba(end),
                },
            });
            s.border_color = s.border_color.map(rgba);
            s.shadows = s.shadows.map(|shadows| {
                shadows.map(|s| {
                    s.map(|mut s| {
                        s.color = rgba(s.color);
                        s
                    })
                })
            });
            s.inset = s.inset.map(|mut b| {
                b.light = rgba(b.light);
                b.shade = rgba(b.shade);
                b
            });
        };
        let states = |s: &mut States| {
            surface(&mut s.normal);
            for s in [
                &mut s.hovered,
                &mut s.pressed,
                &mut s.disabled,
                &mut s.focused,
            ]
            .into_iter()
            .flatten()
            {
                surface(s);
            }
        };
        let v = &mut t.visual;
        for s in [
            &mut v.card,
            &mut v.disclosure,
            &mut v.grip,
            &mut v.background,
            &mut v.panel,
            &mut v.header,
            &mut v.tab,
            &mut v.badge,
            &mut v.progress,
            &mut v.menu,
            &mut v.slider,
            &mut v.slider_rail,
            &mut v.scrollbar,
        ] {
            surface(s);
        }
        for s in [
            &mut v.settings_card,
            &mut v.focus_ring,
            &mut v.switch_thumb,
            &mut v.segment_group,
            &mut v.search_field,
            &mut v.tab_bar,
            &mut v.selected_panel,
            &mut v.selected_tab,
        ]
        .into_iter()
        .flatten()
        {
            surface(s);
        }
        for s in [
            &mut v.button,
            &mut v.important,
            &mut v.danger,
            &mut v.flat,
            &mut v.input,
            &mut v.checkbox,
        ] {
            states(s);
        }
        for s in [
            &mut v.segment,
            &mut v.selected_segment,
            &mut v.toggled_button,
            &mut v.dropdown,
            &mut v.switch_off,
            &mut v.switch_on,
        ]
        .into_iter()
        .flatten()
        {
            states(s);
        }
        self
    }
    pub fn visuals(mut self, visuals: VisualTheme) -> Self {
        self.tokens.visual = visuals;
        self
    }
    pub fn design(mut self, design: crate::design::DesignTokens) -> Self {
        self.design = design;
        self
    }
    #[cfg(feature = "animations")]
    pub fn context(self, policy: crate::motion::Policy) -> crate::components::UiContext {
        let typography = crate::typography::Typography::new(
            self.tokens.assets.body.unwrap_or(Font::DEFAULT),
            self.tokens.assets.monospace.unwrap_or(Font::MONOSPACE),
        )
        .with_scale(self.design.text);
        crate::components::UiContext::new(
            crate::components::Appearance::fixed(typography.body, self.tokens),
            policy,
        )
        .with_typography(typography)
        .with_design(self.design)
    }
}

/// A preset is only data. Components use the same surface pipeline for every look.
pub use crate::themes::{material, metro, neumorphic};

// Retain exact token/asset identity while an Iced custom theme is alive. Weak keys
// prevent theme switches from retaining old themes or mixing concurrent windows.
pub(crate) fn remember(theme: &iced_core::Theme, tokens: Tokens) {
    if let iced_core::Theme::Custom(custom) = theme {
        let mut entries = registry().lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|(key, _)| key.strong_count() > 0);
        entries.push((std::sync::Arc::downgrade(custom), tokens));
    }
}
pub(crate) fn recover(theme: &iced_core::Theme) -> Option<Tokens> {
    let iced_core::Theme::Custom(custom) = theme else {
        return None;
    };
    registry()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(key, _)| key.as_ptr() == std::sync::Arc::as_ptr(custom))
        .map(|(_, tokens)| *tokens)
}
type Entry = (std::sync::Weak<iced_core::theme::Custom>, Tokens);
fn registry() -> &'static std::sync::Mutex<Vec<Entry>> {
    static THEMES: std::sync::OnceLock<std::sync::Mutex<Vec<Entry>>> = std::sync::OnceLock::new();
    THEMES.get_or_init(Default::default)
}

pub fn input_style(
    t: Tokens,
    mut style: iced_widget::text_input::Style,
    status: iced_widget::text_input::Status,
) -> iced_widget::text_input::Style {
    use iced_widget::text_input::Status::*;
    let surface = match status {
        Focused { .. } => t.visual.input.focused,
        Hovered => t.visual.input.hovered,
        Disabled => t.visual.input.disabled,
        Active => None,
    }
    .unwrap_or_default()
    .inherit(t.visual.input.normal);
    let mut background = Some(style.background);
    surface.apply(&mut background, &mut style.border, &mut Shadow::default());
    style.background = background.unwrap();
    if let Some(color) = surface.text_color {
        style.value = color.color();
    }
    style
}
pub fn menu_style(
    t: Tokens,
    mut style: iced_widget::overlay::menu::Style,
) -> iced_widget::overlay::menu::Style {
    let mut background = Some(style.background);
    t.visual
        .menu
        .apply(&mut background, &mut style.border, &mut style.shadow);
    style.background = background.unwrap();
    style
}
pub fn dropdown_style(
    t: Tokens,
    mut style: iced_widget::pick_list::Style,
) -> iced_widget::pick_list::Style {
    let mut background = Some(style.background);
    t.visual.dropdown.unwrap_or(t.visual.input).normal.apply(
        &mut background,
        &mut style.border,
        &mut Shadow::default(),
    );
    style.background = background.unwrap();
    style
}
pub fn slider_style(
    t: Tokens,
    mut style: iced_widget::slider::Style,
) -> iced_widget::slider::Style {
    let rail = t.visual.slider_rail;
    if let Some(fill) = rail.fill {
        style.rail.backgrounds.1 = fill.background(rail.opacity());
    }
    if let Some(shape) = rail.shape {
        style.rail.border.radius = shape.radius();
    }
    if let Some(width) = rail.border_width {
        style.rail.width = width.max(0.);
    }
    if let Some(fill) = t.visual.progress.fill
        && rail.fill.is_some()
    {
        style.rail.backgrounds.0 = fill.background(1.);
    }
    let surface = t.visual.slider;
    if let Some(fill) = surface.fill {
        style.handle.background = fill.background(surface.opacity());
    }
    if let Some(color) = surface.border_color {
        style.handle.border_color = color.color();
    }
    if let Some(width) = surface.border_width {
        style.handle.border_width = width.max(0.);
    }
    if let Some(shape) = surface.shape {
        let width = match style.handle.shape {
            iced_widget::slider::HandleShape::Circle { radius } => (radius * 2.) as u16,
            iced_widget::slider::HandleShape::Rectangle { width, .. } => width,
        };
        style.handle.shape = iced_widget::slider::HandleShape::Rectangle {
            width,
            border_radius: shape.radius(),
        };
    }
    style
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self::new(crate::palette::palette(false, false))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsetBevel {
    pub shade: Rgba,
    pub light: Rgba,
    pub width: f32,
}
