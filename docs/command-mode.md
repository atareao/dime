# Feature: modo comando y ejecución segura

## Qué es

Modo comando del CLI: `--cmd <objetivo>` pide al modelo **únicamente** el comando de shell que cumple el objetivo y lo imprime limpio (sin fences de Markdown ni explicaciones). Opcionalmente, `--exec` lo ejecuta con `sh -c` tras una confirmación interactiva y `--yes` evita el prompt. Corresponde a la **Fase 6** de `PLAN-003` (cambio OpenSpec `command-mode`, PR #35 squash → `development`). Añade la spec `command` (5 requisitos). Sin dependencias nuevas: usa `std::process::Command` y `std::io::IsTerminal`.

## Ficheros implicados

- `src/main.rs` — flags `--cmd`, `--exec` y `--yes` en `clap`; `run_command_mode` (flujo completo), `should_execute` (decisión pura), `is_confirmation`/`confirm_execution` (prompt), `execute_command` (`sh -c`), `exit_code` (propagación) y la rama temprana de `--cmd` en `main`.
- `src/ia.rs` — `COMMAND_SYSTEM_ROLE` (rol de sistema que pide sólo el comando), `clean_command` y sus ayudantes (`strip_fence_language`, `strip_wrapping_backticks`, `FENCE_LANGUAGES`); nueva variante `IABotError::Execution`.
- `tests/command_mode.rs` — 4 tests de integración sobre el binario real con un mock HTTP local y `stdin` sin TTY.
- `README.md` — uso de `--cmd`/`--exec`/`--yes` y advertencias de seguridad (incluida *prompt injection*).
- Spec: `openspec/specs/command` (5 requisitos).

## Cómo funciona

- **Petición (`run_command_mode`):** el rol de sistema por defecto es `COMMAND_SYSTEM_ROLE`, que exige responder sólo con el comando. `--instructions`/`--system` sigue pudiendo sobreescribirlo. La pregunta es el objetivo de `--cmd`; los adjuntos `--file`/`--image` se siguen admitiendo. En este modo `stdin` **no** se usa como contexto (se reserva para la confirmación de `--exec`).
- **Limpieza (`clean_command`, función pura):**
  - Si hay un fence multilínea (```` ```bash ```` + salto + comando + ```` ``` ````), se descarta la línea del lenguaje, el fence de cierre y la prosa previa; se conservan los saltos de línea internos del comando.
  - En un fence de una línea (```` ```bash touch /tmp/x``` ````) se descarta el primer token sólo si es un lenguaje conocido (`bash`, `sh`, `zsh`, `shell`, `console`, `fish`) **y** el resto no empieza por `-`, `./` ni `/`; así `bash -c "echo hi"` y `sh ./script.sh` quedan intactos.
  - Un fence que sólo contiene el lenguaje devuelve cadena vacía, lo que produce el error «el modelo no devolvió ningún comando» y evita abrir un shell interactivo con `--exec --yes`.
  - Se quitan los backticks inline envolventes sólo si son un único backtick en cada extremo y no hay otro backtick dentro: `` `which ls` && `pwd` `` queda intacto.
  - Sin fences se devuelve el texto recortado. Siempre se aplica `trim` final.
- **Impresión:** el comando limpio se imprime siempre en `stdout` antes de cualquier posible ejecución. Si queda vacío, se informa por `stderr` y se sale con código 1. `--json` no se aplica en este modo.
- **Decisión (`should_execute(exec, yes, stdin_is_tty, confirmed)`):** literalmente `exec && (yes || (stdin_is_tty && confirmed))`. Se testean las 16 combinaciones. Sin `--exec` nunca devuelve `true`.
- **Confirmación (`confirm_execution`/`is_confirmation`):** se escribe `¿Ejecutar \`<comando>\`? [y/N]` por `stderr` y se lee una línea de `stdin`. Sólo `y`, `yes`, `s`, `si` y `sí` (sin distinguir mayúsculas ni espacios) confirman; vacío, `n`, `no` o cualquier otra respuesta **no** ejecutan.
- **Ejecución (`execute_command`):** `sh -c <comando>` heredando `stdout`/`stderr`. Un fallo al lanzar el shell es `IABotError::Execution` (nunca *panic*).
- **Código de salida (`exit_code`):** se propaga el código del hijo. Si terminó por señal, en Unix se usa `128 + señal`; fuera de Unix, *fallback* `1`. Sin `--exec` el proceso termina con `0`.

## Configuración

- No requiere variables ni claves nuevas: reutiliza `provider`, `model`, `base_url`, `use_https` y el token de `dime.yml` (o los overrides `--provider`/`--model`).
- `--instructions`/`--system` sustituye el rol `COMMAND_SYSTEM_ROLE` si se quiere afinar el estilo del comando.
- `--cmd` no lee `stdin` como contexto.

## Cómo habilitar / usar

No hay activación aparte: el modo comando se habilita con el flag `--cmd`.

```bash
# Sólo imprime el comando: NUNCA lo ejecuta
dime --cmd "listar los .log más grandes"

# Lo imprime y pide confirmación interactiva antes de ejecutarlo
dime --cmd "listar los .log más grandes" --exec

# Lo imprime y lo ejecuta sin preguntar (para scripts o stdin no-TTY)
dime --cmd "listar los .log más grandes" --exec --yes

# Encadenar con contexto adjunto (ojo con la prompt injection)
dime --cmd "resumir los ficheros modificados" --file diff.txt --exec
```

En un `stdin` que **no** es TTY, `--exec` sin `--yes` aborta con código distinto de cero y un mensaje que sugiere `--yes`; no se ejecuta nada.

## Seguridad

Ejecutar comandos generados por un modelo es peligroso. `dime` aplica estas **seis barreras**:

1. **Opt-in explícito:** sin `--exec` **nunca** se ejecuta nada; sólo se imprime el comando.
2. **Mostrar antes de ejecutar:** el comando limpio se imprime siempre en `stdout` antes de cualquier ejecución, para que el usuario lo revise.
3. **Confirmación con default negativo:** el prompt es `[y/N]`; vacío, `n`, `no` o cualquier respuesta no afirmativa no ejecutan.
4. **No-TTY sin `--yes` aborta:** si `stdin` no es un TTY y no se pasa `--yes`, el proceso termina con código distinto de cero en lugar de ejecutar a ciegas.
5. **Flags coherentes:** `--exec` y `--yes` **requieren** `--cmd`; `--cmd` es incompatible con la pregunta (`-q`/posicional), `--list-models` y `--completions`.
6. **Lo mostrado es lo ejecutado:** `clean_command` elimina los backticks envolventes (que `sh -c` interpretaría como sustitución de comando) y convierte un fence sólo-lenguaje en cadena vacía, evitando abrir un shell interactivo con `--exec --yes`.

> ⚠️ **Prompt injection:** combinar `--file`/`--image` con `--exec --yes` permite que el contenido adjunto incluya instrucciones que el modelo traduzca en un comando, y `dime` lo ejecutará **sin preguntar**. No uses `--exec --yes` con contenido no confiable (diffs, páginas web, documentos de terceros); revisa siempre el comando impreso.

> **Sin sandboxing:** el comando se ejecuta con `sh -c` con los permisos del usuario. `dime` no aísla el proceso; la responsabilidad de lo que se ejecuta es del usuario.

## Decisiones

- Rol de sistema específico (`COMMAND_SYSTEM_ROLE`) en vez de instrucciones ad-hoc: pide una sola línea de comando y reduce explicaciones.
- Limpieza basada en fences y guardas explícitas (`clean_command`) en lugar de expresiones regulares complejas: suficiente, predecible y testeable.
- `should_execute` como función pura con tabla de verdad de 16 casos: permite cubrir TTY/no-TTY, confirmado/rechazado y `--yes` sin un TTY real.
- Ejecutar con `sh -c` (no `$SHELL`) por previsibilidad y portabilidad.
- Propagar el código del hijo (`128 + señal` en Unix) para que un fallo del comando nunca se confunda con éxito.
- Errores tipados (`IABotError::Execution`); nunca *panic*.

## Follow-ups

- **Sandboxing de `--exec`** (Non-Goal de esta fase): valorar límites de tiempo, entorno restringido o ejecución en un directorio controlado.
- El heurístico de `clean_command` para fences de una línea es deliberadamente conservador; revisar si aparecen casos nuevos de modelos que antepongan texto o lenguaje al comando.
- ¿Ejecutar con `$SHELL` en vez de `sh -c`? Decisión actual: `sh -c`; revisable.
- Valorar un historial de comandos o un modo «dry-run» explícito (hoy `--cmd` sin `--exec` ya cumple ese papel).
- Añadir cota de tamaño a los adjuntos usados con `--cmd` (hoy sin límite, igual que en el modo pregunta).
