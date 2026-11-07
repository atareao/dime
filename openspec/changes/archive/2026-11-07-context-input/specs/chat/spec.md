## MODIFIED Requirements

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
