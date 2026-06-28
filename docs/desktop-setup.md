# datatree-rs — Modo escritorio (overlay on-demand)

## Qué es

`datatree-rs` se usa como un overlay invocable, no como una ventana-escritorio permanente. Arranca oculto (`visible: false`), no dibuja nada y consume ~0% CPU. Un atajo global lo muestra u oculta.

Cuando está visible, usa una ventana fullscreen borderless `AlwaysOnTop` con render `Reactive`.

## Por qué no es "ventana tipo Desktop X11"

El spike de fase 6 probó que setear `_NET_WM_WINDOW_TYPE_DESKTOP` en KDE Plasma no sirve para este caso: `plasmashell`, que también es `DESKTOP`, tapa la ventana; `Meta+D` no la revela; y el render continuo necesario para ese modo trababa la sesión, matando de hambre a KWin y haciendo lento `Shift+Tab`.

Por eso se pivoteó a un overlay on-demand.

## Cómo funciona

El proceso debe estar corriendo:

```bash
DISPLAY=:0 ~/dev/datatree-rs/target/release/datatree-rs &
```

Mostrar u ocultar el overlay:

```bash
pkill -USR1 -x datatree-rs
```

`Esc` oculta el overlay.

Cerrar el proceso completo:

```bash
pkill -x datatree-rs
```

## Atajo global KDE (Meta+1)

En Santiago-PC está configurado así:

- Se liberó `Meta+1` del taskbar, donde era "activar entrada 1 del gestor de tareas".
- Se creó `~/.local/share/applications/datatree-toggle.desktop` con `Exec=pkill -USR1 -x datatree-rs`.
- Se bindeó en `~/.config/kglobalshortcutsrc`, grupo `[services][datatree-toggle.desktop]`, con `_launch=Meta+1`.
- Se recargó KDE con:

```bash
systemctl --user restart plasma-kglobalaccel.service
```

Alternativa por GUI: Preferencias del sistema -> Atajos -> Atajos personalizados -> comando:

```bash
pkill -USR1 -x datatree-rs
```

Este bind es per-máquina, no sincroniza. Quedó registrado en `sync-control` op 016.

## Autostart al login (Fase 9)

El proceso arranca solo al iniciar sesión KDE (oculto), así Meta+1 lo muestra sin lanzarlo a mano. Entry en `~/.config/autostart/datatree-rs.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=datatree-rs overlay
Exec=/home/saintwyss/dev/datatree-rs/target/release/datatree-rs
X-KDE-autostart-after=panel
NoDisplay=true
```

Arranca con `visible: false` (idle ~0.6%). Si se cierra, el wrapper `~/.local/bin/datatree-toggle` (el que dispara Meta+1) lo relanza. Per-máquina, no sincroniza. Registrado en `sync-control` op 018.

Desactivar autostart: `rm ~/.config/autostart/datatree-rs.desktop`.

## Right-click / pin (Fase 8)

El pan de cámara está remapeado a **botón del medio** porque el botón derecho quedó reservado para el menú Pin/Unpin (default `button_pan` de PanOrbitCamera era Right y se comía el click). Orbit = izquierdo, pan = medio, zoom = rueda.

## Costo / rendimiento

- Oculto: ~0.6% CPU, idle real, no interfiere con la sesión.
- Al lanzar: el layout force-directed `O(N^2)` de ~4607 carpetas clava CPU unos segundos una vez, como cómputo único de arranque; después queda idle.
- Render `Reactive`: el grafo estático no se redibuja a full framerate.

## Troubleshooting

- `Meta+1` no hace nada: revisar si el proceso está corriendo con `pgrep -x datatree-rs`. Desde Fase 9 arranca solo al login; si no, el wrapper `~/.local/bin/datatree-toggle` lo relanza en el próximo Meta+1.
- Ruido en el log: `Couldn't get swap chain texture: Outdated` mientras está oculto es inofensivo. `Reactive` despierta periódicamente, pero no afecta el ~0.6%.
