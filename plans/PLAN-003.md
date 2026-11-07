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

### Fase 6 — Modo comando (cambio `command-mode`)

`--cmd "<objetivo>"` pide únicamente el comando (sin fences Markdown); `--exec` lo ejecuta **con confirmación**; `--yes` evita el prompt. Seguridad: por defecto solo se imprime; **nunca** se ejecuta sin `--exec`.
- Spec nueva: `command`.
- Criterio: `--cmd` devuelve el comando limpio; `--exec` pide confirmación; sin `--exec` no ejecuta nada.
- Riesgos: ejecución de shell; requiere una spec explícita de seguridad.

### Fase 7 — Tests de integración HTTP (cambio `http-mock-tests`)

`wiremock` como dependencia de desarrollo para cubrir `ask` y `list_models` contra un servidor local: cabecera de autorización, cuerpos, respuestas 4xx/5xx y formas inesperadas.
- Criterio: `cargo test` cubre la ruta real de red (hoy sin test).

## Criterios de aceptación globales

- [ ] Cada fase: cambio OpenSpec aprobado antes del código, tests, `fmt` y `clippy` verdes, PR a `development`. *(Fases 1–5 cumplen el criterio; pendientes Fases 6–7)*
- [x] Sin regresiones en las specs existentes (`config`, `chat`, `models`). *(verificado tras las Fases 1–5)*
- [x] Documentación en `docs/` y `HANDOFF.md` actualizado al cerrar cada fase. *(cerrada para las Fases 1–5)*

## Riesgos

- La Fase 6 (ejecución de comandos) tiene la mayor superficie: exige una spec explícita de seguridad.
- Aumento del número de flags: mantener la ayuda de `clap` clara y los conflictos bien definidos.

## Versionado

**`0.4.0`** alcanzado al cerrar las **Fases 1–3** (Fases 2–3 se mergearon sin bumpear; el bump a `0.4.0` se hizo con la Fase 1). Las **Fases 4 y 5** se han mergeado **sin re-bumpear**: se mantiene `0.4.0`. Las Fases 6–7 en incrementos posteriores.
