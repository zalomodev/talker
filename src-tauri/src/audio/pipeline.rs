use crate::audio::capture::{AudioCapture, AudioSource};
use crate::audio::vad::VoiceActivityDetector;
use crate::transcription::provider::SpeechProviderKind;
use crate::transcription::sherpa::SherpaStreamingProvider;
use crate::translation::provider::TranslationProviderKind;
use crate::windows::foreground::get_foreground_app;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEventPayload {
    pub id: u64,
    pub text: String,
    pub translation: Option<String>,
    pub is_final: bool,
    pub language: Option<String>,
    /// Seconds from utterance start (streaming) or decode duration (batch).
    pub latency_ms: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatusPayload {
    pub is_active: bool,
    pub source_description: String,
    pub active_pid: Option<u32>,
    pub active_app: Option<String>,
    pub vad_speech_detected: bool,
    pub error: Option<String>,
}

pub struct AudioPipeline {
    is_running: Arc<AtomicBool>,
    handle: Arc<TokioMutex<Option<tokio::task::JoinHandle<()>>>>,
}

/// Live state of one streaming utterance (sherpa path).
struct SherpaStreamState {
    active: bool,
    silent_samples: usize,
    last_partial: String,
    last_partial_time: Instant,
    start: Instant,
}

impl SherpaStreamState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            active: false,
            silent_samples: 0,
            last_partial: String::new(),
            last_partial_time: now,
            start: now,
        }
    }
}

/// Finalize the active streaming utterance and emit it (with translation).
/// Translation runs in a detached task: the decoder is already free.
fn finish_stream_utterance(
    provider: &SherpaStreamingProvider,
    app_handle: &AppHandle,
    trans_provider: Option<&Arc<TranslationProviderKind>>,
    target_lang: &str,
    st: &mut SherpaStreamState,
    seq_id: &mut u64,
) {
    let text = provider.utterance_finish();
    st.active = false;
    st.silent_samples = 0;
    st.last_partial.clear();
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        return;
    }
    let language = provider.model_language_tag();
    let id = *seq_id;
    *seq_id += 1;
    let tp = trans_provider.cloned();
    let t_lang = target_lang.to_string();
    let handle = app_handle.clone();
    let latency_ms = Some(st.start.elapsed().as_secs_f32() * 1000.0);
    tokio::spawn(async move {
        let mut translation = None;
        if let Some(tp) = tp {
            if let Ok(tres) = tp
                .translate(&trimmed, &t_lang, language.as_deref())
                .await
            {
                translation = Some(tres.translated_text);
            }
        }
        let _ = handle.emit(
            "transcription-event",
            TranscriptEventPayload {
                id,
                text: trimmed,
                translation,
                is_final: true,
                language,
                latency_ms,
            },
        );
    });
}

/// One capture tick on the true-streaming path: feed the chunk exactly once,
/// surface a live partial ~every second, finalize after sustained silence.
#[allow(clippy::too_many_arguments)]
fn streaming_tick(
    provider: &SherpaStreamingProvider,
    app_handle: &AppHandle,
    trans_provider: Option<&Arc<TranslationProviderKind>>,
    target_lang: &str,
    samples: &[f32],
    speech_now: bool,
    st: &mut SherpaStreamState,
    seq_id: &mut u64,
    min_silence_ms: u32,
) {
    const PARTIAL_EVERY: Duration = Duration::from_millis(1200);
    const MAX_UTTERANCE: Duration = Duration::from_secs(20);

    if speech_now {
        st.silent_samples = 0;
        if !st.active {
            provider.utterance_begin();
            st.active = true;
            st.start = Instant::now();
            st.last_partial.clear();
        }
        provider.utterance_feed(samples);

        // Chain very long utterances instead of letting one grow forever.
        if st.start.elapsed() > MAX_UTTERANCE {
            finish_stream_utterance(
                provider,
                app_handle,
                trans_provider,
                target_lang,
                st,
                seq_id,
            );
            provider.utterance_begin();
            st.active = true;
            st.start = Instant::now();
            st.last_partial.clear();
            return;
        }

        if st.last_partial_time.elapsed() > PARTIAL_EVERY {
            st.last_partial_time = Instant::now();
            let trimmed = provider.utterance_partial();
            let trimmed = trimmed.trim().to_string();
            if !trimmed.is_empty() && trimmed != st.last_partial {
                st.last_partial = trimmed.clone();
                let _ = app_handle.emit(
                    "transcription-event",
                    TranscriptEventPayload {
                        id: *seq_id,
                        text: trimmed,
                        translation: None,
                        is_final: false,
                        language: provider.model_language_tag(),
                        latency_ms: Some(st.start.elapsed().as_secs_f32() * 1000.0),
                    },
                );
            }
        }
    } else if st.active {
        // Feed trailing silence too: the endpointing needs it to cut cleanly.
        provider.utterance_feed(samples);
        st.silent_samples += samples.len();
        let hangover = ((min_silence_ms.max(500) as usize) * 16000) / 1000;
        if st.silent_samples >= hangover {
            finish_stream_utterance(
                provider,
                app_handle,
                trans_provider,
                target_lang,
                st,
                seq_id,
            );
        }
    }
    // Silence with no active utterance: nothing to do.
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            handle: Arc::new(TokioMutex::new(None)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub async fn start(
        &self,
        app_handle: AppHandle,
        audio_mode: String,
        pinned_pid: Option<u32>,
        speech_provider: SpeechProviderKind,
        translation_provider: Option<TranslationProviderKind>,
        target_lang: String,
        vad_threshold: f32,
        min_speech_ms: u32,
        min_silence_ms: u32,
        transcription_lang: Option<String>,
    ) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let is_running_flag = Arc::clone(&self.is_running);
        // Wrap providers in Arc so they can be cloned into spawned tasks
        let speech_provider = Arc::new(speech_provider);
        let translation_provider = translation_provider.map(Arc::new);
        // Local transcription (sherpa) runs on one shared recognizer; never
        // pile up concurrent transcribes or the process runs out of memory.
        let transcribing = Arc::new(AtomicBool::new(false));

        let task = tokio::spawn(async move {
            info!("Starting audio processing pipeline loop");
            let mut current_pid: Option<u32> = None;
            let mut current_capture: Option<AudioCapture> = None;
            let mut vad = match VoiceActivityDetector::new(vad_threshold, min_speech_ms, min_silence_ms) {
                Ok(v) => v,
                Err(e) => {
                    error!("VAD initialization failed: {e}");
                    let _ = app_handle.emit(
                        "audio-status",
                        AudioStatusPayload {
                            is_active: false,
                            source_description: "VAD failed to initialize".into(),
                            active_pid: None,
                            active_app: None,
                            vad_speech_detected: false,
                            error: Some(e.to_string()),
                        },
                    );
                    is_running_flag.store(false, Ordering::SeqCst);
                    return;
                }
            };

            let mut speech_buffer: Vec<f32> = Vec::with_capacity(16000 * 15);
            let mut in_speech = false;
            let mut speech_seq_id: u64 = 1;
            let mut last_partial_time = Instant::now();
            let mut last_foreground_check = Instant::now();
            let mut vad_log_tick: u64 = 0;
            let mut silent_samples: usize = 0;
            // How much of speech_buffer the last partial already covered.
            // Partials only decode newly arrived audio (bounded work).
            let mut partial_decoded_len: usize = 0;

            // True-streaming path for sherpa: feed each chunk once, read live
            // partials, finalize on utterance end. Nothing is ever re-decoded.
            let sherpa_streaming =
                matches!(speech_provider.as_ref(), SpeechProviderKind::Sherpa(_));
            let mut stream_state = SherpaStreamState::new();
            if sherpa_streaming {
                // One-time model load + warmup off the capture loop.
                let wp = Arc::clone(&speech_provider);
                tokio::task::spawn_blocking(move || {
                    if let SpeechProviderKind::Sherpa(p) = wp.as_ref() {
                        p.warmup();
                    }
                });
            }

            while is_running_flag.load(Ordering::SeqCst) {
                // Check active foreground application every 500ms if in auto_foreground mode
                if audio_mode == "auto_foreground" && last_foreground_check.elapsed() > Duration::from_millis(500) {
                    last_foreground_check = Instant::now();
                    if let Some(app) = get_foreground_app() {
                        if Some(app.pid) != current_pid {
                            info!("Foreground switched to PID {}: {}", app.pid, app.process_name);
                            current_pid = Some(app.pid);
                            // Switch capture source
                            current_capture = match AudioCapture::new(AudioSource::Process(app.pid)) {
                                Ok(c) => Some(c),
                                Err(e) => {
                                    warn!("Per-process capture failed for PID {}, falling back to System Loopback: {e}", app.pid);
                                    AudioCapture::new(AudioSource::SystemLoopback).ok()
                                }
                            };
                            let _ = app_handle.emit(
                                "audio-status",
                                AudioStatusPayload {
                                    is_active: true,
                                    source_description: format!("Captured: {}", app.process_name),
                                    active_pid: Some(app.pid),
                                    active_app: Some(app.process_name),
                                    vad_speech_detected: in_speech,
                                    error: None,
                                },
                            );
                        }
                    }
                } else if current_capture.is_none() {
                    // Initialize first capture based on mode
                    let (source, desc, app_name) = match audio_mode.as_str() {
                        "pinned_process" => {
                            if let Some(pid) = pinned_pid {
                                (AudioSource::Process(pid), format!("Pinned PID: {pid}"), None)
                            } else {
                                (AudioSource::SystemLoopback, "System Audio Loopback".into(), None)
                            }
                        }
                        "system_loopback" => (AudioSource::SystemLoopback, "System Audio Loopback".into(), None),
                        _ => {
                            if let Some(app) = get_foreground_app() {
                                current_pid = Some(app.pid);
                                (AudioSource::Process(app.pid), format!("Active App: {}", app.process_name), Some(app.process_name))
                            } else {
                                (AudioSource::SystemLoopback, "System Audio Loopback".into(), None)
                            }
                        }
                    };

                    current_capture = match AudioCapture::new(source) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            error!("Audio capture error: {e}");
                            let _ = app_handle.emit(
                                "audio-status",
                                AudioStatusPayload {
                                    is_active: false,
                                    source_description: desc,
                                    active_pid: current_pid,
                                    active_app: app_name,
                                    vad_speech_detected: false,
                                    error: Some(e.to_string()),
                                },
                            );
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                            continue;
                        }
                    };
                }

                // Poll chunk from WASAPI stream
                let chunk = if let Some(capture) = current_capture.as_mut() {
                    capture.poll_chunk()
                } else {
                    None
                };

                    match chunk {
                        Some(samples) => {
                            let events = vad.process(&samples);
                            vad_log_tick += 1;
                            let last_prob = vad
                                .last_probabilities()
                                .iter()
                                .copied()
                                .next_back()
                                .unwrap_or(0.0);
                            if vad_log_tick % 600 == 0 {
                                log::debug!(
                                    "VAD tick #{} in_speech={} prob_last={:.4} events={:?}",
                                    vad_log_tick, in_speech, last_prob, events
                                );
                            }

                            // Speech is derived from the VAD's last-frame probability.
                            // flexaudio-vad only returns SpeechStart/SpeechEnd when the whole
                            // segment is confirmed, which is too late for streaming, so we gate
                            // on the live probability instead.
                            let speech_now = last_prob >= vad_threshold;
                            let mut speech_ended = false;
                            // Hangover: tolerate brief pauses so one utterance isn't
                            // chopped into sub-second fragments (sherpa returns
                            // empty text for those). Also require ~1s minimum.
                            let hangover_samples =
                                ((min_silence_ms.max(500) as usize) * 16000) / 1000;
                            const MIN_FINAL_SAMPLES: usize = 16000;

                            if sherpa_streaming {
                                // True-streaming path: each chunk decoded once.
                                if let SpeechProviderKind::Sherpa(p) =
                                    speech_provider.as_ref()
                                {
                                    streaming_tick(
                                        p,
                                        &app_handle,
                                        translation_provider.as_ref(),
                                        &target_lang,
                                        &samples,
                                        speech_now,
                                        &mut stream_state,
                                        &mut speech_seq_id,
                                        min_silence_ms,
                                    );
                                }
                            } else if speech_now {
                                in_speech = true;
                                silent_samples = 0;
                                speech_buffer.extend_from_slice(&samples);

                                if !speech_ended && speech_buffer.len() >= 16000 * 2 && last_partial_time.elapsed() > Duration::from_millis(2500) {
                                    // Differential partial: only decode audio that
                                    // arrived since the last partial, so the work
                                    // stays bounded instead of re-decoding a
                                    // growing buffer from scratch.
                                    let new_len = speech_buffer.len().saturating_sub(partial_decoded_len);
                                    if new_len < 16000 * 3 / 2 {
                                        // Less than 1.5s of new audio: wait.
                                    } else if transcribing.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                                        // A decode is still running; try again on
                                        // the next tick without losing anything.
                                    } else {
                                        last_partial_time = Instant::now();
                                        let tail = speech_buffer[partial_decoded_len..].to_vec();
                                        partial_decoded_len = speech_buffer.len();
                                        let provider = Arc::clone(&speech_provider);
                                        let lang = transcription_lang.clone();
                                        let handle_clone = app_handle.clone();
                                        let seq = speech_seq_id;
                                        let busy = Arc::clone(&transcribing);

                                        tokio::spawn(async move {
                                            let t0 = Instant::now();
                                            if let Ok(res) = provider.transcribe(&tail, lang.as_deref()).await {
                                                if !res.text.trim().is_empty() {
                                                    let _ = handle_clone.emit(
                                                        "transcription-event",
                                                        TranscriptEventPayload {
                                                            id: seq,
                                                            text: res.text,
                                                            translation: None,
                                                            is_final: false,
                                                            language: res.language,
                                                            latency_ms: Some(t0.elapsed().as_secs_f32() * 1000.0),
                                                        },
                                                    );
                                                }
                                            }
                                            busy.store(false, Ordering::SeqCst);
                                        });
                                    }
                                }

                                if speech_buffer.len() >= 16000 * 12 {
                                    in_speech = false;
                                    silent_samples = 0;
                                    speech_ended = true;
                                }
                            } else if in_speech {
                                // Hangover: keep accumulating through brief
                                // silence; only finalize after sustained quiet.
                                silent_samples += samples.len();
                                speech_buffer.extend_from_slice(&samples);
                                if silent_samples >= hangover_samples {
                                    in_speech = false;
                                    silent_samples = 0;
                                    if speech_buffer.len() >= MIN_FINAL_SAMPLES {
                                        speech_ended = true;
                                    } else {
                                        // Too short to transcribe: keep it so it
                                        // merges with the next speech burst.
                                        log::debug!(
                                            "Retaining short fragment ({}s)",
                                            speech_buffer.len() as f32 / 16000.0
                                        );
                                    }
                                } else if speech_buffer.len() > 16000 * 14 {
                                    // Safety cap including hangover audio.
                                    in_speech = false;
                                    silent_samples = 0;
                                    speech_ended = true;
                                }
                            } else if !speech_buffer.is_empty()
                                && speech_buffer.len() < MIN_FINAL_SAMPLES
                            {
                                // Accumulating onto a retained short fragment.
                                speech_buffer.extend_from_slice(&samples);
                                let max_keep = MIN_FINAL_SAMPLES + hangover_samples;
                                if speech_buffer.len() > max_keep {
                                    let excess = speech_buffer.len() - max_keep;
                                    speech_buffer.drain(0..excess);
                                }
                            } else {
                                // Keep a small rolling pre-speech buffer (200ms) to not clip initial word plosives
                                let pre_samples = 16000 / 5;
                                speech_buffer.extend_from_slice(&samples);
                                if speech_buffer.len() > pre_samples {
                                    let excess = speech_buffer.len() - pre_samples;
                                    speech_buffer.drain(0..excess);
                                }
                            }

                            if speech_ended {
                                let audio_to_process = std::mem::take(&mut speech_buffer);
                                partial_decoded_len = 0;
                                if audio_to_process.len() >= 16000 / 4 {
                                    // At least 250ms of speech. If a partial is
                                    // still decoding, keep the audio for the
                                    // next final attempt instead of dropping it.
                                    if transcribing.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                                        warn!("Transcription busy: retaining {}s for retry", audio_to_process.len() as f32 / 16000.0);
                                        let keep = audio_to_process.len().min(16000 * 12);
                                        speech_buffer = audio_to_process[audio_to_process.len() - keep..].to_vec();
                                    } else {
                                    // At least 250ms of speech
                                    let provider = Arc::clone(&speech_provider);
                                    let trans_provider = translation_provider.clone();
                                    let t_lang = target_lang.clone();
                                    let lang = transcription_lang.clone();
                                    let handle_clone = app_handle.clone();
                                    let seq = speech_seq_id;
                                    speech_seq_id += 1;
                                    let busy = Arc::clone(&transcribing);

                                    tokio::spawn(async move {
                                        let t0 = Instant::now();
                                        let res = provider.transcribe(&audio_to_process, lang.as_deref()).await;
                                        let latency_ms = Some(t0.elapsed().as_secs_f32() * 1000.0);
                                        // Free the decoder before the (slow)
                                        // cloud translation so the next segment
                                        // can start decoding immediately.
                                        busy.store(false, Ordering::SeqCst);
                                        match res {
                                            Ok(res) => {
                                                let text = res.text.trim().to_string();
                                                if !text.is_empty() {
                                                    let mut translation = None;
                                                    if let Some(tp) = trans_provider {
                                                        if let Ok(tres) = tp.translate(&text, &t_lang, res.language.as_deref()).await {
                                                            translation = Some(tres.translated_text);
                                                        }
                                                    }

                                                    let _ = handle_clone.emit(
                                                        "transcription-event",
                                                        TranscriptEventPayload {
                                                            id: seq,
                                                            text,
                                                            translation,
                                                            is_final: true,
                                                            language: res.language,
                                                            latency_ms,
                                                        },
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                error!("Speech transcription error: {e}");
                                                let _ = handle_clone.emit(
                                                    "audio-status",
                                                    AudioStatusPayload {
                                                        is_active: true,
                                                        source_description: "Active".into(),
                                                        active_pid: None,
                                                        active_app: None,
                                                        vad_speech_detected: false,
                                                        error: Some(e.to_string()),
                                                    },
                                                );
                                            }
                                        }
                                    });
                                    }
                                }
                            }
                        }
                        None => {
                            tokio::time::sleep(Duration::from_millis(15)).await;
                        }
                    }
            }

            if let Some(mut c) = current_capture {
                c.stop();
            }
            info!("Audio pipeline terminated gracefully");
        });

        let mut guard = self.handle.lock().await;
        *guard = Some(task);
    }

    pub async fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        let mut guard = self.handle.lock().await;
        if let Some(task) = guard.take() {
            let _ = task.await;
        }
    }
}
