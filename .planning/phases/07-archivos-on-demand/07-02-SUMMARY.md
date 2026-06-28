---
phase: 07-archivos-on-demand
plan: 02
subsystem: desktop-rendering
tags: [rust, bevy, picking, filesystem, interaction]

requires:
  - phase: 07-archivos-on-demand
    provides: one-level scan_files API, FileAssets, FILE_NODE_SCALE, BURST_RADIUS, fibonacci_offset
provides:
  - native Pointer<Click>.count routing for folder/file click behavior
  - FileNode and Expanded ECS components for on-demand file bursts
  - folder single-click expand/collapse and double-click open without layout movement
  - file double-click open with default app
  - hover highlighting for both folder and file nodes
affects: [07-archivos-on-demand, file-burst, interaction]

tech-stack:
  added: []
  patterns: [Bevy native click count routing, ECS marker toggle state, absolute sibling burst entities]

key-files:
  created:
    - .planning/phases/07-archivos-on-demand/07-02-SUMMARY.md
  modified:
    - src/interaction.rs
    - src/scan.rs
    - src/render.rs

key-decisions:
  - "Used Pointer<Click>.count and a Primary-button guard; removed the manual Instant timer entirely."
  - "Spawned files as absolute-positioned sibling Mesh3d entities with FileNode, not NodeRef."
  - "Used FileAssets handles and render::FILE_NODE_SCALE/BURST_RADIUS/fibonacci_offset directly."

patterns-established:
  - "FileNode { parent, path } stores the folder owner and open target while keeping files out of GraphData-indexed systems."
  - "Expanded marks folder entities with visible file bursts; collapse despawns only FileNode children whose parent matches that folder."
  - "Hover observers use Or<(With<NodeRef>, With<FileNode>)> so files restore through OriginalMaterial like folders."

requirements-completed: [FILE-01, FILE-03, FILE-04, INT-01]

duration: 25min
completed: 2026-06-28T13:22:50-03:00
---

# Phase 7: Archivos On-Demand Plan 02 Summary

**Native Bevy click routing now toggles folder file bursts and opens folders/files without moving the galaxy**

## Accomplishments

- Added `FileNode { parent, path }` and `Expanded` in `src/interaction.rs`.
- Rewrote `on_node_click` to use `Pointer<Click>.count` plus `PointerButton::Primary`; removed `Local<Instant>` and `std::time` click timing.
- Folder single-click now toggles a radial file burst using `scan::scan_files`, cloned `FileAssets`, `render::fibonacci_offset`, `render::BURST_RADIUS`, and `render::FILE_NODE_SCALE`.
- Folder double-click opens the folder with `open::that` and ensures the burst remains shown; file double-click opens the file path with `open::that`.
- File entities carry no `NodeRef`, do carry `OriginalMaterial`, and hover observers now highlight folders and files through `Or<(With<NodeRef>, With<FileNode>)>`.
- Removed the temporary Plan 01 `#[allow(dead_code)]` lines from the now-used file scan and file render symbols.

## Files Modified

- `src/interaction.rs` - Added file burst components/helpers, native click routing, file open handling, and widened hover filters.
- `src/scan.rs` - Removed temporary dead-code allowances from `FileEntry` and `scan_files`.
- `src/render.rs` - Removed temporary dead-code allowances from `FILE_NODE_SCALE`, `BURST_RADIUS`, `FileAssets`, and `fibonacci_offset`.
- `.planning/phases/07-archivos-on-demand/07-02-SUMMARY.md` - Completion summary.

## Deviations from Plan

None - plan executed as written. No new crates were added, `run_layout` is not referenced from `src/interaction.rs`, and no git commit was created.

## Verification

- Ran `cargo fmt`.
- Did not run `cargo build`, `cargo clippy`, or cargo tests per the cross-AI constraint that Claude verifies build/clippy/test status.
- Grep checks confirmed `FileNode`, `Expanded`, `PointerButton::Primary`, `trigger.count`, both `open::that` branches, `scan_files`, `fibonacci_offset`, `BURST_RADIUS`, `FILE_NODE_SCALE`, widened hover filters, no `std::time`, no `Local<Option<(Instant`, and no `run_layout` in `src/interaction.rs`.

## Next Phase Readiness

Plan 03 can tactilely verify the runtime behavior: single-click folder burst/collapse, folder double-click open while leaving the burst visible, file double-click default-app open, right-click no-op, file hover highlight, and no galaxy movement on expand/collapse.
