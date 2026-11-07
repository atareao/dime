# output

## ADDED Requirements

### Requirement: Salida JSON

Con `--json`, el sistema DEBE escribir en `stdout` exactamente un objeto JSON válido con los campos `content` (cadena con la respuesta), `model` (cadena con el modelo usado) y `usage` (el objeto de uso del proveedor, o `null` si no se aporta). `model` es SIEMPRE una cadena: si el proveedor no lo informa, DEBE usarse el modelo configurado. No DEBE escribir ningún otro texto en `stdout`.

#### Scenario: Respuesta con uso

- Given una respuesta con contenido `Hola`, modelo `gpt-4o` y un objeto `usage`
- When se ejecuta con `--json`
- Then `stdout` es un JSON válido con `content = "Hola"`, `model = "gpt-4o"` y `usage` con los datos del proveedor

#### Scenario: Respuesta sin uso

- Given una respuesta sin campo `usage`
- When se ejecuta con `--json`
- Then el JSON tiene `usage` igual a `null`

#### Scenario: Respuesta sin modelo

- Given una respuesta del proveedor sin campo `model` y una configuración con `model: mi-modelo`
- When se ejecuta con `--json`
- Then el JSON tiene `model = "mi-modelo"`

#### Scenario: Error en modo JSON

- Given un error de API
- When se ejecuta con `--json`
- Then `stdout` no contiene JSON y `stderr` muestra un mensaje legible con código de salida distinto de cero

### Requirement: Control de color y logs

La respuesta se escribe en `stdout` y NUNCA DEBE contener secuencias de escape ANSI. Los mensajes de log se escriben en `stderr` y SÓLO PUEDEN contener códigos ANSI cuando su destino (`stderr`) es un TTY y no se ha pasado `--no-color`.

#### Scenario: `--no-color`

- Given la ejecución con `--no-color`
- When se ejecuta `dime`
- Then ni `stdout` ni `stderr` contienen códigos ANSI

#### Scenario: Salida canalizada

- Given `stdout` redirigido a un fichero o a una tubería
- When se ejecuta `dime`
- Then la salida no contiene códigos ANSI

#### Scenario: Logs fuera de la salida estándar

- Given `log_level` en `info` o `debug`
- When se ejecuta `dime`
- Then los mensajes de log se escriben en `stderr` y `stdout` sólo contiene la respuesta
