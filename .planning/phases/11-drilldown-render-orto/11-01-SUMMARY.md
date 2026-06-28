# 11-01 SUMMARY - Drill-down render ortho

## Implemented

- Added `src/drill.rs` with:
  - `NavState { tree, current, dirty }`
  - `LevelNode` and `LevelChild(usize)`
  - `render_level` despawn/respawn renderer for one current level
  - `on_level_click` primary-click descend with egui pointer guard
  - `ascend` on Backspace only, no-op at root

- Rewired startup in `src/main.rs`:
  - registered `mod drill`
  - replaced `start_layout` with `init_nav`
  - `init_nav` scans `RootConfig`, builds `FolderTree`, sets root as current, and inserts dirty `NavState`

- Replaced the camera path:
  - camera is now `Camera3d + OrthographicProjection`
  - no `PanOrbitCamera` component is attached
  - `PanOrbitCameraPlugin` remains installed so existing UI camera code is harmless/no-op

- Rewired the update/observer path:
  - removed registered `apply_layout`
  - removed registered `interaction::apply_pins`
  - removed registered `render::draw_edges`
  - removed registered `interaction::on_node_click`
  - added `drill::ascend`
  - added `drill::render_level`
  - added `drill::on_level_click`

- Kept old galaxy/burst/pin code in place but dormant:
  - FR layout task functions remain in `main.rs` with dead-code allows
  - old v2 click/pin/edge systems remain in their modules with dead-code allows
  - `handle_rescan` now rebuilds `NavState` directly instead of starting the old galaxy layout

- Updated exports:
  - added `pub mod drill` in `src/lib.rs`
  - removed the broad F10 dead-code allowances from used `nav` symbols

## Verification

- Ran `cargo fmt`.
- Did not run `cargo build` or `cargo clippy`, per instruction; Claude owns build/clippy verification.

## Notes

- Ascend is Backspace-only so Escape remains dedicated to hiding the overlay.
- Per-level entities currently use simple despawn/respawn as required; pooling is deferred.
