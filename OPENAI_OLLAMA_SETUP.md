# Soporte para OpenAI y Ollama

El sistema ahora soporta tanto OpenAI como Ollama localmente con configuración flexible.

## Configuración para OpenAI

Copia `config.openai.example.yml` a `config.yml`:

```yaml
provider: "openai"
base_url: "api.openai.com"
use_https: true
endpoint: "v1/chat/completions"
models_endpoint: "v1/models"
token: "sk-your-openai-api-key-here"
model: "gpt-3.5-turbo"
temperature: "1"
log_level: "info"
```

**Notas:**

- `token`: Requerido para OpenAI (tu API key)
- `use_https`: true
- `base_url`: api.openai.com
- Modelos disponibles: gpt-3.5-turbo, gpt-4, etc.

## Configuración para Ollama

Copia `config.ollama.example.yml` a `config.yml`:

```yaml
provider: "ollama"
base_url: "localhost:11434"
use_https: false
endpoint: "api/chat"
models_endpoint: "api/tags"
token: ""
model: "llama2"
temperature: "1"
log_level: "info"
```

**Notas:**

- `token`: No requerido (dejar vacío)
- `use_https`: false
- `base_url`: Cambiar según donde corra Ollama (localhost:11434 es el default)
- Modelos disponibles: llama2, mistral, neural-chat, etc.

## Usar Ollama en otro puerto

Si Ollama corre en otro puerto, solo cambia `base_url`:

```yaml
base_url: "localhost:8000" # o tu host/puerto
use_https: false
```

## Usar Ollama remoto

Para conectar a Ollama en otra máquina:

```yaml
base_url: "192.168.1.100:11434"
use_https: false
```

O con HTTPS si está configurado:

```yaml
base_url: "ollama.example.com:11434"
use_https: true
```

## Instalar modelos en Ollama

Si aún no tienes un modelo en Ollama:

```bash
ollama pull llama2
# o
ollama pull mistral
```

## Verificar modelos disponibles

Desde terminal:

```bash
ollama list
```

O a través de la API:

```bash
curl http://localhost:11434/api/tags
```

## Cambios principales en el código

1. **Nuevo campo `provider`**: "openai" u "ollama"
2. **Nuevo campo `use_https`**: true para HTTPS, false para HTTP
3. **Token opcional**: Solo se agrega header Bearer para OpenAI
4. **URL flexible**: Construida con protocolo, host y puerto
5. **Respuestas compatibles**: Ambos usan el formato chat/completions similar
