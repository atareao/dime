# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 6). `src/main.rs` resuelve la entrada y renderiza la salida; `src/ia.rs` construye el cuerpo y trata la respuesta. La ejecución usa `std::process::Command` (sin dependencias nuevas). Existe `std::io::IsTerminal` para detectar TTY.

## Goals / Non-Goals

- Goal: modo comando con ejecución opt-in y segura.
- Non-Goal: sandboxing de la ejecución; el usuario es responsable de lo que ejecuta.
- Non-Goal: historial de comandos o deshacer.

## Decisions

### D1. `--cmd` y rol de sistema

`--cmd <objetivo>` se trata como una pregunta con un rol de sistema específico que pide responder sólo con el comando. La respuesta se limpia con una función pura `clean_command(text)` que elimina fences de Markdown (```bash ... ```) y espacios sobrantes.

- Alternativa: post-procesar con expresiones regulares complejas. Descartada: basta con recortar fences y `trim`.

### D2. Seguridad por defecto

Sin `--exec`, NUNCA se ejecuta: sólo se imprime. La ejecución es **opt-in** explícita.

### D3. Confirmación interactiva

Con `--exec`, se imprime el comando y se pide confirmación; la respuesta por defecto es **negativa**. La decisión se factoriza en una función pura `should_execute(exec, yes, stdin_is_tty, confirmed) -> bool` para poder testear todos los caminos sin un TTY real.

- Si `stdin` no es un TTY y no hay `--yes` → no se ejecuta y se termina con código distinto de cero.
- Alternativa: leer la confirmación de `/dev/tty`. Descartada en esta fase por simplicidad y portabilidad.

### D4. Ejecución y código de salida

El comando se ejecuta con `sh -c <comando>` (hereda la salida estándar). El estado de salida del proceso hijo se propaga como código de salida de `dime`.

### D5. Relación entre flags

`--exec` y `--yes` REQUIEREN `--cmd` (`requires = "cmd"`). `--cmd` es incompatible con la pregunta, `--list-models` y `--completions` (`conflicts_with_all`).

## Risks / Trade-offs

- [Ejecutar comandos generados por un modelo es peligroso] → Opt-in, confirmación con default negativo, nunca sin `--exec`, y el comando se muestra antes de ejecutar.
- [Confirmación en no-TTY] → Se exige `--yes` explícito; sin él se aborta.
- [Cambiar el código de salida puede afectar a scripts] → Es el comportamiento deseado para `--exec`; sin `--exec` el código no cambia.

## Migration Plan

1. `--cmd` + limpieza del comando. 2. `--exec`/`--yes` + confirmación. 3. Ejecución y propagación. 4. Tests, README, `fmt`/`clippy`, PR a `development`.

Rollback: revert del PR; sin los flags el comportamiento no cambia.

## Open Questions

- ¿Ejecutar con `sh -c` o con `$SHELL`? Decisión: `sh -c` (predecible); revisable.
