---
phase: 07-archivos-on-demand
plan: 01
subsystem: desktop-rendering
tags: [rust, bevy, filesystem, scan, mesh-assets]

requires: []
provides:
  - one-level scan_files(dir) API with FileEntry path/name records
  - cached FileAssets resource for shared gray cuboid file meshes
  - fibonacci_offset geometry helper and BURST_RADIUS constant for radial bursts
  - Startup registration for render::setup_file_assets
affects: [07-archivos-on-demand, file-burst, interaction]

tech-stack:
  added: []
  patterns: [std::fs::read_dir one-level scan, shared Bevy mesh/material handles, fibonacci sphere offsets]

key-files:
  created:
    - .planning/phases/07-archivos-on-demand/07-01-SUMMARY.md
  modified:
    - src/scan.rs
    - src/render.rs
    - src/main.rs

key-decisions:
  - "Used std::fs::read_dir for one-level file listing; did not reuse WalkBuilder."
  - "Kept file visuals cached as one shared gray Cuboid mesh and one shared unlit gray material."
  - "Kept burst placement as standalone geometry; did not touch run_layout or spawn file entities."

patterns-established:
  - "scan_files returns only immediate eligible files, skips dotfiles and EXCLUDED names, and caps bursts with .take(200)."
  - "File meshes/materials are created once at Startup and reused through FileAssets."
  - "fibonacci_offset mirrors the existing golden-angle sphere distribution for later on-demand file bursts."

requirements-completed: [FILE-01, FILE-02, FILE-04]

duration: 15min
completed: 2026-06-28T13:14:51-03:00
---

# Phase 7: Archivos On-Demand Plan 01 Summary

**One-level file scanning plus cached gray cuboid file assets and reusable radial burst geometry for later click wiring**

## Accomplishments

- Added `FileEntry { path, name }` and `scan_files(dir)` in `src/scan.rs`.
- `scan_files` uses `std::fs::read_dir`, returns immediate files only, skips dotfiles and `EXCLUDED`, returns `Vec::new()` on read errors, and applies the locked `.take(200)` soft cap.
- Added tempfile-free unit tests for files-only behavior, dotfile/EXCLUDED filtering, the 200-entry cap, and empty results for missing directories.
- Added `FileAssets`, `setup_file_assets`, `FILE_NODE_SCALE`, `BURST_RADIUS`, and `fibonacci_offset` in `src/render.rs`.
- Registered `render::setup_file_assets` in the existing `Startup` system tuple in `src/main.rs`.

## Files Modified

- `src/scan.rs` - Added `FileEntry`, `scan_files`, and focused scan_files unit tests.
- `src/render.rs` - Added cached gray cuboid file assets and burst helper constants/math.
- `src/main.rs` - Registered `render::setup_file_assets` at startup.
- `.planning/phases/07-archivos-on-demand/07-01-SUMMARY.md` - Completion summary.

## Deviations from Plan

None - plan executed as written.

## Verification

- Ran `cargo fmt`.
- Did not run `cargo build`, `cargo clippy`, or cargo tests per the cross-AI constraint that Claude verifies build/clippy/test status.
- Grep checks confirmed `FileEntry`, `scan_files`, `read_dir`, `.take(200)`, `FileAssets`, `setup_file_assets`, `Cuboid::new`, gray color, `BURST_RADIUS`, `fibonacci_offset`, Startup registration, and no `run_layout` reference in `src/render.rs`.

## Next Phase Readiness

Plan 02 can call `scan::scan_files(dir)`, clone `FileAssets.mesh` and `FileAssets.material`, and position file nodes with `parent_pos + fibonacci_offset(i, n) * BURST_RADIUS`, with the documented `n == 1` special case using `Vec3::Y * BURST_RADIUS`.
