---
phase: 07-archivos-on-demand
plan: 03
status: complete
---

# Summary — 07-03 rescan teardown

## What was built

`handle_rescan` (src/main.rs) now despawns every `With<interaction::FileNode>`
entity and removes `interaction::Expanded` markers on reroot, alongside the
existing folder (`NodeRef`) teardown. Files are absolute siblings (not
`ChildOf` children), so the folder despawn never took them — without this no
gray file cube survived a "Reescanear".

## Verification

- `cargo build --release` / `cargo clippy --release -- -D warnings` / `cargo fmt --check` clean.
- Grep gates: `With<interaction::FileNode>`, `file_entities`, `Expanded` present in main.rs.
- 50 tests green across the suite.
- Tactile checkpoint: overlay runs and toggles via Meta+1 (signal-handler race
  fixed in commit 1a79750). Click→burst / collapse / double-click-open flow
  exercised live by the user on the running overlay.

## Deviations

During phase 7 a separate desktop-integration bug surfaced and was fixed:
SIGUSR1 (the Meta+1 overlay toggle) hit the default disposition (terminate)
because `install_signal_handler` ran as a Startup system, after Bevy's ~1-2s
window/GPU init. Moved the install to the first line of `main()`
(commit 1a79750) and added a launch-or-toggle wrapper
(`~/.local/bin/datatree-toggle`) so Meta+1 relaunches the overlay if it was
closed. Both are per-machine desktop integration, not part of the file-burst
feature itself.
