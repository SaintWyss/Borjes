---
phase: 10-modelo-layout-radial
plan: 01
status: complete
---

# Summary — 10-01 NEURA model + radial layout foundation

## What was built

Added `src/nav.rs` with `FolderTree`, an owned wrapper around scanned `FolderNode` values plus a one-pass direct-children index derived from `parent_id`.
The index exposes `children`, `parent`, `node`, `root`, `find_by_path`, `len`, and `is_empty`.
Out-of-range child lookups return an empty slice.

Added deterministic `radial_layout(n) -> Vec<bevy::math::Vec2>` with an implied hub at the origin.
It returns no positions for `n == 0`, places a single child on the ring for `n == 1`, and evenly distributes children at angle `i * TAU / n`.
The ring radius is `RADIAL_BASE + RADIAL_PER_CHILD * n as f32`, with constants marked for F11 visual tuning.

Registered the new module in both crate entry points:

- `src/lib.rs` exports `pub mod nav;`
- `src/main.rs` includes `mod nav;`

## Files Modified

- `src/nav.rs` - New pure navigation model, radial layout helper, constants, and unit tests.
- `src/lib.rs` - Exported the nav module.
- `src/main.rs` - Registered the nav module for the binary.
- `.planning/phases/10-modelo-layout-radial/10-01-SUMMARY.md` - Completion summary.

## Deviations

None.
No Bevy systems, resources, components, rendering, camera code, new crates, or global Fruchterman-Reingold coupling were added.
`layout.rs` was left untouched.

## Verification

- Ran `cargo fmt`.
- Static gates passed for `FolderTree`, `from_nodes`, `children`, `find_by_path`, `radial_layout`, `RADIAL_BASE`, `pub mod nav`, and `mod nav`.
- Confirmed `src/nav.rs` has zero matches for `run_layout`, `fruchterman`, or `Fruchterman`.
- Did not run `cargo build`, `cargo clippy`, or cargo tests per the cross-AI constraint that Claude verifies build/clippy/test status.

## Next Phase Readiness

F11 can call `FolderTree::from_nodes(scan::scan_folder(root))`, resolve the current folder's children with `tree.children(current_id)`, and place visible child nodes with `radial_layout(children.len())`.
The hub remains implied at the origin for the drill-down renderer.
