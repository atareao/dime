# Proposal

## Why

dime sólo imprime texto plano pensado para humanos, lo que dificulta encadenarlo con jq u otros consumidores en scripts. Además no existe forma de controlar los códigos ANSI en la salida canalizada.

## What Changes

- Nuevo flag --json: emite en stdout exactamente un objeto JSON válido con los campos content (respuesta), model (modelo usado) y usage (objeto de uso del proveedor, o null si no se aporta).
- Nuevo flag --no-color: garantiza que la salida no contiene secuencias de escape ANSI.
- El color se aplica automáticamente sólo cuando stdout es un TTY; al canalizar o redirigir nunca se emiten códigos ANSI.
- En modo --json, los errores se escriben en stderr como texto legible y el proceso termina con código distinto de cero; stdout no contiene JSON parcial.

## Capabilities

### New Capabilities
- output: salida estructurada (--json) y control de color para scripting.

### Modified Capabilities
- chat: la respuesta expone el modelo y el objeto de uso del proveedor.

## Impact
- src/main.rs (flags y composición de la salida), src/ia.rs (la consulta devuelve contenido y metadatos), README.md.
- Sin dependencias nuevas (serde_json ya está presente).