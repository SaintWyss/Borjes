# Arquitectura de Borjes 1.0

## Principio

El filesystem completo se mantiene como modelo indexado, mientras la GPU recibe solamente el nivel visible. Navegación, búsqueda temática y render están separados.

## Flujo

1. `scan` produce `FolderNode` y filtra ruido.
2. `FolderTree` construye relaciones e índice normalizado.
3. `NavState` mantiene carpeta actual e historial.
4. `drill::render_level` materializa hub, hijos, padre y eco.
5. Búsqueda y lentes montan constelaciones virtuales.
6. `PowerGovernor` duerme el event loop cuando la escena queda estable.

## Seguridad

- Borjes no mueve, renombra ni elimina archivos.
- Las aperturas se delegan al sistema.
- Los enlaces simbólicos no se siguen recursivamente.
- Directorios ocultos y de ruido se excluyen.
- Los bursts se limitan a 200 archivos ordenados.

## Integración Linux

El binario permanece oculto y supervisado por un servicio de usuario. El toggle envía `SIGUSR1`. Un thread dedicado usa `sigwait` y `EventLoopProxy<WinitUserEvent>` para despertar el loop sin temporizadores.
