use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_secs: f32,
    pub is_partial: bool,
}

#[async_trait]
pub trait SpeechProvider: Send + Sync {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        language: Option<&str>,
    ) -> AppResult<TranscriptResult>;
}

/// Concrete enum wrapper — avoids `dyn SpeechProvider` which is not object-safe
/// when using async fn (even with #[async_trait] on stable Rust 1.95).
pub enum SpeechProviderKind {
    Groq(crate::transcription::groq::GroqSpeechProvider),
    Local(crate::transcription::whisper_local::LocalWhisperProvider),
    Sherpa(crate::transcription::sherpa::SherpaStreamingProvider),
}

impl SpeechProviderKind {
    pub async fn transcribe(
        &self,
        pcm: &[f32],
        language: Option<&str>,
    ) -> AppResult<TranscriptResult> {
        match self {
            SpeechProviderKind::Groq(p) => p.transcribe(pcm, language).await,
            SpeechProviderKind::Local(p) => p.transcribe(pcm, language).await,
            SpeechProviderKind::Sherpa(p) => p.transcribe(pcm, language).await,
        }
    }
}
