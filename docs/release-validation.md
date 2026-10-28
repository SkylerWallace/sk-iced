# Release validation

The following checks were run locally on Windows using the standalone crate,
without downstream renderer patches:

- Library/integration tests for immediate actions, retained state, keyboard
  focus/activation, reduced motion.
- Gallery pages rendered in Default, Neumorphic, Material and Metro,
  in both light and dark modes.
- Rust 1.88 GNU all-feature library check, including the Linux target.
- Cross-target library checks for x86_64 Linux and ARM64 macOS.
- Feature checks for policy-only,
  serde-only, theme, animations, components, icons and SVG.
- Formatting, strict Clippy, rustdoc and compiling documentation examples.
- Package inspection and publish dry run, plus a fresh consumer of the extracted
  package using registry iced_renderer 0.14.0 and iced_tiny_skia 0.14.1.
- Both downstream applications passed compile checks against the shared source.
- Downstream frozen-theme and UI-profile compatibility checks now live in a
  separate private harness; public tests and CI have no private dependency.

## Native Windows gallery

Native checks verified:

- Startup using the documented development build.
- Tab and Shift+Tab navigation, dropdown arrow selection and Space activation.
- Popup autofocus, focus containment, Escape dismissal, Close activation,
  return to the trigger, and reopening through the restored keyboard focus.
- Disabled samples reject pointer actions and are skipped during Tab traversal.
- Reduced-motion changes take effect immediately.
- Switching among all four styles retains gallery state; window resizing keeps
  the views usable.

Issues found during validation were fixed:

- A development-build stack overflow: this package's development profile now
  uses optimization level 1; gallery construction is tested on a two-MiB stack.
- Standalone Linux backend selection: the explicit x11 feature enables a window
  backend without changing either downstream application's feature selection.
- Stock-renderer retained shadow artifacts: the gallery uses the opt-in
  RedrawBoundary full-frame mode. Pixel neutrality and absence of idle frame
  requests have regression coverage. Native popup dismissal was rechecked
  without the earlier shadow artifacts.

See [renderer compatibility](renderer-compatibility.md) for the full-frame
tradeoff. Neither app's renderer, layout, motion policy or build profile was changed.

## External checks still required

Hosted Windows/Linux/macOS CI is intentionally disabled to avoid account usage.
No hosted result is claimed; re-enable it explicitly when usage is available.
Native Linux/macOS interaction validation requires its target environments.

The package has not been published. No GPU parity or screen-reader support is
claimed. Follow the [release checklist](releasing.md) before tagging/publishing.

## Test privacy

Public coverage is independent of the unpublished downstream compatibility harness.
Stage-based test filenames have been replaced with responsibility-based names.
Frozen app themes and historical UI profiles are excluded from the public crate
and source export; their private harness retains those regression checks.
