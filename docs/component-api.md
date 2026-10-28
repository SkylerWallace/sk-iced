# Component API contracts

Use `UiContext` for view-local fonts, design defaults, palette and motion policy.
Use direct builders when a control needs an explicit appearance. There is no
global theme, route, selection model or animation timer.

- Applications own values, commands, stable keys, workers, persistence and busy state.
- Standard controls have defaults; explicit sizing and spacing override them.
  Dimensions are logical DIPs; Iced owns device scaling.
- `enabled` gates actions independently of tone, border and selection.
  No action means noninteractive. Selected and disabled can coexist.
- Converting a builder into `Element` selects its shared presentation path.
  Buttons, dropdowns and animated layouts also provide `build()` returning an element.
- Native adapters retain native layout/state and are deliberate exceptions:
  Checkbox/TextInput `build()` and `into_native()` return native widgets;
  other native composition builders can return their native container/row.
  Convert into an element to retain shared checkbox keyboard support.
- `custom` retains the supplied native widget's geometry/style. Replacing its
  content can require rebinding its action. It does not infer private native state.

Width/height are available only where the layout makes sense. Single-axis
controls expose that axis (for example progress `girth`); do not assume every
builder accepts every size. Shared text/icon conventions use a left icon and
centered standard button content. Native custom buttons retain their alignment.

## State, overlays and motion

Keep IDs/keys stable and mount gates, popups and scopes consistently across state
changes. Use keyed layout transitions for moves; do not recreate app models to
animate them. Closing content is paint-only and immediately leaves input/focus.
Popup content must have space for shadows within the parent clipping boundary.
Native dropdown overlays retain their selection/search state.

Pass the current `Policy` to animated components and refresh it on system
preference changes. Failed system queries conservatively disable it. Non-Windows defaults disable motion; an app
may supply an explicit policy. Applications own environment-specific restrictions;
pass `Policy::DISABLED` when motion must stay off, and reapply overrides after
refreshing system preferences. Actions never wait for an animation to finish.

See [keyboard support](keyboard.md), [theming](theming.md),
[renderer compatibility](renderer-compatibility.md), and the gallery examples.
The initial 0.1 API is evolving; behavior fixes and any source-breaking changes
will be recorded in the changelog, with breaking changes reserved for 0.2.
