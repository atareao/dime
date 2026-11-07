# Design

## Context

Ver `proposal.md` y `plans/PLAN-003.md` (Fase 5). `src/ia.rs` construye el cuerpo de la petición (`build_body`) con DTOs `serde`; `src/main.rs` parsea `clap` y resuelve la entrada. Se añaden `arboard` (portapapeles) y `base64`.

## Goals / Non-Goals

- Goal: `--file`, `--image` (multimodal) y `--copy`.
- Non-Goal: imágenes por URL remota; `--image` acepta sólo rutas locales.
- Non-Goal: otros tipos de adjunto (audio, PDF).
- Non-Goal: instalar/colgar la imagen en el portapapeles; `--copy` copia texto.

## Decisions

### D1. `--file`: contenido como texto

Cada fichero se lee como texto (`fs::read_to_string`) y se añade al mensaje de usuario con una cabecera con su ruta, tras el contexto de `stdin`. Un fichero no UTF-8 o ilegible produce un error legible.

- Alternativa: adjuntos estructurados por fichero. Descartada: el texto plano es universal en ambos proveedores.

### D2. `--image`: multimodal

Los bytes se codifican en base64 (crate `base64`); el MIME se deduce de la extensión (`png`, `jpg`/`jpeg`, `gif`, `webp`); una extensión no reconocida produce un error legible.

- `openai`: `content` del mensaje `user` pasa a array `[{type:"text",text:...},{type:"image_url",image_url:{url:"data:<mime>;base64,<datos>"}}]`.
- `ollama`: `message.images = [<base64>...]`, con `content` como texto.

### D3. Compatibilidad de la forma del mensaje

Sin imágenes, `content` sigue siendo una cadena (no rompe peticiones ni tests existentes). Con imágenes, se usa el array de content parts.

### D4. `--copy` con `arboard`

Se copia tras imprimir. Es *best-effort*: si el portapapeles falla (p. ej. entorno headless), se advierte por `stderr` y se continúa con código 0. La operación se aísla en una función `copy_to_clipboard(text) -> Result<(), ...>` para poder testear el camino de error.

### D5. Errores

Se añade una variante tipada (p. ej. `IABotError::Attachment`) con mensaje legible; nunca se entra en *panic*.

## Risks / Trade-offs

- [Portapapeles no disponible en CI/headless] → Comportamiento *best-effort*; no rompe el comando; se testea el camino de error.
- [Sin cota de tamaño para ficheros/imágenes] → Fuera de alcance; se documenta.
- [Cambiar `content` a array afecta a `openai`] → Se mantiene cadena cuando no hay imágenes.

## Migration Plan

1. `--file`. 2. `--image`. 3. `--copy`. 4. Tests, README, `fmt`/`clippy`, PR a `development`.

Rollback: revert del PR; sin adjuntos el comportamiento no cambia.

## Open Questions

- ¿`--image` debe aceptar URLs? Decisión: no en esta fase.
