# datatree-rs

## What This Is

App de escritorio **nativa** (Rust + Bevy) que muestra el árbol de carpetas de `~/Data` como un **grafo 3D navegable tipo galaxia**, para llegar a cualquier carpeta sin recorrerla una por una en el explorador.

Cada carpeta-raíz de `~/Data` (00_Inbox, 10_Workspace, 20_Knowledge, 30_Cybersecurity, …) es un cúmulo con color propio; las subcarpetas orbitan. Cámara orbital, buscador que centra/resalta el nodo, doble-click abre la carpeta en Dolphin.

## Core Value

**Encontrar y abrir una carpeta profunda en segundos, visualmente.** Si el grafo no deja llegar a `AdSI` (enterrada en `20_Knowledge/UTN_FRRe/Ingeniería_en_Sistemas/4_cuarto_año/`) más rápido que el explorador, falló.

## Context

- Surgió de querer un mapa visual del FS; se descartó un preview web (React) porque el usuario quiere un **programa nativo de la PC**, no un sitio.
- Se descartó graphify para esto: grafica *contenido*, no el árbol de carpetas.
- Hardware: RTX 3070 (Vulkan OK), 16 cores, CachyOS KDE X11. Bevy renderiza nativo en GPU.
- Material vivo del usuario (facu) vive en árbol profundo `20_Knowledge/UTN`; ese es el caso de uso estrella.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Escanear `~/Data` y construir el árbol de carpetas (excluir node_modules/.git/cache/Steam)
- [ ] Render del grafo 3D "galaxia": cúmulo por carpeta-raíz, color por área, subcarpetas orbitando (layout force-directed agrupado)
- [ ] Cámara orbital: rotar, zoom, pan
- [ ] Color de nodo por área raíz; tamaño de nodo por cantidad de subcarpetas / peso en disco
- [ ] Buscador: filtra/centra/resalta el nodo coincidente (acento-insensible)
- [ ] Doble-click en nodo → abre la carpeta en Dolphin (xdg-open)
- [ ] Raíz configurable desde la app (default `~/Data`)

### Out of Scope

- Watch en vivo (notify + difusión al render) — v2, agrega complejidad
- Mostrar archivos como nodos hoja — v2, dispara a ~24k+ nodos, necesita LOD/culling
- Editar/mover/borrar carpetas desde la app — solo navegación por ahora
- Multiplataforma (Win/Mac) — es para esta PC Linux

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Bevy (nativo) en vez de web/Tauri | Usuario quiere programa de PC + "hyper rápido"; Bevy = 3D GPU nativo | — Pending |
| Metáfora "galaxia por área" | Clusters coloreados por raíz = legible de un vistazo con 604 nodos | — Pending |
| Solo carpetas en v1 (no archivos) | 604 nodos render fácil; archivos = 24k+, necesita LOD | — Pending |
| Raíz `~/Data` configurable | Caso de uso real; flexible sin abrir todo el FS | — Pending |
| Abrir en Dolphin vía xdg-open | KDE nativo; el explorador del usuario | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-06-26 after initialization*
