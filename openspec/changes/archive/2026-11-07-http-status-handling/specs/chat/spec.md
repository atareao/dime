# chat

## ADDED Requirements

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
