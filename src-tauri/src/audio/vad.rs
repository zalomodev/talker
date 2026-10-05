use crate::error::{AppResult};

#[derive(Debug)]
pub enum VadEvent {
    SpeechStart { at_sample: usize },
    SpeechEnd { at_sample: usize },
}

/// Lightweight energy-based VAD used instead of the removed
/// `flexaudio-vad`/`ort-sys` pair, which conflicts with the ONNX
/// runtime bundled by sherpa-onnx.
pub struct VoiceActivityDetector {
    threshold: f32,
    last_probs: Vec<f32>,
    in_speech: bool,
    accumulate_offset: usize,
}

impl VoiceActivityDetector {
    pub fn new(threshold: f32, _min_speech_ms: u32, _min_silence_ms: u32) -> AppResult<Self> {
        Ok(Self {
            threshold,
            last_probs: Vec::new(),
            in_speech: false,
            accumulate_offset: 0,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        16_000
    }

    pub fn process(&mut self, samples: &[f32]) -> Vec<VadEvent> {
        let rms = (samples.iter().map(|x| x * x).sum::<f32>() / samples.len().max(1) as f32)
            .sqrt();
        let prob = (rms * 6.0).clamp(0.0, 1.0);
        self.last_probs = vec![prob];
        self.accumulate_offset += samples.len();

        let mut events = Vec::new();
        let speaking = prob >= self.threshold;
        if speaking && !self.in_speech {
            self.in_speech = true;
            events.push(VadEvent::SpeechStart { at_sample: self.accumulate_offset });
        } else if !speaking && self.in_speech {
            self.in_speech = false;
            events.push(VadEvent::SpeechEnd { at_sample: self.accumulate_offset });
        }
        events
    }

    pub fn last_probabilities(&self) -> &[f32] {
        &self.last_probs
    }

    pub fn reset(&mut self) {
        self.in_speech = false;
        self.last_probs.clear();
        self.accumulate_offset = 0;
    }
}
