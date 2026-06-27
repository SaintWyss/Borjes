# Requirements — datatree-rs

## v2 "Desktop mode" Requirements (milestone activo)

### Desktop (integración X11/KDE)
> **Decisión 2026-06-27 (dobla D1):** el spike 06-01 probó que la ventana tipo Desktop X11 puro NO sirve en este KDE — plasmashell la tapa, Meta+D no la revela, y el render continuo necesario trababa la sesión. Reinterpretado a **overlay on-demand**: oculto por default (~0% CPU), se invoca con un atajo. Ver `.planning/phases/06-ventana-desktop-x11/06-01-SUMMARY.md`.
- [ ] **DESK-01**: La app corre como **overlay on-demand**: ventana borderless fullscreen, `WindowLevel::AlwaysOnTop`, arranca oculta (`visible:false`), render Reactive; se muestra/oculta con SIGUSR1 (Esc oculta). (Antes: ventana tipo Desktop X11 — inviable, ver decisión arriba.)
- [ ] **DESK-02**: El grafo se revela con un atajo global de KDE (`pkill -SIGUSR1`), conviviendo con el desktop existente sin pelear con el wallpaper (al mostrarse va arriba; oculto no dibuja nada). AnyDesk sigue funcionando.
- [ ] **DESK-03**: La app se lanza automáticamente al login, oculta (autostart KDE, per-máquina) — Fase 9

### Files (archivos on-demand)
- [ ] **FILE-01**: Click en una carpeta expande/colapsa sus archivos como nodos hoja (scan lazy de ese dir, un nivel)
- [ ] **FILE-02**: Los nodos-archivo se ven distintos de las carpetas (tamaño/forma/color)
- [ ] **FILE-03**: Doble-click en un archivo lo abre con su app default (`xdg-open`)
- [ ] **FILE-04**: Al expandir, los archivos aparecen en burst local alrededor de su carpeta SIN re-acomodar toda la galaxia

### Interaction
- [ ] **INT-01**: Click = expandir (toggle archivos); doble-click = abrir (carpeta→Dolphin, archivo→app default)

### Pins
- [ ] **PIN-01**: Click-derecho en un nodo → "Pin" (menú egui)
- [ ] **PIN-02**: Los pins persisten en `~/.config/datatree-rs/pins.json` y sobreviven colapso/reescaneo
- [ ] **PIN-03**: Los nodos pinneados tienen marca visual propia (ej borde dorado) y quedan siempre visibles

## v2 Traceability
| REQ | Fase |
|-----|------|
| DESK-01, DESK-02 | 6 |
| FILE-01..04, INT-01 | 7 |
| PIN-01..03 | 8 |
| DESK-03 | 9 |

---

## v1 Requirements — VALIDADOS ✅ (milestone v1 completo)

### Scan (FS)
- [ ] **SCAN-01**: La app escanea `~/Data` y construye el árbol de carpetas excluyendo node_modules/.git/.cache/Steam/target/__pycache__/.Trash
- [ ] **SCAN-02**: Los symlinks (ej. `01_Now → 20_Knowledge/…`) se incorporan al grafo sin loops (canonicalize + detección de ciclos por inode), marcados como symlink

### Graph / Layout
- [ ] **GRAPH-01**: Render 3D del grafo: una esfera por carpeta, líneas (edges) padre→hijo
- [ ] **GRAPH-02**: Layout "galaxia" force-directed que agrupa cada carpeta-raíz de ~/Data en un cúmulo separado (atracción al centroide del cluster)

### Cámara
- [ ] **CAM-01**: Cámara orbital: rotar, zoom y pan con mouse

### Visual
- [ ] **VIS-01**: Cada nodo coloreado según su área raíz (00_Inbox, 10_Workspace, 20_Knowledge, …)
- [ ] **VIS-02**: Tamaño del nodo según cantidad de subcarpetas (child_count)

### Búsqueda
- [ ] **SEARCH-01**: Buscador que filtra acento-insensible, resalta y centra la cámara en el nodo coincidente

### Abrir
- [ ] **OPEN-01**: Doble-click en un nodo abre esa carpeta en Dolphin (xdg-open, en thread aparte)

### Raíz
- [ ] **ROOT-01**: La raíz a escanear es configurable desde la app (default `~/Data`)

## v2 (deferred)
- Watch en vivo (notify) — actualizar grafo al crear/mover/borrar carpetas
- Mostrar archivos como nodos hoja (necesita LOD/culling, ~24k+ nodos)
- Hover labels más ricos / minimapa

## Out of Scope
- Editar/mover/borrar carpetas desde la app — solo navegación
- Multiplataforma Win/Mac — es para esta PC Linux/KDE

## Traceability
| REQ | Fase |
|-----|------|
| SCAN-01, SCAN-02 | 1 |
| GRAPH-01, CAM-01 | 2 |
| GRAPH-02 | 3 |
| OPEN-01, SEARCH-01 | 4 |
| VIS-01, VIS-02, ROOT-01 | 5 |
