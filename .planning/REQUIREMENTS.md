# Requirements — datatree-rs (v1)

## v1 Requirements

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
