# Shared Iced software renderer companion

PC Tonic and File Beholder consume this same optional companion through their
root Cargo `[patch.crates-io]` entries. The main sk-iced library still uses stock
Iced unless a host explicitly selects this renderer. No app model, route or
hardware code belongs in the companion.

- `iced_tiny_skia/`: complete matching upstream package plus shared modifications.
- `patches/`: exact patch against registry iced_tiny_skia 0.14.1, with original,
  patch and result hashes. `PROVENANCE.json` records the input snapshots.
- `regression/`: headless test support, used with each host's actual dropdowns.
- `SHA256SUMS.json`: one reviewed snapshot for all renderer/test/tool sources.

Both apps must reference this renderer and regression package from the same kit.
Run `python renderer/tools/verify.py --consumer PATH_TO_APP_CARGO_TOML` before
building. PC Tonic's build and File Beholder's build/package scripts perform this
check. Cargo dependency metadata for both apps must resolve iced_tiny_skia to
this exact directory; updating a private copy is not supported.

Run `cargo test -p iced_tiny_skia --lib --locked` and the app's
`open_themed_dropdowns_remain_pixel_identical_after_partial_repaints` test from
each host. The renderer also contains shared pixel oracles for clipped labels,
shadow halos, fills, glyphs, SVG alpha, retained damage, caches and occlusion.
The generic regression support opens real host-built dropdowns and compares
repeated partial paints with a full frame at 100%/150%, light/dark appearance.
The `changing_dropdown_labels_remain_pixel_identical_after_partial_repaints`
consumer test retains the widget tree and framebuffer while changing labels.
It uses actual compositor damage and compares every pixel against a full paint
for all four styles, light/dark appearance, 100%/150%, selection changes and
category option replacement. Cached text damage respects alignment anchors.
`SK_ICED_RENDERER_CAPTURE_DIR` enables optional local menu/label PPM captures.

The active implementation retains File Beholder's newer shadow cache, occlusion,
SVG and retained-damage fixes and PC Tonic's clipping regression oracles. Existing
PC Tonic and File Beholder profiling variables remain aliases for the canonical
`SK_ICED_RENDERER_PRESENT_TRACE`, `SK_ICED_RENDERER_UNCACHED_CLIP_MASK` and
`SK_ICED_RENDERER_REFERENCE_RASTER` controls. Profiling is opt-in and local.

Keep upstream MIT and contributor MIT/Apache-2.0 notices separate (see the renderer
LICENSE, LICENSE-APACHE and NOTICE). This kit is a repository companion, excluded
from the main sk-iced crate's published package. Application source exports bundle
the exact kit and rewrite paths so a sibling checkout is unnecessary to rebuild.

To change the renderer, edit its single active source, run both hosts' affected
checks, regenerate the exact patch/result hashes and SHA256SUMS after review,
and update both consumers together. Never regenerate pins silently during builds.

After reviewing an edit, run `python renderer/tools/update.py --upstream PATH_TO_PRISTINE_ICED_TINY_SKIA` from the repository root, then verify the kit and both consumers. The updater checks the pristine package checksum and never runs during builds.
