---
phase: 08-pins-persistencia
plan: 02
status: complete
---

# Summary — 08-02 right-click pin menu + toggle

## What was built

- `interaction.rs`: right-click (`PointerButton::Secondary`) branch in
  `on_node_click`, placed after the egui-over guard and before the Primary
  guard, opening the pin menu (sets `PinMenu` state with node path/entity +
  cursor pos). Added `Pinned { original }` component, `PinMenu` resource, and
  `apply_pin_visual` / `clear_pin_visual` helpers (overwrite both
  `MeshMaterial3d` and `OriginalMaterial` to gold, stash original in `Pinned`
  for unpin — so hover-out/search-restore return gold automatically).
- `ui.rs`: `pin_menu` egui system — renders a Pin/Unpin button at the cursor,
  toggles the path in `PinStore`, persists via `pins::save_pins`, applies/clears
  the gold visual, dismisses on click-elsewhere.
- `main.rs`: registered `interaction::PinMenu::default()` resource and added
  `ui::pin_menu` to the `EguiPrimaryContextPass` tuple.

## Verification

- `cargo clippy --release -- -D warnings` clean; `cargo fmt --check` clean.
- 62 tests green across the suite.
- No new crates (std-only, serde gate 0).

## Deviations

Codex (the cross-AI builder) wrote interaction.rs + ui.rs but its session
aborted on an internal model API error (HTTP 400, corrupt token output) before
wiring main.rs or writing this summary. Claude completed the main.rs
registration, fixed two lint failures the abort left unverified
(`clippy::while_let_loop` was already clean here; `Context::style` → deprecated,
switched to `global_style()`), and verified the chain. Functionally matches the
plan.
