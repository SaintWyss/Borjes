---
gsd_state_version: 1.0
milestone: v2
milestone_name: Desktop mode
status: planning
last_updated: "2026-06-27T17:28:11.376Z"
last_activity: 2026-06-27
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
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
| 7 | Archivos on-demand | pending (próxima) |
| 8 | Pins + persistencia | pending |
| 9 | Autostart + integración | pending |

## Notes
- Cross-AI: Codex construye vía codex-gsd-exec, Claude verifica (cargo build/test/clippy) + commitea.
- Planner GSD = **opus** (no sonnet).
- Repo en SSD (~/dev/datatree-rs), NO en ~/Data (NTFS).
- Build/run: bevy necesita red (Codex sin red → escribe, Claude buildea). Layout O(N²) en DEBUG es lento → verificar con `--release`. GUI: `DISPLAY=:0 ./target/release/datatree-rs` + `spectacle -b -n -a -o shot.png`.
- Spec v2: docs/superpowers/specs/2026-06-27-desktop-mode-design.md
- Riesgo F6: convivencia ventana Desktop X11 con wallpaper Plasma (research acotada en plan-phase 6).

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-06-27 — Milestone v2 started
