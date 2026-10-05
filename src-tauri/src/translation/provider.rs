use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original_text: String,
    pub translated_text: String,
    pub target_language: String,
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    async fn translate(
        &self,
        text: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> AppResult<TranslationResult>;
}

/// Concrete enum wrapper — avoids `dyn TranslationProvider` which is not object-safe
/// when using async fn on stable Rust 1.95.
pub enum TranslationProviderKind {
    Cloud(crate::translation::cloud::CloudTranslationProvider),
}

impl TranslationProviderKind {
    pub async fn translate(
        &self,
        text: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> AppResult<TranslationResult> {
        match self {
            TranslationProviderKind::Cloud(p) => p.translate(text, target_lang, source_lang).await,
        }
    }
}
