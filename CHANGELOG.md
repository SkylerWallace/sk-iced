# Changelog

## 0.1.0 — Unreleased

- Shared controls, responsive settings cards, forms, navigation, panels/tabs,
  menus/dialog shells, indicators, scrolling, virtual lists, tables and trees.
- Theme-owned palettes, typography, icons, materials, shapes and layered shadows;
  Default, Neumorphic, Material and Metro presets with light/dark support.
- Immediate, interruptible ripple, spin, reveal, keyed-layout and highlight motion,
  respecting Windows animation preferences.
- Keyboard focus IDs, optional Tab scopes, focus rings, control activation and
  selection navigation, with native input state retained.
- Standalone stock-Iced gallery, headless catalog, tests, CI and package checks.

0.1 is the initial public API. See the component contracts and keyboard guide
for native-adapter exceptions and platform/accessibility limitations.

- Native gallery startup and popup focus validated on Windows; development-stack
  regression coverage and an opt-in stock-renderer full-frame redraw boundary.
- Explicit X11 backend selection for standalone Linux builds, with Linux/macOS
  library cross-checks and Linux Rust 1.88 verification.
- Responsibility-based public test names, with downstream app snapshots moved to
  an unpublished private harness; CI is temporarily manual-only/disabled.
