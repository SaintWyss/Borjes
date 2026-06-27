# 06-01 SUMMARY — Spike Ventana Desktop X11 (Wave 0)

**Status:** COMPLETE (spike throwaway). **Verdict: DESKTOP type UNWORKABLE on this KDE install → fallback / rethink needed.**

## Qué se construyó
- `src/desktop.rs`: `apply_desktop_window_type` (Update system) tagea la ventana vía x11rb escape hatch (Bevy 0.19 no expone hook). Sin `.unwrap()/.expect()` en replies X (T-06-01 OK).
- `src/main.rs`: borderless fullscreen, `focused:false`, `WinitSettings` Continuous.
- Cargo.toml: x11rb 0.13 + raw-window-handle 0.6 deps directas (sin bump de lockfile).
- Build release + clippy + fmt limpios. Commits: `1d063d8` (zoom fix), `06ed571` (spike).

## Resultado del script 8 pasos (live KDE, RTX 3070, X11)
| Paso | Resultado |
|------|-----------|
| 1 lanzar | App corre, NO crashea (log limpio) |
| 2 xprop | **PASS** — `_NET_WM_WINDOW_TYPE = _NET_WM_WINDOW_TYPE_DESKTOP`. **Q1 resuelta: KWin honra el cambio post-map** (no hace falta unmap/remap). `_NET_WM_STATE` quedó vacío (KWin ignora below/skip en ventanas DESKTOP). |
| 3 Dolphin | n/a — ventana ya invisible |
| 4 Meta+D | **FAIL** — no revela el grafo |
| 5 click escritorio | **UNTESTABLE** — ventana nunca visible (Q2 sin responder) |
| 6 foco egui | **UNTESTABLE** — ventana nunca visible (Q3 sin responder) |
| 7 wallpaper | n/a |
| 8 AnyDesk | n/a |

## Causa raíz (la clave)
`_NET_CLIENT_LIST_STACKING` confirma: la ventana datatree (`0x4600002`) se apila **debajo del contenedor de escritorio de Plasma** (`0x180000d`), que dibuja el wallpaper **opaco encima** → invisible. plasmashell **también** es tipo DESKTOP y gana el apilamiento. Es exactamente el modo de falla conky (KDE bug 348987) que el research marcó como riesgo.

**Bonus (nuevo riesgo):** con la app corriendo (borderless fullscreen + continuous render) la sesión se degradó — Shift+Tab andaba peor. La ventana fullscreen interfiere con input/WM aunque tenga `focused:false`.

## Branch para Wave 1
**"fallback needed"** — DESKTOP puro no sirve en este KDE: plasmashell tapa la ventana y Meta+D no la revela. Pasos 5 y 6 quedaron sin testear porque la ventana nunca fue visible.

Decisión pendiente del usuario (contradice D1 "NO always-below"):
1. Fallback `normal + below + skip_taskbar/pager` (probado por conky; queda encima del wallpaper, debajo de apps).
2. Suprimir wallpaper de Plasma para que el DESKTOP asome (frágil).
3. Repensar: toggle on-demand (overlay fullscreen a pedido) en vez de vivir como escritorio.

También resolver la interferencia de input/Shift+Tab antes de avanzar.
