# Release checklist

1. Review the changelog, version, Rust minimum, supported Iced version and public API.
2. Run standalone CI: formatting, strict Clippy, tests/doctests, docs, feature
   matrix, Windows/Linux/macOS checks and the declared minimum Rust version.
3. Run the interactive gallery. Check keyboard-only navigation, popup entry and
   focus return, resizing, all styles, disabled states and reduced-motion changes.
   Report native platform coverage honestly.
4. Inspect `cargo package --list`. Verify source, tests, examples and licenses are
   present and build outputs/private app files are absent. Run
   `cargo publish --dry-run --locked` and test the packaged source as a dependency
   in a fresh application with stock Iced. A dry run does not upload a release.
5. Review dependency provenance and applicable notices. Confirm public repository
   access and documentation links, update the release date and tag the reviewed
   commit as `v0.1.0`.
6. Publish manually when approved. CI does not publish and holds no registry token.

A release build/test is not a claim of screen-reader support, automatic high
contrast, GPU-renderer parity or native validation on an untested platform.
