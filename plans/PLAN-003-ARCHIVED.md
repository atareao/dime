# PLAN-003 — Features del CLI `dime`

## Objetivo

Añadir capacidades de uso avanzado al CLI `dime`: overrides por línea de comandos, entrada por stdin/pipes, salida para scripting, completions, contexto (ficheros/imágenes/clipboard), modo comando y tests de integración con servidor HTTP mock.

## Estado de partida

- Versión `0.3.0`; specs `config`, `chat` y `models`; 49 tests; CI en `main` y `development`.
- El CLI solo tiene `-i/--instructions`, `-q/--question`/posicional y `--list-models`.

## Estrategia

Un plan único con **7 fases**. Cada fase es un cambio OpenSpec propio y un **PR independiente** (gitflow `feature/*` → `development` → `main`), ejecutadas en orden. Cada fase añade sus tests y su documentación.

## Fases

### Fase 1 — Overrides por CLI (cambio `cli-overrides`) — ✅ Completada (PR #29 squash → `development`)

Flags que pisan `dime.yml` **solo para esa invocación**, sin modificar el fichero: `--model`, `--temperature`, `--provider` y `--system` (alias/equivalente de `-i/--instructions`).
- Specs: `config` (los overrides no se persisten), `chat` (usan los valores efectivos).
- Criterio: con overrides, la petición usa esos valores y el fichero queda intacto.

### Fase 2 — Entrada por stdin/pipes (cambio `stdin-input`) — ✅ Completada (PR #26 → `development`)

Si no se aporta pregunta y `stdin` no es un TTY, leer de `stdin`; admitir `-` como marcador explícito. Permitir componer `git diff | dime "resume estos cambios"` (instrucción + contexto) y `dime "explica" < file.md`.
- Specs: `chat`/`config` (resolución de la pregunta y del contexto).
- Criterio: la entrada por stdin se usa como pregunta o contexto según el caso.

### Fase 3 — Salida para scripting (cambio `scripting-output`) — ✅ Completada (PR #27 squash → `development`)

`--json` (respuesta + metadatos: modelo y uso si el proveedor lo aporta) y `--no-color`; color automático solo si la salida es un TTY.
- Spec nueva: `output`.
- Criterio: `--json` emite JSON válido; `--no-color` no emite códigos ANSI.

### Fase 4 — Shell completions (cambio `shell-completions`) — ✅ Completada (PR #31 squash → `development`)

Con `clap_complete`: `--completions <bash|zsh|fish>` imprime el script de completado.
- Spec nueva: `completions`.
- Criterio: `dime --completions bash` imprime un script válido.

### Fase 5 — Contexto: ficheros, imágenes y clipboard (cambio `context-input`) — ✅ Completada (PR #33 squash → `development`)

`--file <ruta>` (repetible), `--image <ruta>` (repetible; multimodal) y `--copy` (copiar la respuesta al portapapeles). Imágenes: OpenAI (content parts con data URL base64) y Ollama (`images`).
- Specs: nuevas `attachments` y `clipboard`; modifica `chat` (mensajes con contexto) y `config`.
- Criterio: `dime --file README.md "resume"` incluye el contenido; `--image` envía la imagen; `--copy` copia la respuesta.
- Dependencias: `arboard` (clipboard) y `base64`.
- Limitación conocida: en X11 sin *clipboard manager* el contenido no persiste tras salir el proceso.

### Fase 6 — Modo comando (cambio `command-mode`) — ✅ Completada (PR #35 squash → `development`)

`--cmd "<objetivo>"` pide únicamente el comando (sin fences Markdown); `--exec` lo ejecuta **con confirmación**; `--yes` evita el prompt. Seguridad: por defecto solo se imprime; **nunca** se ejecuta sin `--exec`.
- Spec nueva: `command` (5 requisitos).
- Criterio: `--cmd` devuelve el comando limpio; `--exec` pide confirmación; sin `--exec` no ejecuta nada.
- Implementado: `clean_command` (fences multilínea/de una línea y backticks envolventes), `should_execute` (tabla de 16 casos), `IABotError::Execution`, ejecución con `sh -c` y propagación del código de salida (`128 + señal` en Unix); 4 tests de integración en `tests/command_mode.rs`.
- Riesgos: ejecución de shell; requiere una spec explícita de seguridad.

### Fase 7 — Tests de integración HTTP (cambio `http-mock-tests`) — ✅ Completada (PR #37 squash → `development`)

`wiremock` como dependencia de desarrollo para cubrir `ask` y `list_models` contra un servidor local: cabecera de autorización, cuerpos, respuestas 4xx/5xx y formas inesperadas.
- Criterio: `cargo test` cubre la ruta real de red (hoy sin test).
- Implementado: 11 tests en `tests/http_mock.rs` que ejecutan el binario contra un mock con `dime.yml` temporal (`base_url` al mock, `use_https: false`): chat (método, ruta, `Authorization`, cuerpo y respuesta), sin token, forma del cuerpo ollama, `--list-models` (éxito y 401), errores 401/500/cuerpo sin `choices`/no JSON (error legible, sin *panic*, petición anclada con `verify()`) y comprobación de que `wiremock` sólo está en `[dev-dependencies]`. Spec nueva: `http-testing` (4 requisitos). Sin cambios en `src/`.

## Criterios de aceptación globales

- [x] Cada fase: cambio OpenSpec aprobado antes del código, tests, `fmt` y `clippy` verdes, PR a `development`. *(Fases 1–7 completadas)*
- [x] Sin regresiones en las specs existentes (`config`, `chat`, `models`). *(verificado tras las Fases 1–7)*
- [x] Documentación en `docs/` y `HANDOFF.md` actualizado al cerrar cada fase. *(cerrada para las Fases 1–7)*

## Riesgos

- La Fase 6 (ejecución de comandos) tiene la mayor superficie: exige una spec explícita de seguridad.
- Aumento del número de flags: mantener la ayuda de `clap` clara y los conflictos bien definidos.

## Versionado

**`0.5.0`** al cerrar el plan (Fase 7 incluida). Las Fases 1–3 fijaron `0.4.0`; las Fases 4–6 añadieron features sin re-bumpear, por lo que el cierre aplica un **minor** (`0.4.0` → `0.5.0`, `vampus upgrade --minor`).
