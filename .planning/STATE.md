---
gsd_state_version: 1.0
milestone: v2
milestone_name: Desktop mode
status: complete
last_updated: "2026-06-28T22:00:00.000Z"
last_activity: 2026-06-28
progress:
  total_phases: 4
  completed_phases: 4
  total_plans: 7
  completed_plans: 7
  percent: 100
---

# State — datatree-rs

**Milestone:** v2 "Desktop mode" (defining roadmap)
**Stack:** Rust + Bevy 0.19, bevy_panorbit_camera 0.35, bevy_egui 0.40.1, ignore, open 5.3.5

## v1 — COMPLETA ✅ (5 fases)
| # | Phase | Status |
|---|-------|--------|
| 1 | Scanner | ✅ 4607 nodos, 7 tests (b9619f9) |
| 2 | Render + cámara | ✅ Bevy 0.19 (1ec8e44) |
| 3 | Layout galaxia | ✅ FR+cluster-pull (d5e6a2b) |
| 4 | Interacción + buscador | ✅ egui + doble-click→Dolphin (f14c417) |
| 5 | Visual + raíz config | ✅ tamaño/symlinks/reescanear (b2fc62c) |

Post-v1: fix zoom-on-search (target_radius). Pendiente smoke-test interactivo del usuario.

## v2 — Desktop mode (planificando)
| # | Phase | Status |
|---|-------|--------|
| 6 | Ventana Desktop → overlay on-demand | ✅ COMPLETA (spike→overlay toggle Meta+1; idle 0.6%) |
| 7 | Archivos on-demand | ✅ COMPLETA (click→burst cubos grises, native click-count, rescan teardown; 50 tests) |
| 8 | Pins + persistencia | ✅ COMPLETA (right-click→Pin/Unpin dorado, pins.json std-only, sobrevive rescan/restart; pan→botón medio) |
| 9 | Autostart + integración | ✅ COMPLETA (autostart KDE lanza overlay oculto al login; op 018 sync-control) |

## v3 — NEURA (drill-down + 2.5D; glow→v4) — spec docs/superpowers/specs/2026-06-28-neura-v3-design.md
| # | Phase | Status |
|---|-------|--------|
| 10 | Modelo nav + layout radial | ✅ COMPLETA (FolderTree índice + radial_layout determinista; 10 tests; pura lógica) |
| 11 | Drill-down render + cámara orto + FSM | pending (próxima) |
| 12 | Labels egui + breadcrumb + parent-ghost | pending |
| 13 | Buscador-teletransporte + pins salto-al-pin | pending |

## Notes
- Cross-AI: Codex construye vía codex-gsd-exec, Claude verifica (cargo build/test/clippy) + commitea.
- Planner GSD = **opus** (no sonnet).
- Repo en SSD (~/dev/datatree-rs), NO en ~/Data (NTFS).
- Build/run: bevy necesita red (Codex sin red → escribe, Claude buildea). Layout O(N²) en DEBUG es lento → verificar con `--release`. GUI: `DISPLAY=:0 ./target/release/datatree-rs` + `spectacle -b -n -a -o shot.png`.
- Spec v2: docs/superpowers/specs/2026-06-27-desktop-mode-design.md
- Riesgo F6: convivencia ventana Desktop X11 con wallpaper Plasma (research acotada en plan-phase 6).

## Current Position

Phase: — (milestone v2 COMPLETE, 9/9 phases)
Plan: —
Status: Milestone v2 "Desktop mode" complete. Overlay on-demand + file-burst + pins + autostart all shipped on acer.
Last activity: 2026-06-28 — Phase 9 done (autostart al login, op 018 sync-control)
