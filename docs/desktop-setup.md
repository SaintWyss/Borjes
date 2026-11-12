# Integración de escritorio

Borjes se instala como un servicio de usuario y arranca oculto junto con la sesión gráfica.

## Instalación

```bash
./scripts/install.sh
```

El servicio se puede inspeccionar con:

```bash
systemctl --user status borjes.service
journalctl --user -u borjes.service
```

## Atajo KDE

Crear un atajo global para:

```text
~/.local/bin/borjes-toggle
```

La recomendación actual es `Meta+1`.

El wrapper inicia el servicio si fuera necesario, espera que el proceso esté disponible y envía `SIGUSR1`. El self-pipe interno despierta directamente el event loop; no existe polling cada 250 ms.

## Operaciones

```bash
systemctl --user restart borjes.service
systemctl --user stop borjes.service
systemctl --user start borjes.service
```

## Problemas frecuentes

- Si el atajo no responde, comprobar `systemctl --user status borjes`.
- Si faltan iconos, comprobar `~/.local/share/borjes`.
- Si no toma foco, crear una regla KWin para forzar foco y mantener arriba la ventana Borjes.
- La configuración personal está en `~/.config/borjes`.

