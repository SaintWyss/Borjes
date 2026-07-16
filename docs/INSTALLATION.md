# Instalación y compilación

## 1. Alcance

Borjes se distribuye actualmente como código fuente. Esta guía produce un binario local `target/release/borjes`; no crea un paquete firmado ni instala automáticamente atajos o autostart.

## 2. Plataformas

| Plataforma | Estado |
|---|---|
| Linux + KDE Plasma + X11 | Camino principal y mejor documentado |
| Linux + KDE Plasma + Wayland | Compilable; foco y always-on-top requieren aceptación manual |
| Otros escritorios Linux | No verificados sistemáticamente |
| Windows | No soportado por el mecanismo actual de señales Unix |
| macOS | No soportado ni probado |

## 3. Requisitos

- Git.
- Rust estable con Cargo.
- Compilador C/C++ y linker.
- `pkg-config`.
- Bibliotecas de desarrollo Linux requeridas por Bevy/winit para ventana, input, audio y dispositivos.
- Driver gráfico funcional con soporte Vulkan compatible con Bevy.
- Sesión gráfica iniciada.

Los nombres exactos de paquetes cambian por distribución. Como referencia, normalmente se requieren familias de paquetes para:

- X11/XCB;
- Wayland y xkbcommon;
- ALSA;
- udev;
- toolchain de compilación;
- `pkg-config`.

No copie listas antiguas de paquetes sin revisar el gestor de su distribución y los requisitos de la versión de Bevy fijada en `Cargo.toml`.

## 4. Instalar Rust

Se recomienda administrar Rust mediante `rustup`.

```bash
rustup toolchain install stable
rustup default stable
rustc --version
cargo --version
```

El repositorio no fija actualmente una versión exacta con `rust-toolchain.toml`; por eso una actualización futura de stable puede descubrir incompatibilidades nuevas.

## 5. Obtener el código

```bash
git clone https://github.com/SaintWyss/Borjes.git
cd Borjes
```

El repositorio es privado; la cuenta o credencial Git debe tener acceso.

## 6. Preparar la raíz

El flujo activo usa `~/Data`.

```bash
mkdir -p ~/Data
```

Puede ser una carpeta real o un symlink hacia otro volumen. El proceso seguirá symlinks según la política implementada por el scanner; valide permisos y evite apuntar a raíces excesivamente grandes sin medir consumo.

La raíz todavía no se puede cambiar mediante un argumento CLI soportado.

## 7. Validar el checkout

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Si faltan bibliotecas nativas, Cargo mostrará el crate o binding que no pudo compilar. Instale el paquete de desarrollo correspondiente y repita `cargo check`.

## 8. Compilar release

```bash
cargo build --release
```

Resultado:

```text
target/release/borjes
```

El binario usa assets del repositorio mediante una ruta derivada de `CARGO_MANIFEST_DIR` en tiempo de compilación. En el estado actual, no se debe copiar solamente el binario y asumir que todos los assets seguirán resolviendo. Para una instalación estable conserve el checkout o implemente un layout de assets instalable.

## 9. Probar manualmente

En una terminal gráfica:

```bash
./target/release/borjes
```

En otra terminal:

```bash
pkill -USR1 -x borjes
```

Validar:

1. el overlay aparece;
2. recibe foco de teclado;
3. permanece sobre las ventanas normales;
4. la búsqueda responde;
5. `Esc` limpia la búsqueda y luego oculta;
6. puede navegar a una carpeta;
7. `Enter` abre la carpeta con la aplicación del sistema;
8. un segundo `SIGUSR1` vuelve a mostrarlo.

Detener:

```bash
pkill -x borjes
```

## 10. Instalación local conservando el repositorio

Una opción simple para uso personal es mantener el checkout en una ruta estable:

```text
$HOME/.local/opt/borjes/
```

Ejemplo:

```bash
mkdir -p ~/.local/opt
mv Borjes ~/.local/opt/borjes
cd ~/.local/opt/borjes
cargo build --release
```

Después use la ruta absoluta del binario en autostart o systemd user:

```text
/home/USUARIO/.local/opt/borjes/target/release/borjes
```

No copie literalmente `USUARIO`; reemplácelo por el home real.

## 11. Actualizar

Desde el checkout estable:

```bash
git status --short
git pull --ff-only
cargo test
cargo build --release
```

Luego reinicie el proceso:

```bash
pkill -x borjes || true
./target/release/borjes &
```

Si usa systemd user:

```bash
systemctl --user restart borjes.service
```

## 12. Rollback

Antes de actualizar, registre el commit actual:

```bash
git rev-parse HEAD
```

Si la nueva versión falla:

```bash
git checkout <commit-anterior>
cargo build --release
systemctl --user restart borjes.service
```

Los JSON de configuración no tienen versionado formal. Antes de una actualización que cambie formatos, respalde:

```bash
cp -a ~/.config/borjes ~/.config/borjes.backup
cp -a ~/.config/datatree-rs ~/.config/datatree-rs.backup 2>/dev/null || true
```

## 13. Desinstalar

Detenga y quite la integración de escritorio según [`desktop-setup.md`](desktop-setup.md), luego elimine el checkout.

Los datos del usuario permanecen en:

```text
~/.config/borjes/
~/.config/datatree-rs/
```

Elimínelos solo si no necesita pins, iconos, lentes o historial de frecency.