# Forms, feedback and viewers

`FormField` composes an app-owned control with a label and optional helper/error
text. Defaults are a 115-DIP label, 13.333-DIP type and an 8-DIP horizontal gap.
When no helper/error is supplied it returns the native row directly. Validation
rules, parsed values and submit actions belong in the app.

`FormSection` returns a native column. `KeyValueRow` returns a native two-cell row.
`ChoiceGrid` lays out arbitrary tiles or summary cells in a fixed column count
with a four-DIP default gap. Use the existing shared `Button` with custom content
and selected-state styling for choice tiles; a second button family is unnecessary.
App-specific swatches, layout icons and diagnostic category content stay local.

`Slider` retains Iced's native height, range, step and theme defaults. It supports
sizing, step/shift-step, a reset value, release actions and custom styles. Optional
`enabled` gating retains the child tree while blocking input and overlays.
`Dropdown` and `SearchableDropdown` support the same opt-in gating. Calling
`build()` on these selectors returns an `Element` so gating cannot be bypassed;
the default path still contains the original native widget without an added host.
Enablement controls interaction; custom styles can independently express disabled
appearance. Keep `.enabled(...)` in both branches when toggling it so the host is
stable. Disabling closes native popups and cancels capture without forwarding
cancel-generated actions or requesting extra frames.

`Notice` supports a message or custom content, semantic tone, optional icon,
action, helper, padding and surface. The unframed path is native text when no
additional slots are requested. `EmptyState` reuses this static presentation for
empty, loading or failure messages. The app supplies messages and retry actions;
loading adds no spinner or timer.

`LabeledProgress` uses native progress/text layers with contrast clipped at the
fill boundary. `ShareMeter` handles small rounded percent fills. `FillClip` is a
paint-only full-width layer whose visible fraction changes without reflowing
right-aligned contents. Do not put interactive controls inside `FillClip`.
`ResizeGrip` provides local vertical resize capture and bounded heights; app
adapters supply limits and resize messages.

`ReadOnlyText` borrows native `text_editor::Content`; selection/cursor/clipboard
state remain native and content ownership stays in the app. For software rendering, native text paints glyphs while a transparent editor
provides selection. Use `perform_read_only` in the action handler. It rejects edit
actions, applies selection/navigation actions, and returns scroll lines for the
app's existing external scroll area. No application-owned text is rewritten or
copied into a second persistent model.

`PreviewSurface` and `DialogContent` provide native title/toolbar, body and
footer/action slots. They do not own document rendering, PDF generation, printing,
async jobs, focus IDs or default/cancel command semantics. Existing `Dialog` and
`Popup` own the overlay/motion shell; app command guards retain current behavior.

## Catalog

From sk-iced run:

```text
cargo run --offline --features components --example catalog -- target/catalog
```

This renders `light.ppm` and `dark.ppm` using the headless software renderer,
without a GUI/runtime dependency. It covers normal/primary/danger/disabled buttons,
fields, validation, sliders, selectors, choice grids, feedback, progress, selectable
results and preview slots. It is a static contact sheet, not an interactive app.
Use the component tests for keyboard/actions, disabled-state transitions, state
retention, motion policies and compatibility with native widgets.

The components require no global timer or per-item batch animation.
Stateless presentation is immediate. Existing animated wrappers continue to use
the app-supplied policy, including Windows animation preferences.
