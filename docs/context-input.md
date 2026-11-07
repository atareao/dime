# Feature: contexto de entrada (ficheros, imágenes y clipboard)

## Qué es

Añade **contexto a la pregunta** desde el CLI: `--file <ruta>` (repetible) adjunta el contenido de ficheros como texto, `--image <ruta>` (repetible, multimodal) adjunta imágenes y `--copy` copia la respuesta al portapapeles. Corresponde a la **Fase 5** de `PLAN-003` (cambio OpenSpec `context-input`, PR #33 squash → `development`). Crea las specs `attachments` y `clipboard` y modifica `chat`.

## Ficheros implicados

- `src/main.rs` — flags `--file`/`--image`/`--copy` en `clap`; `compose_user_text` (pregunta + `stdin` + ficheros), `copy_to_clipboard`/`copy_to_clipboard_with`, `clipboard_text`, `report_copy_result` y el guardián del portapapeles en `main`.
- `src/ia.rs` — `compose_context`, `read_files`, `read_images`, `mime_from_path`, `openai_content`; `IABot::ask` recibe las imágenes y `build_body` compone el cuerpo por proveedor.
- `Cargo.toml` — nuevas dependencias `arboard` (portapapeles) y `base64` (codificación de imágenes).
- `README.md` — uso de `--file`, `--image` y `--copy`.
- Specs: `openspec/specs/attachments` (nueva), `openspec/specs/clipboard` (nueva) y `openspec/specs/chat` (mensaje de usuario con contexto).

## Cómo funciona

- **`--file` (`compose_context`/`read_files`/`compose_user_text`):** cada fichero se lee como texto UTF-8 y se añade al mensaje de usuario con una cabecera `## <ruta>`, separados por línea en blanco y en el orden de los flags. El bloque de ficheros va **después** del contexto de `stdin`. Un fichero inexistente o no UTF-8 produce `IABotError::Attachment` con mensaje legible, sin petición y con salida distinta de cero.
- **`--image` (`read_images`/`mime_from_path`):** los bytes se codifican en base64 y el MIME se deduce por extensión (`png`, `jpg`/`jpeg`, `gif`, `webp`, insensible a mayúsculas). Extensión no soportada o imagen ilegible → `IABotError::Attachment`.
- **Envío por proveedor (`openai_content`/`build_body`):** con `openai`, si hay imágenes el `content` del mensaje `user` es un array de content parts (una parte `text` + una `image_url` con `data:<mime>;base64,...` por imagen); **sin imágenes sigue siendo una cadena** (compatibilidad). Con `ollama`, las imágenes van en el campo `images` (base64) y `content` es texto.
- **`--copy` (`copy_to_clipboard`/`clipboard_text`):** tras imprimir, copia el campo `content` de la respuesta (también en `--json`, no el objeto completo). Es *best-effort*: si no hay portapapeles se avisa por `stderr` y el proceso termina con código 0. El handle de `arboard::Clipboard` se mantiene vivo hasta el final de `main` (en X11 sin *clipboard manager* el proceso es el dueño del contenido).

## Configuración

- No requiere variables ni claves nuevas; reutiliza el `provider` de `dime.yml` (o el override `--provider`) para elegir la forma del cuerpo.
- Sin cota de tamaño para ficheros/imágenes (ver follow-ups).

## Uso en CLI

```bash
# Adjuntar uno o varios ficheros como contexto
dime --file src/main.rs "resume el flujo"
dime --file src/main.rs --file Cargo.toml "revisa esta configuración"

# Adjuntar imagen multimodal
dime --image captura.png "describe qué muestra"
dime --image antes.png --image despues.png "explica las diferencias"

# Copiar la respuesta al portapapeles
dime --copy -q "escribe un correo de disculpa"

# Combinado: contexto de stdin + fichero + copia + JSON
git diff | dime --file notas.md --copy --json "resume los cambios" | jq -r '.content'
```

## Decisiones

- Ficheros como texto plano con cabecera `## <ruta>` (universal en ambos proveedores) en vez de adjuntos estructurados.
- El contexto de ficheros se añade **después** del de `stdin`, respetando el orden de los flags.
- MIME por extensión local (`--image` no acepta URLs remotas ni otros tipos como audio/PDF en esta fase).
- Mantener `content` como cadena sin imágenes para no romper peticiones ni tests existentes.
- `--copy` *best-effort* y aislado en `copy_to_clipboard_with` (inyección de la escritura) para testear el camino de error; el handle se devuelve y se mantiene vivo hasta el final de `main`.
- Errores de adjuntos tipados como `IABotError::Attachment`; nunca *panic*.

## Follow-ups

- Cota de tamaño al leer ficheros/imágenes (evitar consumo de memoria sin límite).
- LIMITACIÓN conocida: en X11 **sin** *clipboard manager* el contenido no persiste tras salir el proceso (limitación de X11); funciona en Wayland/macOS/Windows y en X11 con manager. Test de roundtrip real marcado `#[ignore]`.
- ¿`--image` debe aceptar URLs remotas? Decisión: no en esta fase.
- Valorar otros tipos de adjunto (audio, PDF) y pegar imágenes al portapapeles.
