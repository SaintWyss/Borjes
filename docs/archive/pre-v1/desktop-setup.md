# Integración de escritorio Linux

## 1. Objetivo

Borjes funciona como overlay invocable, no como ventana permanente. El proceso queda activo y oculto; un atajo global envía `SIGUSR1` para mostrarlo u ocultarlo.

Esta guía evita rutas personales hardcodeadas y usa:

- checkout estable en `~/.local/opt/borjes`;
- servicio systemd de usuario;
- wrapper `~/.local/bin/borjes-toggle`;
- atajo global configurado desde el escritorio.

## 2. Prerrequisitos

Complete primero [`INSTALLATION.md`](INSTALLATION.md) y confirme que existe:

```text
$HOME/.local/opt/borjes/target/release/borjes
```

Validar:

```bash
test -x "$HOME/.local/opt/borjes/target/release/borjes"
```

## 3. Servicio systemd de usuario

Crear el directorio:

```bash
mkdir -p ~/.config/systemd/user
```

Crear `~/.config/systemd/user/borjes.service`:

```ini
[Unit]
Description=Borjes folder navigation overlay
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/opt/borjes/target/release/borjes
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
```

Cargar y habilitar:

```bash
systemctl --user daemon-reload
systemctl --user enable --now borjes.service
```

Verificar:

```bash
systemctl --user status borjes.service
pgrep -a -x borjes
```

El proceso debe estar activo sin mostrar una ventana.

## 4. Wrapper de toggle

Crear `~/.local/bin/borjes-toggle`:

```bash
mkdir -p ~/.local/bin
cat > ~/.local/bin/borjes-toggle <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if ! pgrep -x borjes >/dev/null; then
  systemctl --user start borjes.service
  for _ in {1..20}; do
    pgrep -x borjes >/dev/null && break
    sleep 0.1
  done
fi

if ! pgrep -x borjes >/dev/null; then
  printf 'Borjes no pudo iniciar. Revise: journalctl --user -u borjes.service\n' >&2
  exit 1
fi

pkill -USR1 -x borjes
EOF
chmod +x ~/.local/bin/borjes-toggle
```

Probar:

```bash
~/.local/bin/borjes-toggle
```

Ejecutarlo otra vez debe alternar la visibilidad.

## 5. Entrada de aplicación

Crear un `.desktop` con la ruta absoluta expandida por el shell:

```bash
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/borjes-toggle.desktop <<EOF
[Desktop Entry]
Type=Application
Name=Borjes
Comment=Mostrar u ocultar el navegador 3D de carpetas
Exec=$HOME/.local/bin/borjes-toggle
Icon=folder
Terminal=false
Categories=Utility;FileManager;
StartupNotify=false
EOF
```

Actualizar caché si la distribución dispone del comando:

```bash
update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

## 6. Atajo global en KDE Plasma

Camino recomendado por GUI:

1. Abrir **Preferencias del sistema**.
2. Ir a **Teclado → Atajos**.
3. Agregar un comando o aplicación personalizada.
4. Seleccionar `Borjes` o usar:

   ```text
   /home/USUARIO/.local/bin/borjes-toggle
   ```

5. Asignar el atajo elegido.

`Meta+1` suele estar reservado para activar la primera entrada del gestor de tareas. Si se usa, primero se debe eliminar el conflicto. También puede elegirse otro atajo, por ejemplo `Meta+Space` o `Meta+B`, según disponibilidad.

No copie literalmente `/home/USUARIO`; use la ruta producida por:

```bash
printf '%s/.local/bin/borjes-toggle\n' "$HOME"
```

## 7. Autostart alternativo con `.desktop`

Use esta opción solo si no quiere systemd user. No habilite ambos métodos a la vez.

```bash
mkdir -p ~/.config/autostart
cat > ~/.config/autostart/borjes.desktop <<EOF
[Desktop Entry]
Type=Application
Name=Borjes overlay
Exec=$HOME/.local/opt/borjes/target/release/borjes
NoDisplay=true
X-KDE-autostart-after=panel
EOF
```

Si usa este método, el wrapper debe iniciar el binario directamente cuando no esté activo, o se debe mantener systemd solo para recuperación. La configuración recomendada sigue siendo systemd user.

## 8. KDE/X11

Este es el camino principal.

La implementación vuelve a solicitar foco y nivel always-on-top después de mostrar la ventana porque KWin/X11 puede retirar esas propiedades durante un unmap.

Validar especialmente:

- primer show;
- segundo y tercer show;
- foco inmediato del buscador;
- overlay por encima de ventanas;
- `Esc` ocultando correctamente;
- cambio entre monitores.

## 9. Wayland

Wayland restringe global shortcuts, foco forzado y control de ventanas de manera distinta según compositor. El binario puede compilar y abrir, pero no se considera automáticamente soportado.

En KDE/Wayland:

- configure el atajo desde KDE, no desde un daemon de teclas X11;
- pruebe que `SIGUSR1` muestra la ventana;
- verifique si KWin permite always-on-top y foco;
- registre cualquier diferencia en la matriz de `TESTING.md`.

No intente resolver incompatibilidades desactivando seguridad del compositor o ejecutando el proceso como root.

## 10. Logs

Servicio:

```bash
journalctl --user -u borjes.service -n 200 --no-pager
journalctl --user -u borjes.service -f
```

Estado:

```bash
systemctl --user status borjes.service
pgrep -a -x borjes
```

## 11. Troubleshooting

### El atajo no hace nada

```bash
~/.local/bin/borjes-toggle
systemctl --user status borjes.service
journalctl --user -u borjes.service -n 100 --no-pager
```

Compruebe además que el atajo no esté en conflicto con otra acción.

### El proceso existe pero la ventana no aparece

```bash
pkill -USR1 -x borjes
```

Si funciona desde terminal pero no desde el atajo, el problema está en la configuración del escritorio o en la ruta del comando.

### La ventana aparece debajo

Este síntoma depende del compositor. Reinicie el servicio y pruebe de nuevo:

```bash
systemctl --user restart borjes.service
~/.local/bin/borjes-toggle
```

Si persiste, registre escritorio, sesión X11/Wayland, versión de KWin y logs.

### El buscador no recibe teclado

Oculte y muestre una vez. Si el problema es consistente, compruebe que otra ventana no esté activando políticas de prevención de robo de foco.

### La aplicación consume recursos oculta

```bash
pidstat -p "$(pgrep -x borjes)" 1
```

El proceso usa modo reactivo de bajo consumo cuando no tiene foco. Un consumo alto sostenido requiere revisar logs, cantidad de carpetas y driver gráfico.

### El servicio entra en restart loop

```bash
systemctl --user stop borjes.service
journalctl --user -u borjes.service -n 200 --no-pager
```

Ejecute el binario manualmente desde una terminal para obtener el error directo.

## 12. Actualización

```bash
cd ~/.local/opt/borjes
git pull --ff-only
cargo test
cargo build --release
systemctl --user restart borjes.service
```

## 13. Desactivar y desinstalar integración

```bash
systemctl --user disable --now borjes.service
rm -f ~/.config/systemd/user/borjes.service
rm -f ~/.local/bin/borjes-toggle
rm -f ~/.local/share/applications/borjes-toggle.desktop
rm -f ~/.config/autostart/borjes.desktop
systemctl --user daemon-reload
```

Después elimine manualmente el atajo global desde las preferencias del escritorio.

Este procedimiento no borra la configuración de Borjes ni el contenido de `~/Data`.