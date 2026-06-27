# 06-02 SUMMARY — On-demand overlay (Wave 1)

**Status:** COMPLETE. **DESK-01 satisfied via on-demand overlay.**

## Qué se construyó
- `src/desktop.rs`: reescrito. SIGUSR1 handler (libc, async-signal-safe = solo un `AtomicBool` store) + `install_signal_handler` (Startup) + `toggle_overlay_visibility` (Update): SIGUSR1 togglea `Window.visible`, Esc oculta. Sin x11rb, sin unwrap.
- `src/main.rs`: Window `visible:false` (arranca oculto), `WindowLevel::AlwaysOnTop`, borderless fullscreen; `WinitSettings` Reactive (no Continuous). Sistemas registrados.
- `Cargo.toml`: x11rb + raw-window-handle removidas (overlay no las necesita); `libc = "0.2"` directa. Sin bump de lockfile.
- Commit: `5b24b53`. Build/clippy/fmt/grep gates limpios.

## Verificación en vivo (usuario, DISPLAY=:0)
| Check | Resultado |
|-------|-----------|
| Hidden idle CPU (steady-state, /proc 5s) | ✅ **0.6%** — el blocker del spike resuelto |
| Show via SIGUSR1 | ✅ overlay aparece fullscreen, AlwaysOnTop |
| Grafo al mostrarse | ✅ galaxia poblada (el "vacío" inicial era timing del layout O(N²)) |
| Buscador egui toma teclado | ✅ |
| Cámara orbital + zoom | ✅ |
| Esc oculta | ✅ |
| Shift+Tab / sesión con overlay oculto | ✅ normal (exacto, sin lag) |

## Notas
- Los dos blockers del spike (invisible bajo plasmashell; lag por render Continuous) → **ambos eliminados**.
- Costo de arranque: el layout O(N²) de 4607 nodos clava CPU ~unos segundos UNA vez al lanzar (no en idle). En el modelo autostart (Fase 9) ocurre una vez al login.
- Log noise: "Couldn't get swap chain texture: Outdated" cada ~1.6s estando oculto (Reactive despierta periódico) — inofensivo a 0.6% CPU; se puede silenciar después si molesta.
- Usuario marcó: usar **Meta+D como tecla del toggle es raro** → en 06-03 elegir otra (ej. Meta+grave). Meta+D ya es "Mostrar escritorio" en KDE.
- App quedó corriendo (PID vivo) para que 06-03 bindee el atajo a ESTE proceso.
