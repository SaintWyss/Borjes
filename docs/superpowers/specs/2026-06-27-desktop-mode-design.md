# datatree-rs v2 — "Desktop mode" — Design

**Fecha:** 2026-06-27
**Estado:** Aprobado por el usuario, listo para roadmap (GSD milestone v2)
**Base:** v1 completa (scanner, render galaxia, cámara, buscador, visual encoding)

## Visión

Convertir datatree-rs de un explorador de archivos 3D en **la capa de escritorio** del usuario: el grafo vive de fondo (debajo de las ventanas) y se revela con **Mostrar escritorio (Meta+D)**. Reusa todo lo de v1.

## Core Value

El escritorio deja de ser un wallpaper estático: pasa a ser el mapa navegable de las carpetas y archivos del usuario, siempre a un Meta+D de distancia.

## Decisiones (locked)

| # | Decisión | Detalle |
|---|----------|---------|
| D1 | Ventana tipo Desktop X11 | winit setea `_NET_WM_WINDOW_TYPE_DESKTOP` nativo; KDE la trata como escritorio, queda bajo todo, Meta+D la revela. NO xwinwrap, NO always-below. |
| D2 | Revelado | Show Desktop (Meta+D) / minimizar ventanas. No overlay/hotkey al frente. |
| D3 | Nodos | Carpetas (v1, todas visibles) + archivos **on-demand** + pins. NO apps/launchers, NO widgets. |
| D4 | Archivos on-demand | **Click** en carpeta = toggle: aparecen/desaparecen sus archivos como nodos hoja (scan lazy de ese dir, un nivel). |
| D5 | Interacción | **Click = expandir** (toggle archivos de esa carpeta). **Doble-click = abrir** (carpeta→Dolphin, archivo→`xdg-open` app default). |
| D6 | Archivo nodo | Más chico, forma/color distinto (ej gris/diamante) para diferenciar de carpeta. |
| D7 | Pins | **Click-derecho → "Pin"** (menú egui). Pin siempre visible (sobrevive colapso/reescaneo), marca propia (borde dorado). Persistido en `~/.config/datatree-rs/pins.json`. |
| D8 | Expandir NO re-acomoda la galaxia | Archivos aparecen en **burst local radial** alrededor de su carpeta; el resto del grafo queda fijo. Colapsar los saca. |
| D9 | Perf | On-demand mantiene el conteo bajo → no se necesita Barnes-Hut todavía. |
| D10 | Autostart | App al login (autostart KDE, per-máquina). Registrar en sync-control. Pedir OK al activar. |
| D11 | Persistencia | Solo `pins.json` por ahora. Posición de cámara / estado expandido = YAGNI (diferido). |

## Riesgo principal (spike)

**Convivencia con el wallpaper de Plasma:** una ventana `_NET_WM_WINDOW_TYPE_DESKTOP` de otra app puede quedar tapada por el desktop/wallpaper que dibuja Plasma. Mitigación: poner el fondo de Plasma transparente/plano. Cambio de config Plasma = per-máquina, reversible, con OK del usuario. Es lo primero a validar.

## Fases tentativas (las afina el roadmap GSD)

1. **Ventana Desktop X11** — winit `_NET_WM_WINDOW_TYPE_DESKTOP` + spike de convivencia con wallpaper Plasma
2. **Archivos on-demand** — scan lazy de archivos por carpeta + burst local radial al expandir (sin re-layout global) + click=expandir / doble-click=abrir (xdg-open archivos)
3. **Pins + persistencia** — click-derecho→Pin, marca visual, `pins.json`, sobrevive reescaneo
4. **Autostart + integración escritorio** — autostart KDE, registro sync-control

## Fuera de scope (v2)

- Apps/launchers como nodos, widgets vivos
- Watch en vivo del FS (sigue diferido de v1)
- Wayland / multiplataforma (es X11 por AnyDesk)
- Reemplazar KDE/WM (eso sería v3, otro orden de magnitud)

## No cambia (de v1)

Layout galaxia force-directed, buscador egui acento-insensible, cámara orbital + zoom-on-search, color por cluster, tamaño por child_count, symlinks magenta, raíz configurable.
