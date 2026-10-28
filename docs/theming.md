# Theme-driven looks

`ThemeDefinition` combines semantic color tokens, visual properties, typography,
design metrics and an optional icon pack. Components never select a preset by
name. `themes::neumorphic` is an ordinary theme definition built from this API.

`ThemeDefinition::default()` preserves the default light appearance. Existing
`Appearance` constructors also keep their current look when visual properties
are omitted. Start with `palette::palette(dark, high_contrast)` for other modes.

```rust
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
```

Retain the definition and motion policy in application state. Use its context to
build views and its native Iced theme for rendering. A registered Iced theme
retains the exact tokens and asset resolver; concurrent windows do not share an
active global theme. Unrelated Iced themes use the default visual properties.

## Visual properties

| Property | Supported values |
| --- | --- |
| Fill | Solid color or linear gradient, including alpha and angle in degrees |
| Shape | Rectangle, independently rounded corners, pill |
| Border | Color, alpha and width |
| Foreground | Optional per-surface text color |
| Shadows | Up to four layers, each with color, alpha, angle, distance and blur radius |
| Opacity | Independent surface opacity; content retains its own contrast |
| Typography | Body/monospace fonts and text scale |
| Metrics | Control height, padding, icon gap and other `DesignTokens` |
| Icons | Semantic-name resolver with SVG, RGBA or font-glyph assets |

Roles cover background, flat cards, expanding cards, resize grips, segmented choices, buttons, important/danger actions, flat controls, inputs,
checkboxes, panels, headers, tabs, badges, progress, menus, sliders and scrollbars.
Selected panels/tabs have separate optional overrides. Interaction states include
normal, hovered, pressed, disabled and focused where applicable.

An omitted property inherits from the normal state, then the component's default.
`opacity: Some(0.)` is transparent; `shadows: Some([None; 4])` explicitly removes
shadows. Shapes and spacing do not change the component's message or model.

`Surface` decorates custom content using the same properties. Standard buttons,
panels and form controls integrate decoration without delaying actions. Native
`build()` adapters deliberately return the underlying Iced widget; use conversion
to `Element` for layered decoration, or wrap specialized content in `Surface`.
Explicit application styling can override the theme and remains an escape hatch.

Surfaces support solid and linear fills, opacity, rounded corners, shadows and
inset edges. Motion uses the supplied policy.

## Fonts and icon packs

Register font bytes with Iced before constructing views. The library neither
installs fonts nor loads font files. Unspecified fonts inherit the supplied
appearance; a definition's context uses Iced's default font when none is supplied.

An icon resolver returns `Option<IconAsset>` for a semantic name. Missing or
unsupported assets use the icon supplied by the application:

```text
ThemeDefinition::new(tokens).icons(my_icon_pack)
Icon::from_asset(ui.appearance, fallback_asset)?.semantic("save").build()
```

Semantic icons cache native rendering and layout in their existing widget tree.
Fallback, color, size, font and enabled-state changes invalidate the cache.
Artwork and its licenses stay with the application or theme package. The `icons`
feature enables the asset widgets; SVG assets additionally require `svg`.

Numeric visual properties support the optional `serde` feature. Runtime font
handles and icon resolver functions are not serialized; register them in code.

## Preset and example

```text
themes::neumorphic(palette::palette(false, false)).context(policy)
cargo run --features icons --example theming
cargo run --features components --example catalog -- target/neumorphic neumorphic
```

The neumorphic preset defines light/dark surfaces, directional highlight/shadow
layers, recessed inputs and pressed controls, flat cards/rows, elevated panels,
soft progress fills and selection accents. Idle themes schedule no animation frames.

`InsetBevel` defines recessed edge colors and width. Custom native buttons preserve
their own styling without automatic elevation; opt in with `surface_role(...)`.
Surface roles define visual elevation. Iced’s layout and overlay stack still own
paint order and hit testing; a visual depth setting never reorders input targets.

`ThemeDefinition::tint_surfaces(accent, amount)` adds a subtle accent wash to surface
fills, borders, shadows and inset edges while preserving text and semantic colors.
The default palette remains available when no tint is applied. Metro uses square
accent tiles for standard actions, with matching readable icon/text ink and
neutral disabled states; flat navigation and data rows keep their own surfaces.

Shared `Button::new` content is centered on both axes by default. Explicit
`center_x(...)` sizing still takes precedence, and `Button::custom` retains the
native button layout. Disabled vector/glyph icons use the theme’s disabled button
ink when defined, keeping icon and text actions visually consistent.

Persistent button selection is separate from tone: use `Button::selected(true)`
and define `VisualTheme::toggled_button` for the on-state material. Native button
adapters can use `Tokens::toggle_button(selected)` and `SurfaceRole::Toggle(selected)`
together to match their fill, text and decoration. Without an on-state material,
selected buttons retain the standard primary appearance. Neumorphic toggles and
selected segmented options share a recessed material, including while hovered.

`VisualTheme::dropdown` optionally separates dropdown materials from text inputs.
Both fixed-list and searchable dropdown builders use it for their raised shell;
text inputs retain their own recessed material. Selected Neumorphic segments
combine a restrained accent fill with inset edges. `ResizeGrip::indicator(false)`
keeps only the original hover/drag divider feedback when a permanent grip marker
is inappropriate.

Standard icon-only buttons default to 32×32 DIP with centered artwork. Text
buttons use the same default height and expand to fit their labels. Explicit
width, height and padding take precedence; native-button adapters retain their
original geometry.

`VisualTheme::segment_group` styles the group separately from its choices. The
Material preset uses a rounded outline and softly tinted selection; Metro uses
rectangular outlines and accent-filled selection. Selection state and motion
policy stay with the existing segmented-control implementation.

`SettingsCard` separates the label/description from its action and stacks them in
narrow layouts without replacing either widget tree. Set `VisualTheme::settings_card`
for the card material. `ToggleSwitch` uses `switch_off`, `switch_on` and
`switch_thumb` for independently themed track and thumb surfaces. Its controlled
value updates immediately; only the thumb position animates for 100 ms, using
the supplied motion policy. Existing popup reveals handle settings open/close.
