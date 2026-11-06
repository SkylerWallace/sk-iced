# Renderer compatibility

sk-iced 0.1 targets Iced 0.14 and enables its stock tiny-skia software renderer.
The published UI crate contains no renderer fork or application-specific paths.
An optional repository companion under `renderer/` supplies the common patched
backend and generic regression support used by PC Tonic and File Beholder.
Consumers do not need a [patch.crates-io] section. Use compatible Iced versions
and enable SVG/image features explicitly when needed.

Validate from this repository, not from a downstream application's workspace:

```sh
cargo test --all-features --all-targets --locked
cargo test --all-features --doc --locked
cargo run --features components --example gallery --locked
cargo run --features components --example catalog --locked -- target/catalog neumorphic
```

The tests cover layout, immediate actions, retained control state, clipping,
policy-controlled motion and theme surfaces. The headless catalog uses the same
stock renderer and writes portable PPM images. The interactive gallery uses a
normal Iced window and exercises all four style presets, light/dark modes,
disabled samples, reduced motion.

Downstream renderer patches can change shadow rasterization, SVG alpha handling,
retained-frame behavior and cache performance. The optional common companion is distributed in this repository, outside the
published UI crate. Passing sk-iced tests does not establish pixel identity or equal resize/
scroll throughput between a patched renderer and stock Iced. Benchmark the actual
application renderer when making performance claims.

Windows standalone builds and tests were checked during release preparation.
CI covers Windows, Linux and macOS builds/tests; native Linux/macOS interaction,
GPU rendering and assistive technology remain separate manual
validation. Do not advertise those as verified based on a headless Windows run.

## Stock tiny-skia shadow redraws

Native Windows testing found retained shadow artifacts in stock tiny-skia 0.14.1.
Its native quad shadow painting can write beyond the partial-damage clip. The
gallery therefore wraps its window root in:

```rust
use sk_iced::{Element, components::RedrawBoundary};
fn stock_window<'a, M: 'a>(view: Element<'a, M>) -> Element<'a, M> {
    RedrawBoundary::new(view).full_frame(true).into()
}
```

This opt-in boundary changes damage tracking with a pixel-neutral marker. It
redraws the full view on already-requested frames and schedules no idle frames.
Full-frame painting costs more than correct partial painting; leave it disabled
with a renderer that handles retained shadows correctly. The downstream app
renderer paths are unchanged. This does not replace the backend's clipping or
shadow rasterization.

The development profile optimizes this package at level 1 so rich example views
fit the native Windows stack. This package-local profile does not override a
consuming application's profiles. A regression test constructs each gallery page
on a two-MiB thread stack.

## Linux builds

For standalone Linux library builds, enable the x11 feature (for example,
`cargo check --lib --features components,x11`). A host Iced application can
instead supply its window backend through feature unification. Policy-only builds
need no window backend. Windows and macOS require no X11 flag. Linux x86_64 and
macOS ARM64 library cross-checks pass; these do not validate native display,
clipboard or input integration.

## Shared application companion

PC Tonic and File Beholder point their root Cargo patch to the same
`renderer/iced_tiny_skia` directory and their dev dependency to
`renderer/regression`. Each host builds its real themed dropdowns and invokes
one shared incremental-versus-full repaint assertion. The renderer's pixel
oracles are also the same source and run through both dependency graphs.

The companion retains the newer File Beholder cache, occlusion, SVG and retained
frame fixes plus PC Tonic clipping oracles. Build/package verification checks the
single `renderer/SHA256SUMS.json` snapshot and rejects drift. Matching companion
sources, licenses and patches are included in both apps' source exports, with
relative manifest paths rewritten for standalone rebuilds. See
`renderer/README.md` for update and validation commands.
