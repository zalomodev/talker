use crate::error::{AppError, AppResult};
use crate::transcription::provider::{SpeechProvider, TranscriptResult};
use async_trait::async_trait;
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::ops::softmax;
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self as m, audio, Config};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

static MEL_FILTERS: &[u8] = include_bytes!("melfilters.bytes");
static MEL_FILTERS_128: &[u8] = include_bytes!("melfilters128.bytes");

pub struct LocalWhisperProvider {
    model_path: PathBuf,
    inner: Arc<Mutex<Option<LoadedModel>>>,
}

struct LoadedModel {
    model: m::model::Whisper,
    tokenizer: Tokenizer,
    config: Config,
    device: Device,
    mel_filters: Vec<f32>,
}

impl LocalWhisperProvider {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    fn load_weights(path: &Path) -> AppResult<LoadedModel> {
        let safetensors_path = path.join("model.safetensors");
        let config_path = path.join("config.json");
        let tokenizer_path = path.join("tokenizer.json");

        if !safetensors_path.exists() || !config_path.exists() || !tokenizer_path.exists() {
            return Err(AppError::Model(format!(
                "The selected local model is not installed at {path:?}. Please download it in Settings."
            )));
        }

        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| AppError::Model(format!("Failed to read model config: {e}")))?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| AppError::Model(format!("Failed to parse config.json: {e}")))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| AppError::Model(format!("Failed to load tokenizer: {e}")))?;

        let device = Device::Cpu;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[&safetensors_path], m::DTYPE, &device)
                .map_err(|e| AppError::Model(format!("Failed to map model weights: {e}")))?
        };

        let model = m::model::Whisper::load(&vb, config.clone())
            .map_err(|e| AppError::Model(format!("Failed to build Whisper neural net: {e}")))?;

        let mel_bytes = if config.num_mel_bins == 128 {
            MEL_FILTERS_128
        } else {
            MEL_FILTERS
        };

        let mut mel_filters = vec![0.0f32; mel_bytes.len() / 4];
        for (i, chunk) in mel_bytes.chunks_exact(4).enumerate() {
            mel_filters[i] = f32::from_le_bytes(chunk.try_into().unwrap());
        }

        Ok(LoadedModel {
            model,
            tokenizer,
            config,
            device,
            mel_filters,
        })
    }
}

#[async_trait]
impl SpeechProvider for LocalWhisperProvider {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        _language: Option<&str>,
    ) -> AppResult<TranscriptResult> {
        if pcm_16k_mono.is_empty() {
            return Ok(TranscriptResult {
                text: String::new(),
                language: None,
                duration_secs: 0.0,
                is_partial: false,
            });
        }

        let mut guard = self.inner.lock().await;
        if guard.is_none() {
            let loaded = Self::load_weights(&self.model_path)?;
            *guard = Some(loaded);
        }

        let state = guard.as_mut().unwrap();
        let duration_secs = pcm_16k_mono.len() as f32 / 16000.0;

        // Pad or truncate to Whisper standard frames
        let mel_vec = audio::pcm_to_mel(&state.config, pcm_16k_mono, &state.mel_filters);
        let num_frames = mel_vec.len() / state.config.num_mel_bins;
        if num_frames == 0 {
            return Ok(TranscriptResult {
                text: String::new(),
                language: None,
                duration_secs,
                is_partial: false,
            });
        }

        let mel = Tensor::from_vec(
            mel_vec,
            (1, state.config.num_mel_bins, num_frames),
            &state.device,
        )
        .map_err(|e| AppError::Transcription(format!("Tensor creation failed: {e}")))?;

        // Run encoder
        let audio_features = state
            .model
            .encoder
            .forward(&mel, true)
            .map_err(|e| AppError::Transcription(format!("Encoder error: {e}")))?;

        // Setup decoder initial tokens: <|startoftranscript|> <|transcribe|> <|notimestamps|>
        let sot_token = state
            .tokenizer
            .token_to_id(m::SOT_TOKEN)
            .ok_or_else(|| AppError::Transcription("Missing SOT token".into()))?;
        let transcribe_token = state
            .tokenizer
            .token_to_id(m::TRANSCRIBE_TOKEN)
            .ok_or_else(|| AppError::Transcription("Missing transcribe token".into()))?;
        let no_timestamps_token = state
            .tokenizer
            .token_to_id(m::NO_TIMESTAMPS_TOKEN)
            .ok_or_else(|| AppError::Transcription("Missing notimestamps token".into()))?;
        let eot_token = state
            .tokenizer
            .token_to_id(m::EOT_TOKEN)
            .ok_or_else(|| AppError::Transcription("Missing EOT token".into()))?;

        let mut tokens = vec![sot_token, transcribe_token, no_timestamps_token];
        let max_tokens = state.config.max_target_positions.min(128);

        // Greedy decoding
        for i in 0..max_tokens {
            let tokens_tensor = Tensor::new(tokens.as_slice(), &state.device)
                .map_err(|e| AppError::Transcription(format!("Token tensor failed: {e}")))?
                .unsqueeze(0)
                .map_err(|e| AppError::Transcription(format!("Unsqueeze failed: {e}")))?;

            let ys = state
                .model
                .decoder
                .forward(&tokens_tensor, &audio_features, i == 0)
                .map_err(|e| AppError::Transcription(format!("Decoder forward failed: {e}")))?;

            let (_, seq_len, _) = ys
                .dims3()
                .map_err(|e| AppError::Transcription(format!("Dims failed: {e}")))?;

            let logits = state
                .model
                .decoder
                .final_linear(&ys.i((..1, seq_len - 1..)).map_err(|e| {
                    AppError::Transcription(format!("Slice failed: {e}"))
                })?)
                .map_err(|e| AppError::Transcription(format!("Final linear failed: {e}")))?
                .i(0)
                .map_err(|e| AppError::Transcription(format!("Index failed: {e}")))?
                .i(0)
                .map_err(|e| AppError::Transcription(format!("Index failed: {e}")))?;

            let probs = softmax(&logits, 0)
                .map_err(|e| AppError::Transcription(format!("Softmax failed: {e}")))?;
            let probs_vec: Vec<f32> = probs
                .to_vec1()
                .map_err(|e| AppError::Transcription(format!("To vec failed: {e}")))?;

            let next_token = probs_vec
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, _)| idx as u32)
                .unwrap_or(eot_token);

            if next_token == eot_token {
                break;
            }
            tokens.push(next_token);
        }

        // Decode tokens back to text (skipping initial 3 special prompt tokens)
        let text_tokens = if tokens.len() > 3 {
            &tokens[3..]
        } else {
            &[]
        };

        let text = state
            .tokenizer
            .decode(text_tokens, true)
            .map_err(|e| AppError::Transcription(format!("Tokenizer decode failed: {e}")))?;

        Ok(TranscriptResult {
            text: text.trim().to_string(),
            language: None,
            duration_secs,
            is_partial: false,
        })
    }
}
