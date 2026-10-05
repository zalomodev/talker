use crate::error::{AppError, AppResult};
use crate::profiles::dirs_profile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

fn default_sherpa_model() -> String {
    "sherpa-onnx-streaming-zipformer-en-2023-06-26".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub audio_mode: String,
    pub pinned_pid: Option<u32>,
    pub transcription_provider: String,
    pub local_model_name: String,
    pub groq_model_name: String,
    #[serde(default = "default_sherpa_model")]
    pub sherpa_model_name: String,
    pub transcription_language: Option<String>,
    pub vad_threshold: f32,
    pub vad_min_speech_ms: u32,
    pub vad_min_silence_ms: u32,
    pub translation_enabled: bool,
    pub translation_provider: String,
    pub translation_model: String,
    pub translation_target_lang: String,
    pub overlay_font_size: u32,
    pub overlay_opacity: f32,
    pub overlay_reduced_motion: bool,
    #[serde(default = "default_true")]
    pub overlay_show_tempo: bool,
    pub overlay_width: u32,
    pub overlay_height: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            audio_mode: "auto_foreground".to_string(),
            pinned_pid: None,
            transcription_provider: "groq".to_string(),
            local_model_name: "openai/whisper-tiny".to_string(),
            groq_model_name: "whisper-large-v3-turbo".to_string(),
            sherpa_model_name: default_sherpa_model(),
            transcription_language: None, // Auto-detect
            vad_threshold: 0.5,
            vad_min_speech_ms: 250,
            vad_min_silence_ms: 300,
            translation_enabled: false,
            translation_provider: "groq".to_string(),
            translation_model: "openai/gpt-oss-120b".to_string(),
            translation_target_lang: "English".to_string(),
            overlay_font_size: 18,
            overlay_opacity: 0.95,
            overlay_reduced_motion: false,
            overlay_show_tempo: true,
            overlay_width: 580,
            overlay_height: 220,
        }
    }
}

pub struct SettingsManager {
    file_path: PathBuf,
    settings: Mutex<AppSettings>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let base_dir = dirs_profile();
        let file_path = base_dir.join("settings.json");
        let settings = Self::load_from_disk(&file_path).unwrap_or_default();

        Self {
            file_path,
            settings: Mutex::new(settings),
        }
    }

    fn load_from_disk(path: &PathBuf) -> Option<AppSettings> {
        if !path.exists() {
            return None;
        }
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn get(&self) -> AppSettings {
        let guard = self.settings.lock().unwrap();
        guard.clone()
    }

    pub fn save(&self, new_settings: AppSettings) -> AppResult<()> {
        {
            let mut guard = self.settings.lock().unwrap();
            *guard = new_settings.clone();
        }
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&new_settings)
            .map_err(|e| AppError::Settings(format!("Failed to serialize settings: {e}")))?;
        fs::write(&self.file_path, json)
            .map_err(|e| AppError::Io(format!("Failed to write settings to disk: {e}")))?;
        Ok(())
    }
}
