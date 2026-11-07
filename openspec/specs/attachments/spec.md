# attachments Specification

## Purpose
TBD - created by archiving change context-input. Update Purpose after archive.

## Requirements

### Requirement: Contexto desde ficheros

Con `--file <ruta>` (repetible), el sistema DEBE leer cada fichero y añadir su contenido como contexto al mensaje de usuario, respetando el orden en que se indican. Si un fichero no existe o no puede leerse como texto, DEBE mostrar un error legible y terminar con código distinto de cero, sin realizar la petición.

#### Scenario: Un fichero como contexto

- Given la pregunta `resume` y `--file README.md`
- When se construye la petición
- Then el mensaje de usuario contiene la pregunta y el contenido de `README.md`

#### Scenario: Varios ficheros en orden

- Given `--file a.txt --file b.txt`
- When se construye la petición
- Then el contexto incluye `a.txt` antes que `b.txt`

#### Scenario: Fichero inexistente

- Given `--file no-existe.txt`
- When se ejecuta `dime`
- Then se muestra un error legible y el proceso termina con código distinto de cero, sin realizar la petición

### Requirement: Imágenes multimodales

Con `--image <ruta>` (repetible), el sistema DEBE adjuntar la imagen codificada en base64. Con proveedor `openai`, el mensaje `user` DEBE usar el formato de content parts con cada imagen como data URL (`{"type":"image_url","image_url":{"url":"data:<mime>;base64,<datos>"}}`). Con proveedor `ollama`, cada imagen DEBE enviarse en el campo `images` del mensaje como base64, manteniendo `content` como texto. Si una imagen no existe o no puede leerse, DEBE mostrar un error legible y terminar con código distinto de cero.

#### Scenario: Imagen para OpenAI

- Given proveedor `openai` y `--image foto.png`
- When se construye la petición
- Then `messages[user].content` es un array con una parte de texto y una parte `image_url` cuyo `url` empieza por `data:image/png;base64,`

#### Scenario: Imagen para Ollama

- Given proveedor `ollama` y `--image foto.png`
- When se construye la petición
- Then `messages[user]` incluye `images` con la imagen en base64 y `content` sigue siendo una cadena de texto

#### Scenario: Imagen inexistente

- Given `--image no-existe.png`
- When se ejecuta `dime`
- Then se muestra un error legible y el proceso termina con código distinto de cero
