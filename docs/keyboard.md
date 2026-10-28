# Keyboard and focus

Standard buttons, switches, checkboxes, sliders, dropdowns and segmented groups
participate in Iced focus operations. Text inputs retain native focus, cursor,
selection and editing operations. Mouse input keeps the native control path.

Wrap a view in `FocusScope` to opt into Tab/Shift+Tab traversal. This scope cycles
through enabled controls in widget order, scrolls the nearest native scroll area
to reveal focus, and leaves captured events and Ctrl/Alt/Logo shortcuts alone.
Do not replace an app's workspace shortcuts with a scope unless that is intended.

```rust
use iced_core::Font;
use sk_iced::{animation::Element, components::{Appearance, FocusScope, UiContext}, motion::Policy};
let ui = UiContext::new(
    Appearance::fixed(Font::DEFAULT, sk_iced::palette::palette(true, false)),
    Policy::DISABLED,
);
let view: Element<'_, u8> = FocusScope::new(
    ui.button("Save").id("save").on_press(1)
).autofocus("save").into();
```

| Control | Keyboard interaction |
| --- | --- |
| Button, checkbox, switch | Space / Enter activate once; repeated activation keys are suppressed |
| Segmented group | One focus stop; arrows wrap through available choices; Home / End select endpoints |
| Dropdown | Arrows select adjacent values, Home / End select endpoints; selection is immediate and closes an open pointer menu |
| Slider | Arrows adjust through the native step behavior; Shift uses its configured larger step |
| Text input / searchable dropdown | Native Iced editing and focus behavior |

Dropdown Space/Enter opens the existing native menu. Keyboard selection changes
the controlled value directly; it does not introduce a separate pending selection.
The native pointer menu does not provide an independently highlighted keyboard cursor.

Focus rings appear for keyboard focus, without changing geometry or unfocused
pixels. Themes can override `VisualTheme::focus_ring`. Pointer focus does not show
a ring until keyboard interaction. The default ring combines accent and on-accent
ink so it remains visible on primary buttons. Custom focus materials should use
borders/shadows and leave their fill transparent. Disabling a shared control clears its focus.
There is no focus animation or idle redraw timer.

Use stable `.id(...)` values and keep widget order stable. A standard builder
without an action is not a focus stop. Native adapters such as `Button::custom`
retain their previous behavior: keyboard wrapping is explicit via `.keyboard(true)`
and must be given the actual enabled/action state. Rebind a custom checkbox's
action with `.on_toggle(...)` when enabling its shared keyboard wrapper.

## Popups and dialogs

Applications own popup state and focus routing. Put a separate `FocusScope` in
the popup body, set `.active(open).autofocus("first-control")`, and disable the
background scope with `.active(!open)`. On dismissal, return Iced's
`widget::operation::focus("trigger")` task. Keep the popup and scope mounted in
both states; closing reveal content immediately loses input and focus operations.
See the runnable gallery for this complete pattern, including Escape dismissal.

Use one active scope per interaction surface. A nested active scope captures Tab
before its parent; independent sibling scopes need app-owned activation.
Automatic scrolling targets the nearest native scroll area. Virtualized views
still need app-owned row navigation/reveal requests for rows not mounted yet.

This is keyboard support, not a screen-reader accessibility implementation.
Platform accessibility semantics and automatic Windows high-contrast handling
are not claimed for this release.
