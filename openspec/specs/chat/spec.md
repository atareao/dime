# chat Specification

## Purpose
TBD - created by archiving change normalizacion-repo. Update Purpose after archive.

## Requirements

### Requirement: Construcción de la URL

El sistema DEBE construir la URL como `{protocolo}://{base_url}/{endpoint}`, con protocolo `https` si `use_https` es verdadero y `http` en caso contrario.

#### Scenario: HTTPS por defecto

- Given `base_url="api.openai.com"`, `endpoint="v1/chat/completions"` y `use_https=true`
- When se construye la pet. When se construye la petición
- Then la URL es `https://api.openai.com/v1/chat/completions`.

### Requirement: Petición para OpenAI

Con proveedor `openai`, la petición DEBE incluir los mensajes `system` (rol de sistema) y `user` (pregunta más contexto) más el modelo, y DEBE incluir la cabecera `Authorization: Bearer <token>` cuando el token no esté vacío. Cuando no se adjunten imágenes, el `content` del mensaje `user` DEBE ser una cadena de texto; cuando se adjunten imágenes, DEBE ser un array de content parts (texto e `image_url`).

#### Scenario: Cabecera de autorización

- Given `provider="openai"` y un token no vacío
- When se realiza la consulta
- Then la petición incluye `Authorization: Bearer <token>`

#### Scenario: Sin imágenes, content como cadena

- Given proveedor `openai` y ninguna imagen adjunta
- When se construye la petición
- Then `messages[user].content` es una cadena de texto

#### Scenario: Con imágenes, content como partes

- Given proveedor `openai` y una o más imágenes adjuntas
- When se construye la petición
- Then `messages[user].content` es un array de content parts con texto e `image_url`

### Requirement: Petición para Ollama

Con proveedor `ollama`, la petición DEBE incluir `options.num_ctx` con el valor de `num_ctx` y NO DEBE incluir la cabecera `Authorization`.

#### Scenario: num_ctx en options

- Given `provider="ollama"` y `num_ctx=32000`
- When se construye la petición
- Then el cuerpo contiene `options.num_ctx = 32000` y no hay cabecera `Authorization`

### Requirement: Temperatura

El sistema DEBE enviar la temperatura configurada (ya numérica) como número `f64`: en el nivel superior del cuerpo para `openai` y dentro de `options` para `ollama`.

#### Scenario: Temperatura válida

- Given `temperature=0.2`
- When se construye la petición
- Then la temperatura se envía como el número `0.2`

#### Scenario: Temperatura inválida

- Given un `temperature` con valor no numérico
- When se carga la configuración
- Then se produce un error legible antes de construir ninguna petición

### Requirement: Tratamiento de respuestas

Ante una respuesta con el formato de chat estándar, el sistema DEBE devolver el contenido del primer elemento de choices junto con el modelo (model) y el objeto de uso (usage) de la respuesta cuando estén presentes. Ante una respuesta con campo error, DEBE devolver el mensaje de error del proveedor. Ante cualquier otra forma inesperada, DEBE devolver un error tipado, sin panic.

#### Scenario: Respuesta correcta

- Given una respuesta con choices[0].message.content = "Hola" y model = "gpt-4o"
- When se procesa
- Then la consulta devuelve el contenido "Hola" y el modelo "gpt-4o"

#### Scenario: Respuesta de error del proveedor

- Given una respuesta con error.message = "invalid_api_key"
- When se procesa
- Then la consulta devuelve un error cuyo mensaje es "invalid_api_key"

#### Scenario: Respuesta inesperada

- Given una respuesta JSON sin choices ni error
- When se procesa
- Then la consulta devuelve un error tipado y el proceso no entra en panic

### Requirement: Entrada desde stdin

El sistema DEBE obtener la pregunta desde `stdin` cuando no se aporta `-q`/posicional y `stdin` no es un TTY. El valor `-` como pregunta DEBE forzar la lectura desde `stdin`. Cuando se aporte pregunta y `stdin` no sea un TTY y contenga datos, el contenido de `stdin` DEBE añadirse como contexto al mensaje de usuario. Si `stdin` no es un TTY, no contiene datos y no hay pregunta, el sistema DEBE mostrar un error legible y terminar con código distinto de cero, sin realizar ninguna petición.

#### Scenario: Pregunta sólo desde stdin

- Given que no se aporta `-q`/posicional y `stdin` recibe `hola` por tubería
- When se ejecuta `dime`
- Then la pregunta es `hola`

#### Scenario: Marcador `-`

- Given que `stdin` recibe `hola` por tubería
- When se ejecuta `dime -`
- Then la pregunta es `hola`

#### Scenario: Pregunta más contexto por stdin

- Given la pregunta `resume` y `stdin` con el contenido de un diff
- When se ejecuta `dime "resume"" con `stdin` canalizado
- Then el mensaje de usuario contiene la pregunta y el contexto de `stdin`

#### Scenario: stdin sin datos ni pregunta

- Given que `stdin` no es un TTY, no aporta datos y no hay pregunta
- When se ejecuta `dime`
- Then se muestra un error legible y el proceso termina con código distinto de cero

#### Scenario: Uso interactivo sin pregunta

- Given que `stdin` es un TTY y no se aporta pregunta
- When se ejecuta `dime`
- Then se muestra el error de uso y el proceso termina con código distinto de cero, sin leer de `stdin`

### Requirement: Valores efectivos con overrides

Cuando se aporta un override por CLI (`--model`, `--temperature` o `--provider`), la petición DEBE usar ese valor; en su defecto, el valor de la configuración. Los overrides NO DEBEN cambiar el rol de sistema, que sólo lo modifican `-i/--instructions` o su alias `--system`.

#### Scenario: Override de temperatura

- Given `temperature: 1.0` en el fichero y el flag `--temperature 0.2`
- When se construye la petición
- Then la temperatura enviada es `0.2`

#### Scenario: Override de modelo

- Given `model: gpt-3.5-turbo` en el fichero y el flag `--model gpt-4o`
- When se construye la petición
- Then el modelo enviado es `gpt-4o`

#### Scenario: Override de proveedor a ollama

- Given el proveedor configurado `openai` y el flag `--provider ollama`
- When se construye la petición
- Then el cuerpo contiene `options.num_ctx` y no incluye la cabecera `Authorization`

#### Scenario: Sin overrides

- Given que no se pasa ningún override
- When se construye la petición
- Then se usan los valores de `dime.yml`

### Requirement: El código de estado HTTP determina el resultado

Tras enviar la petición de chat, el sistema DEBE comprobar el código de estado HTTP antes de interpretar el cuerpo. Si el código NO es exitoso (`>= 400`), la operación DEBE fallar con un error tipado, incluso cuando el cuerpo tenga forma de respuesta válida: primero DEBE usarse el mensaje del proveedor si el cuerpo contiene el campo `error`, y en su defecto un error que incluya el código de estado. Nunca DEBE producirse un *panic*.

#### Scenario: 500 con cuerpo de chat válido

- Given un proveedor que responde `500 Internal Server Error` con un cuerpo que contiene `choices` válidas
- When se realiza la consulta
- Then la operación falla con un error tipado que menciona el código `500`
- And el contenido del cuerpo NO se devuelve como respuesta

#### Scenario: 401 con error del proveedor

- Given un proveedor que responde `401 Unauthorized` con un cuerpo que contiene `error`
- When se realiza la consulta
- Then la operación falla con el mensaje del proveedor

#### Scenario: 429 sin campo error

- Given un proveedor que responde `429 Too Many Requests` sin el campo `error` en el cuerpo
- When se realiza la consulta
- Then la operación falla con un error tipado que menciona el código `429`

#### Scenario: 200 sigue siendo éxito

- Given un proveedor que responde `200 OK` con una respuesta válida
- When se realiza la consulta
- Then se devuelve el contenido y no hay error
