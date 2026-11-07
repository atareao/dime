# Proposal

## Why

dime sólo acepta la pregunta como argumento (\-q/posicional). Eso impide usarlo en pipelines Unix (\ echo ... | dime, dime < fichero.md, git diff | dime "resume estos cambios"), muy comunes en scripts y flujos de trabajo de terminal.

## What Changes

- Cuando no se aporta \-q/posicional y stdin no es un TTY, leer la pregunta desde stdin.
- Admitir \- como valor de la pregunta para forzar explícitamente la lectura desde stdin.
- Cuando se aporte pregunta Y stdin no sea un TTY con datos, usar el contenido de stdin como **contexto** añadido al mensaje de usuario.
- Si stdin no es un TTY, no aporta datos (EOF inmediato) y no hay pregunta, mostrar un error legible y salir con código distinto de cero.
- El comportamiento interactivo actual (TTY con \-q/posicional) no cambia.

## Capabilities

### New Capabilities
<!-- Ninguna capacidad nueva. -->

### Modified Capabilities
- `chat`: resolución de la pregunta y del contexto a partir de stdin.

## Impact
- `src/main.rs` (resolución de la entrada y mensajes de error), `src/ia.rs` (composición del mensaje de usuario con contexto), README.md.
- Sin dependencias nuevas.