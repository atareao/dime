# Proposal

## Why

`dime` sólo acepta la pregunta como texto. Para tareas reales (resumir código, analizar una captura) hay que pegar el contenido a mano, y no hay forma de copiar la respuesta. Falta adjuntar ficheros, adjuntar imágenes (multimodal) y copiar el resultado al portapapeles.

## What Changes

- `--file <ruta>` (repetible): añade el contenido de uno o más ficheros como contexto del mensaje de usuario.
- `--image <ruta>` (repetible, multimodal): adjunta imágenes; para `openai` como content parts con data URL base64 y para `ollama` en el campo `images`.
- `--copy`: copia la respuesta al portapapeles además de imprimirla.
- Errores legibles si un fichero o una imagen no existe o no puede leerse, sin *panic*.
- Nuevas dependencias: `arboard` (portapapeles) y `base64`.

## Capabilities

### New Capabilities
- `attachments`: adjuntar ficheros e imágenes como contexto de la petición.
- `clipboard`: copiar la respuesta al portapapeles.

### Modified Capabilities
- `chat`: el mensaje de usuario puede incluir contexto de ficheros e imágenes (content parts en `openai`, campo `images` en `ollama`).

## Impact
- `Cargo.toml` (`arboard`, `base64`), `src/main.rs` (flags y lectura de rutas), `src/ia.rs` (construcción del cuerpo con adjuntos), `README.md`.
