# Contributing

Keep components independent of application routes, models, filesystem operations
and background workers. Use theme properties for materials and explicit policy
for motion. Preserve native state, immediate commands and stable IDs. Avoid
continuous redraws, per-row expensive decoration and unbounded caches.

Before opening a change:

```sh
cargo fmt --all -- --check
cargo clippy --all-features --all-targets --locked -- -D warnings
cargo test --all-features --all-targets --locked
cargo test --all-features --doc --locked
cargo doc --all-features --no-deps --locked
cargo package --locked
```

For a targeted change, run the affected tests first. Add behavioral regression
coverage for state loss, incorrect input routing, disabled actions or motion
policy failures; avoid tests that only repeat implementation constants. Check
new visual states in the interactive gallery with both light/dark palettes and
all relevant presets. Use the standalone crate to avoid downstream patches.

Public builders need documented defaults, state ownership, enablement and
sizing. Add a compiling example for a new API and update the component list.
Keep attribution and applicable MIT/Apache-2.0 notices. Do not add artwork/fonts
without a redistribution-compatible license and provenance.

CI validates the declared Rust minimum and supported feature combinations.
Native platform checks must be reported separately from
headless tests. See [release checklist](docs/releasing.md).

Keep downstream app fixtures and frozen integration profiles in a separate private
harness. Public tests and CI must work without private repositories or credentials.
See [test organization](docs/testing.md).
