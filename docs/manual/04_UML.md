# 04 — Diagramas UML

## Propósito

Yo uso UML para representar Borjes como producto, aplicación nativa y sistema ECS. Mermaid no implementa toda la notación UML estricta, por lo que adapto algunas vistas manteniendo su intención.

## Cobertura UML 2

| Tipo | Cobertura | Aplicación |
|---|---|---|
| casos de uso | incluida | navegación y personalización |
| clases | incluida | modelos y servicios principales |
| objetos | incluida | escena concreta |
| componentes | incluida | módulos ejecutables |
| paquetes | incluida | dependencias del código |
| estructura compuesta | incluida | interior de la escena/overlay |
| despliegue | incluida | binario, desktop y filesystem |
| actividad | incluida | navegación |
| estados | incluida | ciclo del overlay |
| secuencia | incluida | apertura y navegación |
| comunicación | incluida | mensajes entre sistemas ECS |
| visión general de interacción | incluida | flujo búsqueda/navegación |
| temporal | incluida | apertura/render/reposo |
| perfil | incluida | estereotipos de módulos |

## 1. Casos de uso

```mermaid
flowchart LR
    U[Usuario]
    OS[Escritorio Linux]
    FS[Filesystem]

    U --> A((Abrir u ocultar overlay))
    U --> B((Navegar carpetas))
    U --> C((Buscar globalmente))
    U --> D((Abrir carpeta externa))
    U --> E((Gestionar pins))
    U --> F((Aplicar lentes))
    U --> G((Explorar constelaciones))
    U --> H((Resetear personalización))
    A --> OS
    D --> OS
    B --> FS
    C --> FS
```

## 2. Clases/modelos

```mermaid
classDiagram
    class AppState {
      +currentRoot
      +currentPath
      +visible
      +mode
    }
    class FolderNode {
      +path
      +name
      +depth
      +weight
    }
    class NavigationState {
      +breadcrumbs
      +selected
      +enter(path)
      +back()
    }
    class SearchIndex {
      +search(query)
    }
    class PinStore {
      +list()
      +add(path)
      +remove(path)
    }
    class Lens {
      +id
      +matches(node)
      +weight(node)
    }
    class FrecencyStore {
      +record(path)
      +score(path)
    }
    class DesktopAdapter {
      +open(path)
      +focus()
      +toggle()
    }

    AppState --> NavigationState
    NavigationState --> FolderNode
    SearchIndex --> FolderNode
    PinStore --> FolderNode
    Lens --> FolderNode
    FrecencyStore --> FolderNode
    DesktopAdapter ..> AppState
```

## 3. Objetos en una escena

```mermaid
flowchart LR
    ROOT[rootNode<br/>path=~/Data<br/>weight=1.0]
    P[projectsNode<br/>selected=true<br/>frecency=0.8]
    S[studyNode<br/>selected=false<br/>frecency=0.4]
    PIN[pin<br/>path=~/Data/Proyectos]
    L[lens<br/>id=workspace]

    ROOT --> P
    ROOT --> S
    PIN --> P
    L -. destaca .-> P
```

## 4. Componentes

```mermaid
flowchart TB
    MAIN[Bootstrap Bevy]
    INPUT[Input/Shortcut]
    SCAN[Filesystem Scanner]
    NAV[Navigation]
    SEARCH[Search]
    DRILL[Drill/Overlay Scene]
    RENDER[Camera/Render/Animation]
    PINS[Pins]
    ICONS[Icons/Assets]
    LENS[Lenses]
    FREC[Frecency]
    DESKTOP[Desktop Integration]
    CONFIG[(XDG JSON/config)]

    MAIN --> INPUT
    MAIN --> SCAN
    MAIN --> NAV
    MAIN --> DRILL
    NAV --> SEARCH
    NAV --> DRILL
    DRILL --> RENDER
    PINS --> CONFIG
    LENS --> CONFIG
    FREC --> CONFIG
    ICONS --> DRILL
    DESKTOP --> INPUT
    DESKTOP --> NAV
```

## 5. Paquetes

```mermaid
flowchart LR
    APP[main/app]
    FS[scan/root]
    UI[drill/overlay/ui]
    NAV[navigation/search]
    PREF[pins/icons/lens/frecency]
    DESK[desktop]
    ASSET[assets]

    APP --> FS
    APP --> UI
    APP --> NAV
    APP --> PREF
    APP --> DESK
    UI --> ASSET
    NAV --> FS
    UI --> NAV
    PREF --> UI
```

## 6. Estructura compuesta del overlay

```mermaid
flowchart TB
    subgraph Overlay
      ROOT[Central Node]
      CHILD[Child Nodes]
      BREAD[Breadcrumb]
      HUD[Search / Lens / Pins HUD]
      CAMERA[Camera]
      INPUT[Input State]
      ANIM[Animation Systems]
      LAYOUT[Radial Layout]
    end

    INPUT --> ROOT
    ROOT --> CHILD
    ROOT --> BREAD
    HUD --> ROOT
    LAYOUT --> CHILD
    ANIM --> ROOT
    ANIM --> CHILD
    CAMERA --> ROOT
```

## 7. Actividad de navegación

```mermaid
flowchart TD
    A[Abrir overlay] --> B[Escanear/cargar carpeta actual]
    B --> C[Construir nodos]
    C --> D[Ordenar y limitar]
    D --> E[Calcular layout]
    E --> F[Renderizar]
    F --> G{Acción}
    G -- seleccionar carpeta --> H[Actualizar path y breadcrumb]
    H --> B
    G -- buscar --> I[Consultar índice]
    I --> J[Mostrar resultados]
    J --> H
    G -- abrir externa --> K[DesktopAdapter.open]
    G -- cerrar --> L[Ocultar overlay]
```

## 8. Máquina de estados del overlay

```mermaid
stateDiagram-v2
    [*] --> Starting
    Starting --> Hidden: recursos cargados
    Starting --> Failed: error de assets/gráficos
    Hidden --> Opening: señal/atajo
    Opening --> Visible: foco y escena lista
    Visible --> Navigating: entrada del usuario
    Navigating --> Visible: escena actualizada
    Visible --> Searching: abrir búsqueda
    Searching --> Visible: elegir/cancelar
    Visible --> Closing: cerrar
    Closing --> Hidden
    Hidden --> Stopping: terminar proceso
    Visible --> Stopping: terminar proceso
    Stopping --> [*]
```

## 9. Secuencia de apertura

```mermaid
sequenceDiagram
    actor U as Usuario
    participant L as Launcher/Desktop
    participant A as App Borjes
    participant F as Filesystem
    participant R as Render/ECS

    U->>L: atajo global
    L->>A: iniciar o toggle
    A->>F: leer raíz/carpeta actual
    F-->>A: entradas
    A->>R: spawn/update nodes
    R-->>A: frame listo
    A-->>U: overlay visible
```

## 10. Secuencia de búsqueda

```mermaid
sequenceDiagram
    actor U as Usuario
    participant UI as Search UI
    participant S as SearchIndex
    participant N as Navigation
    participant V as Scene

    U->>UI: escribir consulta
    UI->>S: search(query)
    S-->>UI: resultados con path
    U->>UI: seleccionar resultado
    UI->>N: navigate(path)
    N->>V: rebuild current level
    V-->>U: destino visible
```

## 11. Comunicación ECS

```mermaid
flowchart LR
    INPUT[Input system] -- NavigationEvent --> NAV[Navigation system]
    NAV -- CurrentPathChanged --> SCAN[Scan system]
    SCAN -- FolderNodes --> LAYOUT[Layout system]
    LAYOUT -- transforms --> RENDER[Render system]
    NAV -- VisitEvent --> FREC[Frecency system]
    PINUI[Pin UI] -- PinCommand --> PINS[Pin store]
    LENSUI[Lens UI] -- LensChanged --> FILTER[Filter system]
```

## 12. Visión general de interacción

```mermaid
flowchart TD
    I1[[Toggle overlay]] --> I2[[Cargar nivel]]
    I2 --> I3[[Renderizar mapa]]
    I3 --> D{Objetivo}
    D -- navegar --> I4[[Entrar/volver]]
    I4 --> I2
    D -- buscar --> I5[[Buscar global]]
    I5 --> I4
    D -- pin --> I6[[Persistir acceso]]
    I6 --> I3
    D -- abrir --> I7[[Delegar al escritorio]]
    D -- cerrar --> I8[[Ocultar y reducir actividad]]
```

## 13. Temporal: visible y oculto

```mermaid
sequenceDiagram
    participant U as Usuario
    participant A as App
    participant R as Render Loop
    participant OS as Desktop

    U->>OS: t0 atajo
    OS->>A: toggle visible
    A->>R: frames activos para apertura/animación
    Note over R: consumo mayor durante interacción
    U->>A: t1 cerrar
    A->>R: completar transición
    A->>R: modo reactivo/oculto
    Note over R: el consumo debe medirse y tender a casi cero
```

La implementación exacta de wake-up/polling depende de la rama fusionada; yo no documento como hecho un cambio que solo vive en otro PR.

## 14. Despliegue

```mermaid
flowchart TB
    subgraph Linux[Sesión Linux del usuario]
      BIN[Binario Borjes]
      ASSETS[Assets instalados]
      CFG[(XDG config)]
      DATA[(Frecency/pins)]
      DESKTOP[Desktop entry / atajo / service]
      FS[~/Data y filesystem]
      FM[Explorador de archivos]
    end

    DESKTOP --> BIN
    BIN --> ASSETS
    BIN --> CFG
    BIN --> DATA
    BIN --> FS
    BIN --> FM
    USER[Usuario] --> DESKTOP
```

## 15. Perfil UML de módulos

```mermaid
classDiagram
    class Scanner {
      <<filesystem-read-only>>
    }
    class Overlay {
      <<interactive-scene>>
      <<latency-sensitive>>
    }
    class PinStore {
      <<user-private-state>>
    }
    class FrecencyStore {
      <<behavioral-local-data>>
    }
    class DesktopAdapter {
      <<platform-specific>>
    }
    class LegacyUI {
      <<not-active-until-wired>>
    }
```

## Regla de mantenimiento

Yo actualizo estas vistas cuando cambia un módulo activo, evento ECS, estado, ruta XDG, dependencia de assets o integración del escritorio. El diagrama debe distinguir `main` de funciones que solo existen en PRs abiertos.