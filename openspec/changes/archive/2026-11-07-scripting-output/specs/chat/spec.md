## MODIFIED Requirements

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