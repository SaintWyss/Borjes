# Estado de release

## 1. Identificación

- Crate: `borjes`
- Versión declarada en `Cargo.toml`: `0.1.0`
- Plataforma principal: Linux desktop
- Experiencia principal: overlay radial por nivel
- Distribución actual: compilación desde fuente

## 2. Estado por capacidad

| Capacidad | Estado | Evidencia principal |
|---|---|---|
| Escaneo recursivo y filtrado | Implementado | `scan.rs` + tests |
| Árbol indexado y navegación | Implementado | `nav.rs`, `drill/` + tests |
| Overlay oculto/visible por señal | Implementado, aceptación manual | `desktop.rs`, `main.rs` |
| Foco y always-on-top KDE/X11 | Implementado, dependiente del compositor | `desktop.rs` |
| Búsqueda global | Implementado | `search.rs` + tests |
| Constelación de resultados | Implementado | `constellation.rs` |
| Frecency | Implementado | `frecency.rs` + tests |
| Lentes temáticas | Implementado | `lens.rs` + tests |
| Pins persistentes | Implementado | `pins.rs`, `drill/overlay.rs`, `search.rs` + tests |
| Iconos, emoji, PNG y GLB | Implementado | `icons.rs`, `drill/overlay.rs` + tests parciales |
| Apertura con app del sistema | Implementado, aceptación manual | `open::that` en búsqueda/interacción |
| Configuración de raíz desde UI | No conectada | panel heredado en `ui.rs` |
| Configuración de raíz por CLI | No implementada | sin parser CLI |
| Migración de config `datatree-rs` | No implementada | rutas heredadas en lentes/frecency |
| Autostart/atajo automático | No implementado | configuración manual documentada |
| Paquete AppImage/DEB/RPM | No implementado | sin pipeline de packaging |
| CI | No implementado en el repo | sin workflow canónico |
| Windows/macOS | No soportado | dependencia de señales Unix y falta de aceptación |

## 3. Qué puede considerarse usable

La aplicación puede considerarse usable para un usuario técnico cuando:

- utiliza Linux;
- dispone de `~/Data`;
- puede compilar Rust/Bevy;
- acepta configurar manualmente autostart y atajo;
- valida el comportamiento sobre su compositor y GPU;
- mantiene el checkout junto con assets.

No debe presentarse todavía como aplicación instalable para usuarios generales.

## 4. Riesgos residuales

### Compatibilidad de escritorio

El foco y always-on-top son comportamientos del window manager. El workaround actual está orientado a KWin/X11 y puede no trasladarse a Wayland u otros escritorios.

### Configuración dividida

Pins e iconos usan `borjes`, pero lentes y frecency usan el directorio heredado `datatree-rs`. Esto complica backup, soporte y migración.

### Distribución de assets

El asset root se deriva de `CARGO_MANIFEST_DIR` en compilación. Copiar solo el binario fuera del checkout no constituye una instalación soportada.

### Raíz fija

El usuario no puede seleccionar de manera canónica una raíz distinta. Un symlink `~/Data` es el workaround operativo.

### Filesystem local

El scanner procesa nombres y estructura de la raíz con permisos del usuario. Árboles enormes, montajes lentos o permisos irregulares pueden afectar tiempo de arranque y experiencia.

### Licencia

No existe un archivo `LICENSE`. La distribución pública o de binarios debe esperar una decisión explícita.

## 5. Gates para v0.2

- Añadir configuración de raíz por CLI o archivo XDG.
- Unificar rutas de configuración con migración desde `datatree-rs`.
- Agregar CI con fmt, check, test y clippy.
- Separar assets del path de compilación.
- Producir al menos un artefacto Linux reproducible.
- Registrar matriz real KDE/X11 y KDE/Wayland.
- Definir licencia.
- Retirar o aislar claramente el camino de grafo global heredado.

## 6. Gates para distribución general

- Instalador o paquete firmado.
- Integración automática de menú, autostart y atajo con rollback.
- Configuración gráfica de raíz.
- Mensajes de error orientados al usuario.
- Upgrade y desinstalación probados.
- Compatibilidad declarada por distribución/escritorio.
- Política de assets y licencias de iconos/modelos.
- Release notes y checksums por versión.

## 7. Definición de terminado documental

Una capacidad solo debe figurar como implementada cuando:

1. está conectada en `main.rs` o en un camino llamado por sistemas activos;
2. existe evidencia en código;
3. tiene pruebas automáticas cuando la lógica es aislable;
4. tiene aceptación manual documentada cuando depende del escritorio;
5. sus límites y persistencia están documentados;
6. no depende de paths personales no parametrizados.