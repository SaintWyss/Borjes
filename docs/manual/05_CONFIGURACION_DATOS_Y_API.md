# 05 — Configuración, datos y contratos del escritorio

## Idea general

Borjes no necesita una API web. Sus contratos externos principales son el filesystem, los archivos XDG, la sesión gráfica y los comandos seguros para abrir recursos.

Yo separo:

- código y assets instalados;
- configuración editable;
- estado personal como pins/frecency;
- datos reales del filesystem, que Borjes solo observa y abre.

## Raíz de navegación

El flujo principal documentado parte de:

```text
~/Data
```

Si la raíz no existe, yo prefiero un error claro antes que navegar otra carpeta por accidente.

El cambio de raíz debe considerarse activo únicamente cuando:

- exista un ajuste conectado al runtime;
- se valide el path;
- se persista de forma XDG;
- tenga pruebas;
- el scanner y la búsqueda usen la misma raíz.

## Rutas XDG

El objetivo de portabilidad es usar rutas equivalentes a:

```text
$XDG_CONFIG_HOME/borjes/
$XDG_DATA_HOME/borjes/
$XDG_CACHE_HOME/borjes/
```

Si las variables no están definidas se usan defaults bajo `$HOME`.

El estado actual puede conservar archivos heredados en:

```text
~/.config/datatree-rs/
```

Yo documento esa deuda y no muevo datos automáticamente sin una migración no destructiva.

## Pins

Formato conceptual:

```json
{
  "pins": [
    {"path": "/home/usuario/Data/Proyectos", "label": "Proyectos"}
  ]
}
```

Reglas:

- paths absolutos normalizados;
- no ejecutar contenido del archivo;
- tolerar entradas inexistentes;
- escribir de forma atómica;
- conservar backup ante migración.

## Lentes

Una lente puede definir filtros, etiquetas o ponderaciones. El esquema exacto debe estar versionado o validado al cargar.

Yo rechazo o ignoro de forma segura:

- JSON inválido;
- expresiones desconocidas;
- paths fuera de alcance cuando corresponda;
- valores numéricos no finitos;
- assets inexistentes.

## Frecency

El store conserva visitas y timestamps. Es información local de comportamiento y debe:

- permanecer privada;
- poder resetearse;
- limitar crecimiento;
- tolerar paths movidos;
- no bloquear el arranque si se corrompe.

## Iconos y assets

Los assets incluyen fuentes, texturas e iconos. Una instalación soportada debe resolverlos desde una ruta estable, no desde el directorio del repositorio.

Yo defino precedencia:

1. icono personalizado válido;
2. icono por categoría/tipo;
3. fallback incluido en la aplicación.

## Scanner de filesystem

Entrada:

- path raíz o actual;
- política de symlinks;
- límites;
- opciones de exclusión.

Salida conceptual:

```text
FolderNode { path, name, kind, child_count, metadata }
```

Yo no leo contenido de archivos para construir el árbol salvo que una feature lo requiera explícitamente.

## Búsqueda

El contrato de búsqueda devuelve resultados con:

- path completo;
- nombre;
- tipo;
- score/criterio;
- contexto suficiente.

La búsqueda no debe devolver un path fuera de la raíz configurada salvo que el producto lo declare y lo muestre claramente.

## Integración de escritorio

### Abrir carpeta

Yo delego en mecanismos seguros del sistema como `xdg-open` o APIs equivalentes, pasando el path como argumento separado. No construyo una cadena de shell con texto del filesystem.

### Atajo/toggle

El wrapper debe:

- identificar la instancia;
- enviar el mecanismo soportado de wake/toggle;
- iniciar si no existe;
- evitar procesos duplicados;
- no usar permisos root.

### Desktop entry

Debe declarar:

- nombre;
- `Exec` estable;
- icono instalado;
- categorías;
- comportamiento de terminal;
- path independiente del checkout.

## Logs

Yo registro:

- errores de scan;
- config inválida;
- assets faltantes;
- backend gráfico;
- fallos de apertura;
- migraciones de configuración.

No registro el árbol completo de carpetas por defecto, porque puede revelar información personal.

## Compatibilidad

La configuración debe incluir o documentar:

- Linux/distribución;
- KDE/GNOME/otros;
- X11/Wayland;
- backend gráfico;
- GPU/driver;
- escala HiDPI;
- filesystem local/remoto.

## Cambios de esquema

Cuando cambio JSON o paths:

1. defino versión;
2. leo el formato anterior;
3. creo backup;
4. migro de forma idempotente;
5. no sobrescribo datos válidos;
6. agrego pruebas;
7. documento rollback.