# Testing y aceptación

## 1. Objetivo

Borjes combina lógica pura con comportamiento dependiente del escritorio, GPU, compositor y filesystem real. Por eso la validación se divide en pruebas automáticas y aceptación manual.

## 2. Cadena local recomendada

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

Estado actual: el repositorio no contiene un workflow de CI canónico. Estos comandos deben ejecutarse localmente antes de integrar cambios.

## 3. Cobertura automática existente

Las pruebas unitarias cubren, entre otros puntos:

- expansión de `~`;
- raíz inexistente sin panic;
- escaneo de carpetas y archivos;
- exclusión de directorios ocultos y ruido;
- límites de archivos mostrados;
- construcción y navegación de `FolderTree`;
- layout radial determinista;
- layout force-directed sin NaN o infinitos;
- normalización sin acentos;
- scoring y orden de búsqueda;
- rutas abreviadas de resultados;
- serialización atómica de pins;
- configuración y validación de iconos;
- parseo de colores de lentes;
- deduplicación de ramas en lentes;
- cálculo y persistencia de frecency.

## 4. Lo que no prueban los tests unitarios

- Recepción real de `SIGUSR1`.
- Foco de teclado después de ocultar y mostrar.
- Reaplicación de always-on-top en KWin.
- Compatibilidad con Wayland.
- Render correcto en distintos drivers/GPU.
- Interacción real de picking con mouse.
- Apertura de rutas mediante la asociación del sistema.
- Consumo real de CPU/GPU oculto y visible.
- Autostart y atajos del escritorio.
- Assets externos PNG/GLB arbitrarios.
- Directorios con permisos denegados, montajes remotos o millones de entradas en hardware real.

## 5. Smoke test de escritorio

Ejecutar sobre el binario release:

```bash
cargo build --release
./target/release/borjes
```

En otra terminal:

```bash
pkill -USR1 -x borjes
```

Checklist:

1. El proceso inicia sin ventana visible.
2. La señal muestra el overlay.
3. La ventana aparece en el monitor esperado.
4. El overlay queda por encima de ventanas normales.
5. El campo de búsqueda recibe teclado inmediatamente.
6. La búsqueda ignora mayúsculas y acentos.
7. `Tab` y flechas cambian el resultado enfocado.
8. `Enter` abre la carpeta elegida y oculta el overlay.
9. `Esc` limpia primero una consulta activa.
10. `Esc` con consulta vacía oculta el overlay.
11. `Backspace` con búsqueda vacía sube al padre.
12. Click o hover sostenido navega a una subcarpeta.
13. Click derecho permite personalizar y fijar una carpeta.
14. El pin aparece como chip y funciona con `Alt+n`.
15. Una lente `@nombre` muestra resultados temáticos.
16. Un segundo ciclo ocultar/mostrar conserva foco y always-on-top.
17. Cerrar y volver a abrir conserva pins, iconos, lentes y frecency.

## 6. Matriz mínima de compatibilidad

Registrar evidencia por combinación:

| Distribución | Escritorio | Sesión | GPU/driver | Resultado |
|---|---|---|---|---|
| CachyOS/Arch | KDE Plasma | X11 | completar | Camino principal |
| CachyOS/Arch | KDE Plasma | Wayland | completar | No verificado |
| Otra Linux | GNOME/KDE | X11/Wayland | completar | No verificado |

No declarar una combinación como soportada solo porque compila.

## 7. Prueba de persistencia

Con `XDG_CONFIG_HOME` temporal para no tocar datos reales:

```bash
TMP_CONFIG="$(mktemp -d)"
XDG_CONFIG_HOME="$TMP_CONFIG" ./target/release/borjes
```

Realice cambios de pin e icono, cierre y revise:

```bash
find "$TMP_CONFIG" -maxdepth 3 -type f -print
```

Nota: lentes y frecency usan todavía el subdirectorio heredado `datatree-rs`.

## 8. Prueba de raíz faltante

Renombre temporalmente el symlink o carpeta usada como `~/Data` y arranque Borjes. Debe registrar o mostrar un error de raíz y permanecer estable.

No ejecute esta prueba moviendo datos importantes sin backup.

## 9. Rendimiento

Medir por separado:

- tiempo de arranque;
- tiempo de escaneo;
- memoria residente;
- CPU oculto;
- CPU/GPU visible sin interacción;
- respuesta de búsqueda;
- latencia de mostrar/ocultar.

Ejemplos de observación:

```bash
/usr/bin/time -v ./target/release/borjes
pidstat -p "$(pgrep -x borjes)" 1
```

Los valores dependen de cantidad de carpetas, GPU, compositor y driver. No se deben publicar cifras universales a partir de una sola máquina.

## 10. Gates de release recomendados

Antes de etiquetar una versión:

- cadena Rust completa verde;
- smoke KDE/X11 aprobado;
- prueba de segundo show con foco y always-on-top;
- persistencia verificada con XDG temporal;
- revisión de archivos cambiados y formatos JSON;
- README y `RELEASE_STATUS.md` actualizados;
- binario probado desde una ruta sin depender del directorio de ejecución;
- ausencia de paths personales nuevos en documentación o scripts;
- licencia definida antes de distribuir públicamente.

## 11. Gaps prioritarios de testing

1. Test de migración de rutas `datatree-rs` a `borjes` cuando se implemente.
2. Test de configuración de raíz por CLI/archivo.
3. Integración automatizada del scanner con symlinks y permisos.
4. Pruebas de assets inválidos y archivos demasiado grandes.
5. CI Linux con fmt, check, test y clippy.
6. Build de artefacto reproducible.
7. Smoke automatizado bajo compositor virtual cuando sea viable.