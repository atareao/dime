# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 6). `src/main.rs` resuelve la entrada y renderiza la salida; `src/ia.rs` construye el cuerpo y trata la respuesta. La ejecución usa `std::process::Command` (sin dependencias nuevas). Existe `std::io::IsTerminal` para detectar TTY.

## Goals / Non-Goals

- Goal: modo comando con ejecución opt-in y segura.
- Non-Goal: sandboxing de la ejecución; el usuario es responsable de lo que ejecuta.
- Non-Goal: historial de comandos o deshacer.

## Decisions

### D1. `--cmd` y rol de sistema

`--cmd <objetivo>` se trata como una pregunta con un rol de sistema específico
que pide responder sólo con el comando. La respuesta se limpia con una función
pura `clean_command(text)`:

- **Fence multilínea** (```` ``` ```` + lenguaje + salto de línea + comando +
  ```` ``` ````): se descarta la línea del lenguaje y el fence de cierre; se
  conservan los saltos de línea internos del comando.
- **Fence de una línea** (```` ``` ```` + [lenguaje] + comando + ```` ``` ````):
  se descarta el primer token sólo si es un lenguaje conocido (`bash`, `sh`,
  `zsh`, `shell`, `console`, `fish`) y el resto no empieza por `-`, `./` ni `/`,
  para no mutilar `bash -c "echo hi"` ni `sh ./script.sh`. Un fence que sólo
  contiene el lenguaje devuelve cadena vacía, lo que provoca el error "el modelo
  no devolvió ningún comando" (evita abrir un shell interactivo con
  `--exec --yes`).
- **Backticks inline envolventes**: se quita el par sólo si es un único backtick
  en cada extremo y no hay ningún otro backtick dentro; así
  `` `which ls` && `pwd` `` queda intacto (recortarlo corrompería el comando).
- **Sin fences**: se devuelve el texto recortado tal cual.
- En todos los casos se aplica `trim` final.

Quitar los backticks envolventes es una medida de **seguridad**: el comando se
imprime y, con `--exec`, se pasa a `sh -c`, donde los backticks se interpretan
como sustitución de comando; lo mostrado debe ser exactamente lo ejecutado.

- Alternativa: post-procesar con expresiones regulares complejas. Descartada: la
  limpieza basada en fences y guardas explícitas es suficiente y testeable.

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
