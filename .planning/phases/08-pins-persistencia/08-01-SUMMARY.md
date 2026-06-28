---
phase: 08-pins-persistencia
plan: 01
status: complete
---

# Summary — 08-01 pin persistence foundation

## What was built

Added `src/pins.rs` with a path-keyed `PinStore`, `load_pins`, `save_pins`, and `pins_path`.
Pins load from `~/.config/datatree-rs/pins.json` or `$XDG_CONFIG_HOME/datatree-rs/pins.json` when set.
Missing or malformed files resolve to an empty set without panicking.

Persistence is std-only and writes atomically: parent directories are created, paths are sorted for deterministic output, JSON is written to a sibling temp file, then published with `std::fs::rename`.
The JSON helper handles `\` before `"` and tests cover round-tripping paths containing both characters.

Added `PinAssets` and `setup_pin_assets` in `src/render.rs` with one shared unlit gold material using `Color::srgb(1.0, 0.84, 0.0)`.
Registered `mod pins`, inserted `pins::PinStore(pins::load_pins())`, registered `render::setup_pin_assets` at `Startup`, and exported `pub mod pins` from `src/lib.rs`.

## Files Modified

- `src/pins.rs` - New pin store, config path resolution, atomic load/save helpers, minimal JSON scanner, and unit tests.
- `src/render.rs` - Added shared gold `PinAssets` resource and startup setup system.
- `src/main.rs` - Loaded pins at app startup and registered pin assets.
- `src/lib.rs` - Exported the pins module for crate tests/users.
- `.planning/phases/08-pins-persistencia/08-01-SUMMARY.md` - Completion summary.

## Deviations

None. No Plan 02/03 systems were added; `apply_pins` and `pin_menu` remain absent from `src/main.rs`.

## Verification

- Ran `cargo fmt`.
- Static gates passed for `PinStore`, `load_pins`, `save_pins`, `pins_path`, `to_json_array`, `parse_json_array`, `create_dir_all`, `rename`, `PinAssets`, `setup_pin_assets`, gold color, startup registration, and absence of `apply_pins`/`pin_menu`.
- Confirmed zero matches for the disallowed JSON dependency token in `Cargo.toml` and `src/pins.rs`.
- Did not run `cargo build`, `cargo clippy`, or cargo tests per the cross-AI constraint that Claude verifies build/clippy/test status.

## Next Phase Readiness

Plan 02 can mutate `PinStore` and call `pins::save_pins(&store.0)` after Pin/Unpin.
Plan 03 can clone `PinAssets.material` to re-apply the gold pinned mark after startup load or rescan.
