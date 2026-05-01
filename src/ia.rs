use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fmt;
use std::{path::PathBuf, process};
use tokio::fs::read_to_string;
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct IABotError {
    message: String,
}

impl IABotError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// Display implementation is required for std::error::Error.
impl fmt::Display for IABotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for IABotError {} // Defaul

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IABot {
    #[serde(default = "get_default_log_level")]
    log_level: String,
    #[serde(default = "get_default_provider")]
    provider: String,
    #[serde(default = "get_default_base_url")]
    base_url: String,
    #[serde(default = "get_default_endpoint")]
    endpoint: String,
    #[serde(default = "get_default_models_endpoint")]
    models_endpoint: String,
    #[serde(default = "get_default_token")]
    token: String,
    #[serde(default = "get_default_model")]
    model: String,
    // What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic.
    #[serde(default = "get_default_temperature")]
    temperature: String,
    #[serde(default = "get_default_use_https")]
    use_https: bool,
}

fn get_default_log_level() -> String {
    "info".to_string()
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

fn get_default_models_endpoint() -> String {
    "v1/models".to_string()
}

fn get_default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn get_default_temperature() -> String {
    "1".to_string()
}

impl IABot {
    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }

    pub fn get_provider(&self) -> &str {
        &self.provider
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }

    pub async fn save(&self, path: &PathBuf) {
        let _ = tokio::fs::write(path, serde_yaml::to_string(&self).unwrap().as_bytes()).await;
    }

    pub async fn ask(
        &self,
        system: &str,
        question: &str,
    ) -> Result<String, Box<dyn Error + 'static>> {
        let protocol = if self.use_https { "https" } else { "http" };
        let url = format!("{}://{}/{}", protocol, self.base_url, self.endpoint);
        debug!("Url: {}", url);
        debug!("Provider: {}", self.provider);
        debug!("Question: {}", question);

        let client = Client::new();

        let body = serde_json::to_string(&json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": question}
            ]
        }))
        .unwrap();

        let mut request_builder = client.post(&url).header("Content-Type", "application/json");

        // Agregar token solo si es OpenAI y el token no está vacío
        if self.provider.to_lowercase() == "openai" && !self.token.is_empty() {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", self.token));
        }

        match request_builder.body(body).send().await {
            Ok(resp) => {
                let body = resp.text().await?;
                debug!("{}", &body);

                let data: serde_json::Value = serde_json::from_str(&body).unwrap();

                // Manejo de errores
                if data.get("error").is_some() {
                    let error = data.get("error").unwrap();
                    let message = if error.is_object() {
                        error.get("message").unwrap().as_str().unwrap()
                    } else {
                        error.as_str().unwrap()
                    };
                    Err(Box::new(IABotError::new(message)))
                } else {
                    // Ambos OpenAI y Ollama tienen la misma estructura de respuesta
                    let command = data.get("choices").unwrap().as_array().unwrap()[0]
                        .get("message")
                        .unwrap()
                        .get("content")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    Ok(command.to_string())
                }
            }
            Err(e) => {
                error!("{}", e);
                Err(e.into())
            }
        }
    }

    fn default() -> Self {
        Self {
            log_level: get_default_log_level(),
            provider: get_default_provider(),
            base_url: get_default_base_url(),
            endpoint: get_default_endpoint(),
            models_endpoint: get_default_models_endpoint(),
            token: get_default_token(),
            model: get_default_model(),
            temperature: get_default_temperature(),
            use_https: get_default_use_https(),
        }
    }

    pub async fn write_default(file: &PathBuf) {
        let default = Self::default();
        let _ = tokio::fs::write(file, serde_yaml::to_string(&default).unwrap().as_bytes()).await;
    }

    pub async fn read_content(file: &PathBuf) -> IABot {
        info!("File to read: {:?}", file);
        let content = match read_to_string(file).await {
            Ok(value) => {
                debug!("Content read: {}", value);
                value
            }
            Err(e) => {
                error!("Error: {}", e);
                println!("Error with config file `config.yml`: {}", e);
                process::exit(1);
            }
        };
        match serde_yaml::from_str(&content) {
            Ok(configuration) => configuration,
            Err(e) => {
                error!("Error: {}", e);
                println!("Error with config file `config.yml`: {}", e);
                process::exit(1);
            }
        }
    }
}
