# Configuración y persistencia

## 1. Principio general

Borjes es local-first. No envía telemetría ni usa servicios remotos. Su configuración se deriva del código y de archivos JSON bajo el directorio XDG del usuario.

Si `XDG_CONFIG_HOME` está definido, se usa como base. En caso contrario se usa `~/.config`.

## 2. Raíz de navegación

La raíz activa predeterminada es:

```text
~/Data
```

Se define en `RootConfig::default()` dentro de `src/root.rs`.

Estado actual:

- `~` se expande con la variable `HOME`;
- una ruta inexistente produce un error visible, no un panic;
- el panel heredado de `ui.rs` permite editar y reescanear una raíz, pero ese sistema no está registrado en el flujo principal;
- no existe argumento CLI, variable de entorno ni archivo de configuración canónico para cambiar la raíz.

Por lo tanto, cambiar la raíz de forma soportada requiere hoy crear un symlink `~/Data` o modificar el código.

Ejemplo con symlink:

```bash
ln -s /mnt/hdd/Data ~/Data
```

Antes de hacerlo confirme que `~/Data` no contiene datos que serían reemplazados por el enlace.

## 3. Mapa de archivos

| Función | Ruta con XDG | Fallback |
|---|---|---|
| Pins | `$XDG_CONFIG_HOME/borjes/pins.json` | `~/.config/borjes/pins.json` |
| Iconos | `$XDG_CONFIG_HOME/borjes/icons.json` | `~/.config/borjes/icons.json` |
| Frecency | `$XDG_CONFIG_HOME/datatree-rs/frecency.json` | `~/.config/datatree-rs/frecency.json` |
| Lentes | `$XDG_CONFIG_HOME/datatree-rs/lenses.json` | `~/.config/datatree-rs/lenses.json` |
| Assets externos | `$XDG_CONFIG_HOME/borjes/assets/` | `~/.config/borjes/assets/` |

Las rutas `datatree-rs` son una herencia del nombre anterior del proyecto. No deben renombrarse manualmente sin cambiar también el código o implementar una migración compatible.

## 4. Pins

`pins.json` es un array JSON de rutas absolutas:

```json
[
  "/home/user/Data/10_Workspace",
  "/home/user/Data/30_Cybersecurity"
]
```

Los pins se administran desde el menú contextual de una carpeta. Borjes muestra hasta nueve chips ordenados por nombre y permite abrirlos con `Alt+1` a `Alt+9`.

Consideraciones:

- mover o renombrar una carpeta deja un pin inválido;
- sincronizar el archivo entre equipos con homes diferentes requiere adaptar rutas;
- un JSON corrupto se trata como conjunto vacío;
- la escritura usa un archivo temporal y rename para reducir corrupción parcial.

## 5. Iconos y modelos

`icons.json` usa un objeto `folders` cuyas claves pueden ser rutas absolutas o nombres de carpeta.

```json
{
  "folders": {
    "/home/user/Data/30_Cybersecurity": {
      "kind": "png",
      "asset": "icons/shield.png",
      "scale": 1.0,
      "halo": true
    },
    "20_Knowledge": {
      "kind": "emoji",
      "emoji": "📚",
      "scale": 1.1,
      "halo": true
    },
    "Robotics": {
      "kind": "model",
      "asset": "custom/robot.glb",
      "scale": 0.8,
      "halo": false
    }
  }
}
```

Tipos admitidos:

- `sphere`;
- `png`;
- `emoji`;
- `model`.

Resolución:

1. override por ruta absoluta;
2. override por nombre;
3. visual predeterminado.

`scale` se normaliza al rango `0.2`–`4.0`.

Los assets pueden apuntar a:

- rutas relativas dentro del directorio `assets/` del repositorio;
- archivos de usuario bajo `~/.config/borjes/assets/`;
- rutas absolutas, cuando el loader las admite.

No cargue assets de origen desconocido sin revisar formato y licencia.

## 6. Lentes temáticas

El archivo `lenses.json` contiene una lista:

```json
[
  {
    "name": "Seguridad",
    "keywords": ["seguridad", "cyber", "kali", "pentest"],
    "color": "#ff6b6b"
  },
  {
    "name": "Recursos",
    "keywords": ["recursos", "administracion"],
    "color": "#8aff6b"
  }
]
```

Si el archivo no existe, Borjes crea lentes predeterminadas. Si existe pero es inválido, registra un warning y usa defaults en memoria.

Uso:

- click en un chip de lente;
- escribir `@Seguridad` o un prefijo normalizado como `@segu`.

El color debe ser hexadecimal RGB de seis dígitos. El editor gráfico de lentes todavía no existe.

## 7. Frecency

`frecency.json` asocia rutas con puntuación y última apertura:

```json
{
  "/home/user/Data/10_Workspace": {
    "score": 3.5,
    "last": 1784100000
  }
}
```

Cada apertura incrementa la puntuación después de aplicar decaimiento. La vida media implementada es de catorce días.

Este archivo no debería editarse manualmente salvo recuperación. Borrarlo reinicia el ranking sin afectar carpetas ni pins.

## 8. Copias de seguridad

```bash
mkdir -p ~/Backups/borjes
cp -a ~/.config/borjes ~/Backups/borjes/borjes-config
cp -a ~/.config/datatree-rs ~/Backups/borjes/legacy-config 2>/dev/null || true
```

Los archivos contienen rutas locales, no secretos. Aun así pueden revelar nombres y estructura de proyectos personales.

## 9. Restauración

Con Borjes detenido:

```bash
pkill -x borjes || true
cp -a ~/Backups/borjes/borjes-config ~/.config/borjes
cp -a ~/Backups/borjes/legacy-config ~/.config/datatree-rs 2>/dev/null || true
```

Después inicie el proceso y valide pins, lentes e iconos.

## 10. Reset selectivo

```bash
rm ~/.config/borjes/pins.json
rm ~/.config/borjes/icons.json
rm ~/.config/datatree-rs/frecency.json
rm ~/.config/datatree-rs/lenses.json
```

Cada archivo se puede eliminar de manera independiente. No use `rm -rf ~/.config` ni comandos amplios para resetear una sola aplicación.