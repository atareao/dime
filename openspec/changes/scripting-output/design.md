# Design

## Context

Ver proposal.md y plans/PLAN-003.md (Fase 3). Hoy IABot::ask devuelve un String con el contenido y main lo imprime con println!

## Goals / Non-Goals

- Goal: ofrecer una salida apta para scripts (--json) y garantizar la ausencia de ANSI.
- Non-Goal: introducir color en esta fase; la garantía de --no-color es forward-compatible.
- Non-Goal: streaming de la respuesta.

## Decisions

### D1. ask devuelve contenido + metadatos

Se introduce un tipo de respuesta (p. ej. ChatResponse { content: String, model: Option<String>, usage: Option<serde_json::Value> }). main decide cómo imprimirlo.

- Alternativa: mantener String y una segunda llamada para metadatos. Descartada: duplicaría la petición.

### D2. Serialización con serde_json

El modo --json construye el objeto con content, model y usage y lo serializa con serde_json (ya presente vía reqwest/DTO). usage usa serde_json::Value para preservar el objeto crudo del proveedor; si falta, null.

### D3. Errores en modo --json

Los errores se escriben en stderr como texto legible y el proceso termina con código distinto de cero. stdout queda sin JSON parcial.

### D4. Control de ANSI

La salida actual no emite ANSI. --no-color se acepta como flag y la implementación nunca emite ANSI cuando stdout no es un TTY o cuando se pasa --no, detectado con std::io::IsTerminal.

## Risks / Trade-offs

- [Cambiar el tipo de retorno de ask es un cambio interno disruptivo] → Actualizar todos los llamadores y tests en el mismo PR.
- [usage varía entre proveedores] → Se trata como Value opaco; el contrato sólo garantiza el campo y su ausencia como null.

## Migration Plan

1. Tipo de respuesta con metadatos.
2. Flag --json y serialización.
3. Flag --no-color y detección de TTY.
4. Tests, fmt/clippy, PR a development.

Rollback: revert del PR; la salida por defecto no cambia.

## Open Questions

- ¿Debe --json combinarse con --list-models? Decisión: fuera de alcance en esta fase.