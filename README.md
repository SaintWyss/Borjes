# Borjes

Borjes es un navegador visual de carpetas para Linux, escrito en Rust y Bevy. Se abre como un overlay instantáneo, permite buscar en todo el árbol, reunir carpetas por tema y llegar a rutas profundas sin recorrer manualmente el explorador tradicional.

La versión 1.0 está diseñada como aplicación de escritorio real: instalación por usuario, autostart supervisado, configuración persistente, consumo zero-idle, tests y releases reproducibles.

## Funciones

- Búsqueda global sin acentos, ordenada por coincidencia y frecencia.
- Constelación de resultados agrupada por rama.
- Lentes temáticas transversales, por ejemplo `@seguridad`.
- Navegación radial con breadcrumb, historial y preview.
- Pins persistentes y accesos `Alt+1` a `Alt+9`.
- Radar contextual de rama y profundidad.
- Peso visual por cantidad de hijos y frecuencia de uso.
- Iconos PNG, emoji y modelos GLB/GLTF personalizados.
- Migración automática desde `datatree-rs`.

## Consumo

Borjes no mantiene un game loop permanente. Oculto, el event loop espera indefinidamente y se despierta por señal sin polling. Visible e interactuando renderiza fluidamente; cuando la escena queda quieta vuelve al modo reactivo.

La señal `SIGUSR1` es consumida por un hilo bloqueado en `sigwait`, que despierta Bevy mediante `EventLoopProxy`. Ver [PERFORMANCE.md](docs/PERFORMANCE.md).

## Instalación

Requiere Linux, Rust estable, Cargo, las bibliotecas nativas de Bevy y `systemd --user`.

En Arch/CachyOS:

```bash
sudo pacman -S --needed base-devel rust alsa-lib systemd-libs libx11 libxrandr libxi
bash install/install.sh
```

Luego vinculá tu atajo global, por ejemplo `Meta+1`, a:

```text
~/.local/bin/borjes-toggle
```

Desinstalar preservando los datos personales:

```bash
bash install/uninstall.sh
```

## Desarrollo

```bash
cargo run --release
pkill -USR1 -x borjes
```

Cambiar la raíz:

```bash
cargo run --release -- --root ~/Data
BORJES_ROOT=/otra/ruta cargo run --release
```

Usar otro directorio de assets:

```bash
BORJES_ASSET_DIR=/ruta/a/assets cargo run --release
```

## Controles

| Acción | Control |
|---|---|
| Mostrar u ocultar | Atajo global / `SIGUSR1` |
| Buscar | Escribir directamente |
| Seleccionar resultado | Flechas o `Tab` |
| Abrir | `Enter` |
| Entrar a una carpeta | Click o `Tab` sobre el nodo |
| Subir | `Backspace` o nodo padre |
| Volver por historial | Nodo violeta |
| Pin/Unpin | Click derecho |
| Abrir pin | `Alt+1..9` |
| Limpiar búsqueda | `Esc` |

## Configuración

Toda la configuración vive en `$XDG_CONFIG_HOME/borjes/` o `~/.config/borjes/`:

- `pins.json`
- `frecency.json`
- `lenses.json`
- `icons.json`
- `assets/`

Al iniciar se migran de forma idempotente los datos existentes de `~/.config/datatree-rs` sin sobrescribir estado nuevo.

## Calidad

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
```

GitHub Actions ejecuta esas verificaciones, RustSec y empaquetado en cada PR. Los tags `v*` generan un archivo Linux x86_64 con checksum SHA-256.

## Arquitectura

El flujo activo es drill-down por nivel. El árbol completo se indexa para búsqueda y lentes, pero sólo se renderiza el contexto visible. Ver [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Licencia

MIT.
