# Borjes v3 "NEURA" — design spec

**Estado:** decidido (Junta CEO cross-AI Codex+Gemini, 2026-06-28). No empezado.
**Base de diseño:** mock `~/Descargas/Explorador de archivos 3D neuronal.zip` (canvas 2D — sirve como look+UX, no como motor).
**Predecesor:** milestone v2 "Desktop mode" COMPLETO 9/9 (overlay Meta+1, file-burst, pins, autostart).

## Goal

Reemplazar la navegación "orbit de la galaxia completa" (4607 nodos a la vez) por un **drill-down de a un nivel**: ves la carpeta actual como hub central + sus hijos en hub-and-spoke; entrás a un hijo, subís al padre. Estética 2.5D plana (la capa neural/glow se difiere a v4). Conserva el core value: llegar a una carpeta honda (AdSI) en segundos.

## Decisiones LOCKED (no se rediscuten)

| Eje | Decisión | Por qué (junta) |
|-----|----------|-----------------|
| Navegación | drill-down REEMPLAZA orbit. 1 nivel por vez. **click**=entrar, hover=preview | hover-dive del mock = navegación accidental |
| Dimensión | **2.5D ortográfica** (cámara orto), no 3D perspectiva | simple, sin profundidad que pelee con glow |
| Glow | **diferido a v4** (SDF shader, no bloom/HDR global) | bloom global rompe idle 0.6% en GTX1650/Intel |
| Buscador | **teletransporta**: salta niveles + abre el path hasta el nodo | sin esto, drill-down mata el core value |
| Labels | egui + `camera.world_to_viewport`, texto a píxel nativo | billboard = font-atlas thrashing → stutter en zoom |
| Entidades | **pool reciclado** por nivel | spawn/despawn = picos; toggle-visibilidad = 4607 zombis + invalida transform cache |
| Cámara | FSM que **absorbe interrupción** (doble-click/atrás rápido re-targetea, no bloquea input) | supuesto más frágil señalado por Gemini |
| Contexto | breadcrumb + parent-ghost + preview de nietos en hover | drill-down pierde "dónde estoy" |
| Pins | botón **salto-al-pin** (teletransporta al path pinneado) | dorado inútil si estás en otro nivel |

## Stack — Bevy FIJO

Sigue Rust + Bevy 0.19. NO se cambia (web rechazado). Cámara orto en vez de PanOrbitCamera.

### Se REUSA de v1/v2
- `scan.rs` — `scan_folder` (árbol completo, se carga una vez) + `scan_files` (archivos por carpeta).
- `desktop.rs` — overlay on-demand, Meta+1, SIGUSR1, idle reactive. Sin cambios.
- `pins.rs` — PinStore + pins.json. Se agrega salto-al-pin.
- autostart + `~/.local/bin/borjes-toggle`. Sin cambios.

### MUERE
- `layout.rs` Fruchterman-Reingold async global. v3 no posiciona 4607 nodos juntos.
- PanOrbitCamera (orbit libre). Reemplazado por cámara orto + pan/zoom acotado.
- `render::spawn_graph_nodes` del grafo completo. Reemplazado por render por-nivel.

### NUEVO
- **Layout por-carpeta**: radial determinista (hub centro + N hijos en círculo, radio por count). Barato, sin física, cacheable por path.
- **Navegación FSM**: `Nav { current: PathBuf, stack: Vec<PathBuf> }` + `Transition { from, to, t }`. El árbol completo de `scan_folder` queda en memoria (índice path→hijos); cada nivel se deriva de ahí, NO se re-escanea.
- **Render por-nivel**: pool de entidades (hub + hijos visibles). Al entrar/salir, re-bindea el pool a los nuevos nodos + anima Transform.
- **Buscador-teletransporte**: match sobre el árbol completo (ya en memoria) → resuelve el path → setea `Nav.current` al padre del match + resalta el nodo (o abre la cadena con animación de saltos).
- **Labels egui**: por cada nodo visible, proyectar `world_to_viewport` y dibujar nombre + badge(count) en la capa egui.

## Fases (lean, estilo F7-9 — Codex construye, Claude verifica+commitea)

- **F10 — Modelo + layout por-carpeta**: índice path→hijos desde scan_folder; `folder_layout(path)` radial determinista; tests. Sin render nuevo aún.
- **F11 — Drill-down render + cámara orto**: pool de entidades, mostrar nivel actual (hub+hijos), cámara ortográfica, click=entrar / atrás=padre, animación de transición con FSM interrumpible. Checkpoint humano.
- **F12 — Labels egui + breadcrumb + parent-ghost**: world_to_viewport labels+badges, breadcrumb arriba, ghost del padre, preview de nietos en hover. Checkpoint humano.
- **F13 — Buscador-teletransporte + pins salto-al-pin**: search resuelve path y salta; pin → botón saltar. Checkpoint humano.

(v4 = capa neural: SDF glow, paleta teal, halos, badges estilizados — fuera de v3.)

## Out of scope v3
Glow/bloom/SDF, paleta neural final, apps/widgets, reemplazar KDE. Todo eso es v4+.

## Riesgos abiertos
- Transición multi-nivel del buscador (saltar 5 niveles): animar cada salto vs cut directo con breadcrumb. Decidir en F13.
- 2.5D ortográfica vs el mock 3D-ish: aceptado el cambio de feel (más plano).
