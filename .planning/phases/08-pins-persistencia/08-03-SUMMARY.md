---
phase: 08-pins-persistencia
plan: 03
status: complete
---

# Summary — 08-03 apply_pins re-mark + checkpoint

## What was built

`interaction::apply_pins` runs after `apply_layout` in the Update chain over
`Without<Pinned>` NodeRef nodes; any whose path is in the disk-loaded PinStore
gets re-golded via the shared `apply_pin_visual`. One cheap system covers both
startup load (PIN-02) and Reescanear re-mark (PIN-03 always-visible).

## Verification

- `cargo build --release` / `cargo clippy --release -- -D warnings` / `cargo fmt --check` clean.
- Grep gates: `apply_pins`, `Without<Pinned>`, `interaction::apply_pins` registered after apply_layout.
- Tactile checkpoint PASSED live: right-click opens the Pin menu, Pin golds the
  node and writes ~/.config/datatree-rs/pins.json (confirmed
  `["/mnt/hdd/Data/01_Now/AdSI"]`).

## Deviations (bugs found at the checkpoint, fixed in commit 71ae066)

1. PanOrbitCamera default `button_pan = MouseButton::Right` swallowed the
   secondary click, so the menu never opened. Remapped pan to Middle.
2. The opening right-click dismissed the egui Area on its first frame
   (`clicked_elsewhere`). Added a `just_opened` guard.
