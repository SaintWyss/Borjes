# 06-03 SUMMARY — Atajo KDE + coexistencia (Wave 2)

**Status:** COMPLETE. **DESK-02 satisfecho vía overlay on-demand + atajo global.**

## Qué se hizo
- **Atajo global KDE = Meta+1** (usuario eligió; descartó Meta+D por raro). Meta+1 estaba ocupado por "activar entrada 1 del gestor de tareas" → liberado con OK del usuario.
- `~/.local/share/applications/datatree-toggle.desktop` (Exec=`pkill -USR1 -x datatree-rs`).
- Binding en `~/.config/kglobalshortcutsrc` `[services][datatree-toggle.desktop]` `_launch=Meta+1`.
- Reload `systemctl --user restart plasma-kglobalaccel.service` + `kbuildsycoca6`.
- **Verificado por usuario: Meta+1 togglea el overlay** ("genial"). ✅
- Registrado en sync-control `system-ops.tsv` op **016** (per-máquina equipo de escritorio).
- `docs/desktop-setup.md` (Codex): qué es, por qué no es ventana-Desktop, toggle/Esc, binding KDE, costo, troubleshooting.

## Coexistencia (DESK-02)
- El overlay va AlwaysOnTop al mostrarse y desaparece al ocultarse → **no pelea con el wallpaper de Plasma** (no hizo falta el hack "Color plano"). El problema original de DESK-02 se disolvió con el modelo on-demand.
- AnyDesk: usuario marcó "no me interesa / no probé" → no verificado, no bloqueante.

## Notas
- El atajo requiere el proceso corriendo. Autostart al login = **Fase 9** (DESK-03).
- Plasma 6.7.1; formato real de `_launch` = solo `Meta+1` (sin triple-valor).
