# 07 — Operación, despliegue y recuperación

## Instalación desde fuente

```bash
git clone <repositorio>
cd Borjes
cargo build --release
```

Una instalación real debe copiar:

- binario;
- assets;
- icono;
- desktop entry;
- wrapper de toggle;
- servicio de usuario si corresponde.

Yo no considero soportado copiar solo `target/release/borjes` cuando el runtime todavía busca assets en el checkout.

## Ubicaciones recomendadas

```text
~/.local/bin/borjes
~/.local/share/borjes/assets/
~/.local/share/applications/borjes.desktop
~/.config/systemd/user/borjes.service
~/.config/borjes/
```

Las rutas exactas deben coincidir con el instalador vigente.

## Autostart y service

Un servicio de usuario debe:

- ejecutar sin root;
- esperar la sesión gráfica;
- reiniciar solo ante fallos reales;
- registrar logs en journal;
- usar paths absolutos portables basados en `$HOME`/XDG;
- evitar instancias duplicadas.

## Atajo KDE

Yo configuro un atajo que invoque un wrapper estable, no el path del checkout. El wrapper inicia o muestra la instancia.

Después verifico:

- conflicto con otros shortcuts;
- foco;
- comportamiento en múltiples monitores;
- cierre de sesión;
- reinicio del servicio.

## Actualización

1. respaldar configuración;
2. registrar versión actual;
3. compilar nueva versión;
4. ejecutar tests;
5. instalar binario y assets juntos;
6. reiniciar servicio;
7. smoke de escritorio;
8. medir consumo;
9. conservar versión anterior para rollback.

## Rollback

- restaurar binario y assets de la misma versión;
- restaurar config solo si hubo migración incompatible;
- reiniciar servicio;
- comprobar atajo, scan, búsqueda y pins;
- no mezclar binario nuevo con assets viejos.

## Desinstalación

Yo elimino únicamente archivos instalados por Borjes:

- binario;
- assets;
- desktop entry;
- service y symlink de enable;
- caches opcionales.

No elimino configuración/pins sin una confirmación separada.

## Backup

```bash
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "backups/borjes-${stamp}"
cp -a "${XDG_CONFIG_HOME:-$HOME/.config}/borjes" \
  "backups/borjes-${stamp}/" 2>/dev/null || true
cp -a "${XDG_CONFIG_HOME:-$HOME/.config}/datatree-rs" \
  "backups/borjes-${stamp}/" 2>/dev/null || true
sha256sum "backups/borjes-${stamp}"/**/* 2>/dev/null \
  > "backups/borjes-${stamp}/SHA256SUMS" || true
```

Yo protejo el backup porque contiene paths y hábitos personales.

## Restauración

1. detener Borjes;
2. verificar checksums;
3. respaldar el estado actual;
4. restaurar un grupo de archivos por vez;
5. validar JSON;
6. iniciar sin atajo para ver logs;
7. probar pins/lentes/frecency;
8. reactivar autostart.

## Config corrupta

Ante un JSON inválido:

- no borro todo;
- copio el archivo;
- valido con una herramienta JSON;
- identifico el módulo afectado;
- reseteo solo ese archivo;
- agrego una prueba si la aplicación debió tolerarlo.

## Problema gráfico

1. ejecutar desde terminal;
2. capturar logs de wgpu/winit;
3. registrar GPU, driver, compositor y sesión;
4. probar backend/opciones soportadas;
5. desactivar efectos no esenciales si existe configuración;
6. volver a una versión conocida;
7. no atribuirlo al filesystem sin evidencia.

## Alto consumo

Yo mido:

```bash
ps -C borjes -o pid,%cpu,%mem,etime,cmd
```

Y uso herramientas de GPU apropiadas. Comparo visible vs oculto, con y sin interacción, durante una ventana suficiente.

Si el consumo oculto es alto:

- verifico timers/polling;
- eventos de ventana;
- render continuo;
- animaciones que nunca terminan;
- watcher de filesystem;
- múltiples instancias.

## Logs

```bash
journalctl --user -u borjes.service -n 200 --no-pager
```

Yo evito compartir logs sin revisar paths personales.

## Aceptación por entorno

Registro:

- distribución;
- kernel;
- escritorio;
- X11/Wayland;
- compositor;
- GPU/driver;
- resolución/escala;
- monitores;
- filesystem raíz;
- versión Borjes.

## Evidencia mínima de incidente

- timestamp;
- versión/commit;
- acción del usuario;
- estado visible/oculto;
- logs sanitizados;
- config relevante;
- consumo;
- entorno gráfico;
- pasos de recuperación;
- resultado.