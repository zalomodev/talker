use crate::error::{AppError, AppResult};
use crate::transcription::provider::{SpeechProvider, TranscriptResult};
use async_trait::async_trait;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;
use std::io::Cursor;
use std::time::Duration;

pub struct GroqSpeechProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Deserialize)]
struct GroqTranscriptionResponse {
    text: String,
    language: Option<String>,
    duration: Option<f32>,
}

impl GroqSpeechProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key,
            model,
        }
    }

    pub async fn test_connection(&self) -> AppResult<String> {
        let resp = self
            .client
            .get("https://api.groq.com/openai/v1/models")
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .send()
            .await
            .map_err(|e| AppError::Transcription(format!("Connection failed: {e}")))?;

        match resp.status() {
            reqwest::StatusCode::OK => Ok("Connected to Groq successfully".to_string()),
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(AppError::Transcription("Authentication failed: Invalid Groq API key".into()))
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                Err(AppError::Transcription("Groq rate limit exceeded".into()))
            }
            status => {
                let err_text = resp.text().await.unwrap_or_default();
                Err(AppError::Transcription(format!(
                    "Groq API returned HTTP {status}: {err_text}"
                )))
            }
        }
    }

    fn encode_wav(pcm_16k_mono: &[f32]) -> AppResult<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| AppError::Audio(format!("Failed to initialize WAV writer: {e}")))?;

        for &sample in pcm_16k_mono {
            let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(s)
                .map_err(|e| AppError::Audio(format!("Failed to write WAV sample: {e}")))?;
        }

        writer
            .finalize()
            .map_err(|e| AppError::Audio(format!("Failed to finalize WAV audio: {e}")))?;

        Ok(cursor.into_inner())
    }
}

#[async_trait]
impl SpeechProvider for GroqSpeechProvider {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        language: Option<&str>,
    ) -> AppResult<TranscriptResult> {
        if pcm_16k_mono.is_empty() {
            return Ok(TranscriptResult {
                text: String::new(),
                language: None,
                duration_secs: 0.0,
                is_partial: false,
            });
        }

        let wav_bytes = Self::encode_wav(pcm_16k_mono)?;
        let duration_secs = pcm_16k_mono.len() as f32 / 16000.0;

        let part = Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Transcription(format!("MIME error: {e}")))?;

        let mut form = Form::new()
            .part("file", part)
            .text("model", self.model.clone())
            .text("response_format", "verbose_json");

        if let Some(lang) = language {
            if lang != "auto" && !lang.is_empty() {
                form = form.text("language", lang.to_string());
            }
        }

        let resp = self
            .client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Transcription(format!("Groq network request failed: {e}")))?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let data: GroqTranscriptionResponse = resp.json().await.map_err(|e| {
                    AppError::Transcription(format!("Failed to parse Groq response JSON: {e}"))
                })?;

                Ok(TranscriptResult {
                    text: data.text.trim().to_string(),
                    language: data.language,
                    duration_secs: data.duration.unwrap_or(duration_secs),
                    is_partial: false,
                })
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(AppError::Transcription("Authentication failed: Invalid Groq API key".into()))
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                Err(AppError::Transcription("Groq rate limit exceeded".into()))
            }
            status => {
                let err_text = resp.text().await.unwrap_or_default();
                Err(AppError::Transcription(format!(
                    "Groq returned error HTTP {status}: {err_text}"
                )))
            }
        }
    }
}
