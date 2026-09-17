<!-- Fuente única de instrucciones: AGENTS.md (Codex la lee directo, Claude la importa). Acá va SOLO lo exclusivo de Claude. -->
@AGENTS.md

## GSD Workflow Enforcement (solo Claude)

Esta sección no va en AGENTS.md: Codex ejecuta tareas ya planificadas por wrapper y no abre GSD.

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
