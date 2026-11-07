## ADDED Requirements

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
