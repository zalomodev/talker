use crate::error::{AppError, AppResult};
use crate::transcription::provider::{SpeechProvider, TranscriptResult};
use async_trait::async_trait;
use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct SherpaStreamingProvider {
    model_dir: PathBuf,
    /// Created once on first use and reused for every segment. Creating a
    /// recognizer loads ~100MB+ of ONNX sessions, so it must never happen
    /// per-segment (that piled up concurrent loads and aborted the process).
    /// The mutex also serializes decodes.
    shared: Arc<Mutex<Option<OnlineRecognizer>>>,
    /// Active utterance stream for true chunk-by-chunk streaming. Each audio
    /// chunk is fed (and decoded) exactly once — no re-decoding buffers.
    stream: Mutex<Option<OnlineStream>>,
}

impl SherpaStreamingProvider {
    pub fn new(model_dir: PathBuf) -> Self {
        Self {
            model_dir,
            shared: Arc::new(Mutex::new(None)),
            stream: Mutex::new(None),
        }
    }

    fn ensure_loaded(&self) -> AppResult<()> {
        let mut guard = self
            .shared
            .lock()
            .map_err(|e| AppError::Transcription(format!("Sherpa lock poisoned: {e}")))?;
        if guard.is_none() {
            let created = create_recognizer(&self.model_dir)?;
            *guard = Some(created);
            log::info!("Sherpa recognizer ready");
        }
        Ok(())
    }

    /// Pre-load the model and run a short silence decode so the first real
    /// utterance doesn't pay the one-time initialization cost. Blocking.
    pub fn warmup(&self) {
        let started = std::time::Instant::now();
        if self.ensure_loaded().is_err() {
            return;
        }
        let guard = match self.shared.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(rec) = guard.as_ref() {
            let stream = rec.create_stream();
            stream.accept_waveform(16000, &vec![0.0f32; 8000]);
            stream.input_finished();
            while rec.is_ready(&stream) {
                rec.decode(&stream);
            }
            let _ = rec.get_result(&stream);
        }
        log::info!(
            "Sherpa warmed up in {:.1}s",
            started.elapsed().as_secs_f32()
        );
    }

    /// Start a new utterance (drops any previous unfinished stream).
    pub fn utterance_begin(&self) {
        if self.ensure_loaded().is_err() {
            return;
        }
        let guard = match self.shared.try_lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(rec) = guard.as_ref() {
            if let Ok(mut slot) = self.stream.try_lock() {
                *slot = Some(rec.create_stream());
            }
        }
    }

    /// Feed one live chunk; decodes whatever frames are ready. Cheap
    /// (milliseconds) — safe to call per captured chunk. Never blocks:
    /// skips silently if the recognizer is momentarily busy.
    pub fn utterance_feed(&self, pcm: &[f32]) {
        let guard = match self.shared.try_lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let slot = match self.stream.try_lock() {
            Ok(s) => s,
            Err(_) => return,
        };
        if let (Some(rec), Some(stream)) = (guard.as_ref(), slot.as_ref()) {
            stream.accept_waveform(16000, pcm);
            let mut steps = 0u32;
            while rec.is_ready(stream) {
                rec.decode(stream);
                steps += 1;
                if steps > 200 {
                    break;
                }
            }
        }
    }

    /// Current hypothesis for the active utterance (may revise itself).
    pub fn utterance_partial(&self) -> String {
        let guard = match self.shared.try_lock() {
            Ok(g) => g,
            Err(_) => return String::new(),
        };
        let slot = match self.stream.try_lock() {
            Ok(s) => s,
            Err(_) => return String::new(),
        };
        if let (Some(rec), Some(stream)) = (guard.as_ref(), slot.as_ref()) {
            return rec
                .get_result(stream)
                .map(|r| r.text)
                .unwrap_or_default()
                .trim()
                .to_string();
        }
        String::new()
    }

    /// End the utterance: flush trailing context, drain, return final text.
    pub fn utterance_finish(&self) -> String {
        let mut slot = match self.stream.try_lock() {
            Ok(s) => s,
            Err(_) => return String::new(),
        };
        let stream = match slot.take() {
            Some(s) => s,
            None => return String::new(),
        };
        drop(slot);
        let guard = match self.shared.try_lock() {
            Ok(g) => g,
            Err(_) => return String::new(),
        };
        if let Some(rec) = guard.as_ref() {
            stream.input_finished();
            let mut steps = 0u32;
            while rec.is_ready(&stream) {
                rec.decode(&stream);
                steps += 1;
                if steps > 2000 {
                    break;
                }
            }
            return rec
                .get_result(&stream)
                .map(|r| r.text)
                .unwrap_or_default()
                .trim()
                .to_string();
        }
        String::new()
    }

    pub fn model_language_tag(&self) -> Option<String> {
        model_language(&self.model_dir)
    }
}

/// Recursively collect *.onnx files whose name contains `keyword`.
fn collect_candidates(dir: &Path, keyword: &str, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_candidates(&path, keyword, out);
            } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains(keyword) && name.ends_with(".onnx") {
                    out.push(path);
                }
            }
        }
    }
}

/// Deterministic pick matching the official sherpa examples:
/// int8 encoder/joiner + fp32 decoder, falling back to whatever exists.
fn pick_model_file(dir: &Path, keyword: &str, prefer_int8: bool) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    collect_candidates(dir, keyword, &mut candidates);
    if candidates.is_empty() {
        return None;
    }
    candidates.sort();
    if prefer_int8 {
        if let Some(p) = candidates
            .iter()
            .find(|p| p.to_string_lossy().ends_with(".int8.onnx"))
        {
            return Some(p.clone());
        }
    } else if let Some(p) = candidates
        .iter()
        .find(|p| !p.to_string_lossy().ends_with(".int8.onnx"))
    {
        return Some(p.clone());
    }
    candidates.into_iter().next()
}

fn find_tokens_txt(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_tokens_txt(&path) {
                    return Some(found);
                }
            } else if path.is_file()
                && path.file_name().and_then(|n| n.to_str()) == Some("tokens.txt")
            {
                return Some(path);
            }
        }
    }
    None
}

/// English-only zipformer models always output English; the bilingual
/// model mixes languages, so we report no fixed language for it.
fn model_language(model_dir: &Path) -> Option<String> {
    let name = model_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name.contains("bilingual") {
        None
    } else if name.contains("-es-") || name.contains("-es_") {
        Some("es".to_string())
    } else {
        Some("en".to_string())
    }
}

fn create_recognizer(model_dir: &Path) -> AppResult<OnlineRecognizer> {
    let encoder = pick_model_file(model_dir, "encoder", true)
        .ok_or_else(|| AppError::Model("Sherpa model missing encoder .onnx file".into()))?;
    let decoder = pick_model_file(model_dir, "decoder", false)
        .ok_or_else(|| AppError::Model("Sherpa model missing decoder .onnx file".into()))?;
    let joiner = pick_model_file(model_dir, "joiner", true)
        .ok_or_else(|| AppError::Model("Sherpa model missing joiner .onnx file".into()))?;
    let tokens = find_tokens_txt(model_dir)
        .ok_or_else(|| AppError::Model("Sherpa model missing tokens.txt".into()))?;

    log::info!(
        "Loading Sherpa recognizer: encoder={} decoder={} joiner={}",
        encoder.display(),
        decoder.display(),
        joiner.display()
    );

    let mut config = OnlineRecognizerConfig::default();
    config.model_config.transducer.encoder = Some(encoder.to_string_lossy().to_string());
    config.model_config.transducer.decoder = Some(decoder.to_string_lossy().to_string());
    config.model_config.transducer.joiner = Some(joiner.to_string_lossy().to_string());
    config.model_config.tokens = Some(tokens.to_string_lossy().to_string());
    config.model_config.num_threads = 4;
    config.decoding_method = Some("greedy_search".to_string());
    config.enable_endpoint = true;

    OnlineRecognizer::create(&config)
        .ok_or_else(|| AppError::Model("Failed to create Sherpa-ONNX recognizer".into()))
}

#[async_trait]
impl SpeechProvider for SherpaStreamingProvider {
    async fn transcribe(
        &self,
        pcm_16k_mono: &[f32],
        _language: Option<&str>,
    ) -> AppResult<TranscriptResult> {
        if pcm_16k_mono.len() < 16000 / 8 {
            return Ok(TranscriptResult {
                text: String::new(),
                language: Some("en".to_string()),
                duration_secs: 0.0,
                is_partial: false,
            });
        }
        let model_dir = self.model_dir.clone();
        let shared = Arc::clone(&self.shared);
        let pcm = pcm_16k_mono.to_vec();

        // Heavy work (first-time model load + every decode) runs on a
        // blocking thread. The mutex guarantees a single shared recognizer
        // and serialized decodes.
        let text = tokio::task::spawn_blocking(move || -> AppResult<String> {
            let started = std::time::Instant::now();
            let mut guard = shared
                .lock()
                .map_err(|e| AppError::Transcription(format!("Sherpa lock poisoned: {e}")))?;
            if guard.is_none() {
                let created = create_recognizer(&model_dir)?;
                *guard = Some(created);
                log::info!("Sherpa recognizer ready");
            }
            let recognizer = guard
                .as_ref()
                .ok_or_else(|| AppError::Transcription("Sherpa recognizer missing".into()))?;

            let stream = recognizer.create_stream();
            stream.accept_waveform(16000, &pcm);
            stream.input_finished();
            let mut steps = 0u32;
            while recognizer.is_ready(&stream) {
                recognizer.decode(&stream);
                steps += 1;
                if steps > 20_000 {
                    break;
                }
            }
            let text = recognizer
                .get_result(&stream)
                .map(|r| r.text)
                .unwrap_or_default()
                .trim()
                .to_string();
            log::info!(
                "Sherpa decoded {:.1}s audio in {:.1}s -> {} chars",
                pcm.len() as f32 / 16000.0,
                started.elapsed().as_secs_f32(),
                text.chars().count()
            );
            Ok(text)
        })
        .await
        .map_err(|e| AppError::Transcription(format!("Sherpa task failed: {e}")))??;

        Ok(TranscriptResult {
            text,
            language: model_language(&self.model_dir),
            duration_secs: pcm_16k_mono.len() as f32 / 16000.0,
            is_partial: false,
        })
    }
}
