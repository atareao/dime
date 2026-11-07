# chat

## ADDED Requirements

### Requirement: Construcción de la URL

El sistema DEBE construir la URL como `{protocolo}://{base_url}/{endpoint}`, con protocolo `https` si `use_https` es verdadero y `http` en caso contrario.

#### Scenario: HTTPS por defecto

- Given `base_url="api.openai.com"`, `endpoint="v1/chat/completions"` y `use_https=true`
- When se construye la pet. When se construye la petición
- Then la URL es `https://api.openai.com/v1/chat/completions`.

### Requirement: Petición para OpenAI

Con proveedor `openai`, la petición DEBE incluir los mensajes `system` (rol de sistema) y `user` (pregunta) más el modelo, y DEBE incluir la cabecera `Authorization: Bearer <token>` cuando el token no esté vacío.

#### Scenario: Cabecera de autorización

- Given `provider="openai"` y un token no vacío
- When se realiza la consulta
- Then la petición incluye `Authorization: Bearer <token>`

### Requirement: Petición para Ollama

Con proveedor `ollama`, la petición DEBE incluir `options.num_ctx` con el valor de `num_ctx` y NO DEBE incluir la cabecera `Authorization`.

#### Scenario: num_ctx en options

- Given `provider="ollama"` y `num_ctx=32000`
- When se construye la petición
- Then el cuerpo contiene `options.num_ctx = 32000` y no hay cabecera `Authorization`

### Requirement: Temperatura

El sistema DEBE enviar la temperatura configurada como valor numérico: en el nivel superior para `openai` y dentro de `options` para `ollama`. Si la temperatura no es un número válido, el sistema DEBE fallar con un error legible.

#### Scenario: Temperatura válida

- Given `temperature="0.2"`
- When se construye la petición
- Then la temperatura se envía como el número `0.2`

#### Scenario: Temperatura inválida

- Given `temperature="alta"`
- When se intenta construir la petición
- Then se devuelve un error legible en lugar de un *panic`

### Requirement: Tratamiento de respuestas

Ante una respuesta con el formato de chat estándar, el sistema DEBE devolver el contenido del primer elemento de `choices`. Ante una respuesta con campo `error`, DEBE devolver el mensaje de error del proveedor. Ante cualquier otra forma inesperada, DEBE devolver un error tipado, sin *panic*.

#### Scenario: Respuesta correcta

- Given una respuesta con `choices[0].message.content = "Hola"`
- When se procesa
- Then `ask` devuelve "Hola"

#### Scenario: Respuesta de error del proveedor

- Given una respuesta con `error.message = "invalid_api_key"`
- When se procesa
- Then `ask` devuelve un error cuyo mensaje es `invalid_api_key`

#### Scenario: Respuesta inesperada

- Given una respuesta JSON sin `choices` ni `error`
- When se procesa
- Then `ask` devuelve un error tipado y el proceso no entra en *panic`