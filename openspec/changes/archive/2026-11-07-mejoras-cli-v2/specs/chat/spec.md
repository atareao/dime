## MODIFIED Requirements

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