use crate::error::{AppError, AppResult};
use crate::translation::provider::{TranslationProvider, TranslationResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

pub struct CloudTranslationProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    model: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

impl CloudTranslationProvider {
    pub fn new(api_key: String, endpoint: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key,
            endpoint,
            model,
        }
    }

    pub fn groq(api_key: String, model: String) -> Self {
        Self::new(
            api_key,
            "https://api.groq.com/openai/v1/chat/completions".to_string(),
            model,
        )
    }

    pub fn openai(api_key: String, model: String) -> Self {
        Self::new(
            api_key,
            "https://api.openai.com/v1/chat/completions".to_string(),
            model,
        )
    }

    pub fn openrouter(api_key: String, model: String) -> Self {
        Self::new(
            api_key,
            "https://openrouter.ai/api/v1/chat/completions".to_string(),
            model,
        )
    }
}

#[async_trait]
impl TranslationProvider for CloudTranslationProvider {
    async fn translate(
        &self,
        text: &str,
        target_lang: &str,
        _source_lang: Option<&str>,
    ) -> AppResult<TranslationResult> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(TranslationResult {
                original_text: text.to_string(),
                translated_text: String::new(),
                target_language: target_lang.to_string(),
            });
        }

        let system_prompt = format!(
            "You are a subtitle translator. Translate the given spoken dialogue accurately and concisely into {target_lang}. Preserve tone and colloquialisms. Output ONLY the translated sentence, without commentary, prefixes, quotation marks, or explanations."
        );

        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": trimmed
                }
            ],
            "temperature": 0.2,
            "max_tokens": 256
        });

        let resp = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Translation(format!("Translation network request failed: {e}")))?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let data: ChatCompletionResponse = resp.json().await.map_err(|e| {
                    AppError::Translation(format!("Failed to parse translation response: {e}"))
                })?;

                let translated = data
                    .choices
                    .first()
                    .map(|c| c.message.content.trim().to_string())
                    .unwrap_or_else(|| trimmed.to_string());

                Ok(TranslationResult {
                    original_text: text.to_string(),
                    translated_text: translated,
                    target_language: target_lang.to_string(),
                })
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(AppError::Translation("Translation authentication failed: Invalid API key".into()))
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                Err(AppError::Translation("Translation rate limit exceeded".into()))
            }
            status => {
                let err_text = resp.text().await.unwrap_or_default();
                Err(AppError::Translation(format!(
                    "Translation API returned error HTTP {status}: {err_text}"
                )))
            }
        }
    }
}
