# Arquitectura

## Flujo principal

```text
sistema de archivos
  -> scan
  -> FolderTree + índice normalizado
  -> navegación / búsqueda / lentes
  -> nivel visible o constelación
  -> Bevy + egui
```

## Módulos

- `scan`: lectura defensiva de carpetas y archivos.
- `nav`: árbol indexado, padres, hijos y búsqueda por path.
- `search`: ranking, frecencia y experiencia type-first.
- `lens`: temas transversales configurables.
- `drill`: nivel radial, cámara, hover, previews, labels y animaciones.
- `constellation`: visualización global de coincidencias.
- `desktop`: integración Linux, foco y wake-up por señal.
- `performance`: render adaptativo y retorno automático a reposo.
- `config`: rutas XDG y migración de datos heredados.
- `settings`: configuración editable desde la aplicación.

## Persistencia

Los archivos se escriben atómicamente mediante temporal + rename. La ruta canónica es `~/.config/borjes` o su equivalente XDG.

## Límites de seguridad

Borjes navega y abre rutas, pero no mueve, modifica ni elimina contenido del usuario. Los archivos personalizados se cargan desde los assets instalados o desde el directorio de configuración del usuario.

