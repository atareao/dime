use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::path::PathBuf;
use tokio::fs::read_to_string;
use tracing::{debug, error, info};

/// Errores tipados del cliente `IABot`.
///
/// Sustituye al antiguo `Box<dyn Error>` y a los múltiples `unwrap()` que
/// provocaban *panics* ante respuestas o ficheros inesperados.
#[derive(Debug, thiserror::Error)]
pub enum IABotError {
    /// El fichero de configuración no se pudo leer o no es YAML válido.
    #[error("Error con el fichero de configuración `dime.yml`: {0}")]
    Config(String),
    /// Fallo de red al contactar con el proveedor.
    #[error("Error de red al contactar con el proveedor: {0}")]
    Http(String),
    /// El proveedor devolvió un campo `error`.
    #[error("Error devuelto por el proveedor: {0}")]
    Api(String),
    /// La respuesta no tiene ni `choices` ni `error`.
    #[error("Respuesta inesperada del proveedor: no contiene `choices` ni `error`")]
    UnexpectedResponse,
    /// Un campo configurado como cadena no es un número válido.
    #[error("No se pudo interpretar `{field}` como número (valor: `{value}`)")]
    NumberParse { field: String, value: String },
}

/// Cliente de chat para proveedores compatibles con la API de OpenAI.
///
/// La implementación de [`fmt::Debug`] es manual para redactar el token y
/// evitar que aparezca en cualquier traza de log.
#[derive(Serialize, Deserialize)]
pub struct IABot {
    #[serde(default = "get_default_log_level")]
    log_level: String,
    #[serde(default = "get_default_system_role")]
    system_role: String,
    #[serde(default = "get_default_provider")]
    provider: String,
    #[serde(default = "get_default_base_url")]
    base_url: String,
    #[serde(default = "get_default_endpoint")]
    endpoint: String,
    #[serde(default = "get_default_token")]
    token: String,
    #[serde(default = "get_default_model")]
    model: String,
    // What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(default = "get_default_temperature")]
    temperature: String,
    #[serde(default = "get_default_num_ctx")]
    num_ctx: u32,
    #[serde(default = "get_default_use_https")]
    use_https: bool,
}

impl fmt::Debug for IABot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token = if self.token.is_empty() { "" } else { "***" };
        f.debug_struct("IABot")
            .field("log_level", &self.log_level)
            .field("system_role", &self.system_role)
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("endpoint", &self.endpoint)
            .field("token", &token)
            .field("model", &self.model)
            .field("temperature", &self.temperature)
            .field("num_ctx", &self.num_ctx)
            .field("use_https", &self.use_https)
            .finish()
    }
}

fn get_default_log_level() -> String {
    "info".to_string()
}

fn get_default_system_role() -> String {
    "Eres un asistente util y conciso. Responde en espanol claro.".to_string()
}

fn get_default_provider() -> String {
    "openai".to_string() // "openai" o "ollama"
}

fn get_default_base_url() -> String {
    "api.openai.com".to_string()
}

fn get_default_use_https() -> bool {
    true
}

fn get_default_endpoint() -> String {
    "v1/chat/completions".to_string()
}

fn get_default_token() -> String {
    "".to_string()
}

fn get_default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn get_default_temperature() -> String {
    "1".to_string()
}

fn get_default_num_ctx() -> u32 {
    32000
}

impl Default for IABot {
    fn default() -> Self {
        Self {
            log_level: get_default_log_level(),
            system_role: get_default_system_role(),
            provider: get_default_provider(),
            base_url: get_default_base_url(),
            endpoint: get_default_endpoint(),
            token: get_default_token(),
            model: get_default_model(),
            temperature: get_default_temperature(),
            num_ctx: get_default_num_ctx(),
            use_https: get_default_use_https(),
        }
    }
}

impl IABot {
    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    pub fn get_system_role(&self) -> &str {
        &self.system_role
    }

    /// Indica si el proveedor configurado es Ollama (comparación insensible a mayúsculas).
    fn is_ollama(&self) -> bool {
        self.provider.eq_ignore_ascii_case("ollama")
    }

    /// Valida la configuración antes de usarla.
    ///
    /// Falla cuando el proveedor no es Ollama y el token está vacío.
    pub fn validate(&self) -> Result<(), IABotError> {
        if !self.is_ollama() && self.token.is_empty() {
            return Err(IABotError::Config(
                "el token está vacío para un proveedor que lo requiere".to_string(),
            ));
        }
        Ok(())
    }

    /// Devuelve la cabecera `Authorization: Bearer <token>` cuando procede.
    ///
    /// Sólo se envía para el proveedor `openai` y con un token no vacío.
    fn auth_header(&self) -> Option<String> {
        if self.provider.eq_ignore_ascii_case("openai") && !self.token.is_empty() {
            Some(format!("Bearer {}", self.token))
        } else {
            None
        }
    }

    /// Construye la URL de la petición: `{protocolo}://{base_url}/{endpoint}`.
    fn build_url(&self) -> String {
        let protocol = if self.use_https { "https" } else { "http" };
        format!("{}://{}/{}", protocol, self.base_url, self.endpoint)
    }

    /// Construye el cuerpo JSON de la petición sin tocar la red.
    ///
    /// - `openai`: `temperature` en el nivel superior.
    /// - `ollama`: `temperature` y `num_ctx` dentro de `options`.
    fn build_body(&self, system: &str, question: &str) -> Result<String, IABotError> {
        let temperature: f64 =
            self.temperature
                .trim()
                .parse()
                .map_err(|_| IABotError::NumberParse {
                    field: "temperature".to_string(),
                    value: self.temperature.clone(),
                })?;

        let messages = json!([
            {"role": "system", "content": system},
            {"role": "user", "content": question}
        ]);

        let payload = if self.is_ollama() {
            json!({
                "model": self.model,
                "messages": messages,
                "options": {
                    "num_ctx": self.num_ctx,
                    "temperature": temperature,
                }
            })
        } else {
            json!({
                "model": self.model,
                "messages": messages,
                "temperature": temperature,
            })
        };

        serde_json::to_string(&payload).map_err(|e| IABotError::Http(e.to_string()))
    }

    /// Envía la pregunta al proveedor y devuelve su respuesta.
    pub async fn ask(&self, system: &str, question: &str) -> Result<String, IABotError> {
        let url = self.build_url();
        debug!("Url: {}", url);
        debug!("Provider: {}", self.provider);
        debug!("Question: {}", question);

        let body = self.build_body(system, question)?;

        let client = Client::new();
        let mut request_builder = client.post(&url).header("Content-Type", "application/json");

        // Agregar token solo si es OpenAI y el token no está vacío
        if let Some(header) = self.auth_header() {
            request_builder = request_builder.header("Authorization", header);
        }

        let response = request_builder.body(body).send().await.map_err(|e| {
            error!("{}", e);
            IABotError::Http(e.to_string())
        })?;

        let response_body = response
            .text()
            .await
            .map_err(|e| IABotError::Http(e.to_string()))?;
        debug!("Response length: {}", response_body.len());

        parse_response(&response_body)
    }

    /// Escribe en `file` una configuración con los valores por defecto.
    pub async fn write_default(file: &PathBuf) -> Result<(), IABotError> {
        let default = Self::default();
        let content =
            serde_yaml::to_string(&default).map_err(|e| IABotError::Config(e.to_string()))?;
        tokio::fs::write(file, content.as_bytes())
            .await
            .map_err(|e| IABotError::Config(e.to_string()))
    }

    /// Lee y deserializa un fichero `dime.yml`.
    ///
    /// Devuelve un [`IABotError::Config`] legible (mencionando `dime.yml`) en
    /// lugar de terminar el proceso.
    pub async fn read_content(file: &PathBuf) -> Result<IABot, IABotError> {
        info!("File to read: {:?}", file);
        let content = read_to_string(file)
            .await
            .map_err(|e| IABotError::Config(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| IABotError::Config(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: serde_json::Value,
}

/// Interpreta el cuerpo de una respuesta de chat.
///
/// Primero intenta el error del proveedor y después la respuesta estándar.
/// El campo `error` puede ser un objeto con `message` o una cadena.
/// Cualquier otra forma devuelve [`IABotError::UnexpectedResponse`], nunca un *panic*.
fn parse_response(body: &str) -> Result<String, IABotError> {
    if let Ok(api_error) = serde_json::from_str::<ApiErrorBody>(body) {
        if api_error.error.is_null() {
            // `error: null` no es un error real: se continúa al parseo normal.
        } else if let Some(message) = api_error.error.get("message").and_then(|m| m.as_str()) {
            return Err(IABotError::Api(message.to_string()));
        } else if let Some(message) = api_error.error.as_str() {
            return Err(IABotError::Api(message.to_string()));
        } else {
            return Err(IABotError::UnexpectedResponse);
        }
    }

    if let Ok(response) = serde_json::from_str::<ChatResponse>(body) {
        if let Some(choice) = response.choices.into_iter().next() {
            return Ok(choice.message.content);
        }
    }

    Err(IABotError::UnexpectedResponse)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot() -> IABot {
        IABot::default()
    }

    fn openai_bot() -> IABot {
        IABot {
            provider: "openai".to_string(),
            token: "secreto-super-secreto".to_string(),
            ..IABot::default()
        }
    }

    fn ollama_bot() -> IABot {
        IABot {
            provider: "ollama".to_string(),
            ..IABot::default()
        }
    }

    // --- chat spec: Construcción de la URL ---

    #[test]
    fn build_url_uses_https_by_default() {
        assert_eq!(
            bot().build_url(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn build_url_uses_http_when_use_https_false() {
        let mut b = bot();
        b.use_https = false;
        assert_eq!(b.build_url(), "http://api.openai.com/v1/chat/completions");
    }

    // --- chat spec: Petición para OpenAI ---

    #[test]
    fn build_body_openai_has_messages_and_temperature_at_top_level() {
        let body = openai_bot().build_body("sys", "hola").expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["model"], "gpt-3.5-turbo");
        assert_eq!(v["messages"][0]["role"], "system");
        assert_eq!(v["messages"][0]["content"], "sys");
        assert_eq!(v["messages"][1]["role"], "user");
        assert_eq!(v["messages"][1]["content"], "hola");
        assert_eq!(v["temperature"], serde_json::json!(1.0));
        assert!(v.get("options").is_none());
    }

    // --- chat spec: Petición para Ollama ---

    #[test]
    fn build_body_ollama_puts_num_ctx_and_temperature_in_options() {
        let body = ollama_bot().build_body("sys", "hola").expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["options"]["num_ctx"], 32000);
        assert_eq!(v["options"]["temperature"], serde_json::json!(1.0));
        assert!(v.get("temperature").is_none());
    }

    // --- chat spec: Temperatura ---

    #[test]
    fn build_body_uses_configured_temperature() {
        let mut b = bot();
        b.temperature = "0.2".to_string();
        let body = b.build_body("sys", "q").expect("body");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["temperature"], serde_json::json!(0.2));
    }

    #[test]
    fn build_body_invalid_temperature_returns_readable_error() {
        let mut b = bot();
        b.temperature = "alta".to_string();
        let err = b.build_body("sys", "q").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("temperature"), "msg: {msg}");
        assert!(matches!(err, IABotError::NumberParse { .. }));
    }

    // --- chat spec: Tratamiento de respuestas ---

    #[test]
    fn parse_response_ok_returns_content() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"Hola"}}]}"#;
        assert_eq!(parse_response(body).unwrap(), "Hola");
    }

    #[test]
    fn parse_response_provider_error_returns_message() {
        let body = r#"{"error":{"message":"invalid_api_key"}}"#;
        let err = parse_response(body).unwrap_err();
        assert!(err.to_string().contains("invalid_api_key"), "err: {err}");
        assert!(matches!(err, IABotError::Api(_)));
    }

    #[test]
    fn parse_response_unexpected_shape_returns_typed_error() {
        let body = r#"{"foo":"bar"}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::UnexpectedResponse));
    }

    #[test]
    fn parse_response_empty_choices_returns_typed_error() {
        let body = r#"{"choices":[]}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::UnexpectedResponse));
    }

    // --- config spec: Valores por defecto ---

    #[test]
    fn yaml_partial_uses_defaults() {
        let cfg: IABot = serde_yaml::from_str("model: mi-modelo").unwrap();
        assert_eq!(cfg.model, "mi-modelo");
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.base_url, "api.openai.com");
        assert_eq!(cfg.endpoint, "v1/chat/completions");
        assert_eq!(cfg.temperature, "1");
        assert_eq!(cfg.num_ctx, 32000);
        assert!(cfg.use_https);
        assert_eq!(cfg.log_level, "info");
        assert_eq!(cfg.token, "");
    }

    // --- config spec: Errores legibles ---

    #[tokio::test]
    async fn read_content_invalid_yaml_mentions_dime_yml() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-invalid-{}.yml", std::process::id()));
        std::fs::write(&path, "provider: [unclosed").unwrap();
        let err = IABot::read_content(&path).await.unwrap_err();
        assert!(err.to_string().contains("dime.yml"), "msg: {err}");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn read_content_missing_file_mentions_dime_yml() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-missing-{}.yml", std::process::id()));
        let err = IABot::read_content(&path).await.unwrap_err();
        assert!(err.to_string().contains("dime.yml"), "msg: {err}");
    }

    // --- config spec: El token no se registra ---

    #[test]
    fn debug_redacts_token() {
        let dbg = format!("{:?}", openai_bot());
        assert!(!dbg.contains("secreto-super-secreto"), "leak: {dbg}");
        assert!(dbg.contains("***"), "dbg: {dbg}");
    }

    #[test]
    fn debug_empty_token_is_not_shown_as_redacted() {
        let dbg = format!("{:?}", bot());
        assert!(!dbg.contains("***"), "dbg: {dbg}");
    }

    // --- MAYOR 1: cabecera Authorization ---

    #[test]
    fn auth_header_openai_with_token() {
        assert_eq!(
            openai_bot().auth_header(),
            Some("Bearer secreto-super-secreto".to_string())
        );
    }

    #[test]
    fn auth_header_openai_without_token() {
        let mut b = openai_bot();
        b.token = String::new();
        assert_eq!(b.auth_header(), None);
    }

    #[test]
    fn auth_header_ollama_is_none_even_with_token() {
        let mut b = ollama_bot();
        b.token = "algo".to_string();
        assert_eq!(b.auth_header(), None);
    }

    // --- MAYOR 1: validación de token vacío ---

    #[test]
    fn validate_openai_with_empty_token_fails() {
        let mut b = openai_bot();
        b.token = String::new();
        assert!(b.validate().is_err());
    }

    #[test]
    fn validate_ollama_with_empty_token_ok() {
        assert!(ollama_bot().validate().is_ok());
    }

    // --- MENOR 3: comparación de proveedor case-insensitive ---

    #[test]
    fn is_ollama_is_case_insensitive() {
        let mut b = bot();
        b.provider = "Ollama".to_string();
        assert!(b.is_ollama());
    }

    // --- MENOR 2: error de proveedor como cadena ---

    #[test]
    fn parse_response_provider_error_as_string() {
        let body = r#"{"error":"boom"}"#;
        let err = parse_response(body).unwrap_err();
        assert!(matches!(err, IABotError::Api(_)));
        assert!(err.to_string().contains("boom"));
    }

    // --- MENOR 1: `error: null` debe caer a `choices` ---

    #[test]
    fn parse_response_null_error_falls_back_to_choices() {
        let body = r#"{"error":null,"choices":[{"message":{"content":"Hola"}}]}"#;
        assert_eq!(parse_response(body).unwrap(), "Hola");
    }

    // --- MENOR 2: no existe dime.yml -> se crea con defaults ---

    #[tokio::test]
    async fn write_default_creates_file_with_defaults() {
        let mut path = std::env::temp_dir();
        path.push(format!("dime-default-{}.yml", std::process::id()));
        let _ = std::fs::remove_file(&path);

        IABot::write_default(&path).await.expect("write_default");
        assert!(path.exists(), "el fichero por defecto debe existir");

        let cfg = IABot::read_content(&path).await.expect("read_content");
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.base_url, "api.openai.com");
        assert_eq!(cfg.model, "gpt-3.5-turbo");
        assert_eq!(cfg.num_ctx, 32000);
        assert!(cfg.use_https);

        let _ = std::fs::remove_file(&path);
    }
}
