use crate::audio::pipeline::AudioPipeline;
use crate::credentials::{delete_secret, get_secret, set_secret};
use crate::error::{AppError, AppResult};
use crate::models::{ModelInfo, ModelManager};
use crate::profiles::{AppProfile, ProfileManager};
use crate::settings::{AppSettings, SettingsManager};
use crate::transcription::groq::GroqSpeechProvider;
use crate::transcription::provider::SpeechProviderKind;
use crate::transcription::whisper_local::LocalWhisperProvider;
use crate::transcription::sherpa::SherpaStreamingProvider;
use crate::translation::cloud::CloudTranslationProvider;
use crate::translation::provider::TranslationProviderKind;
use crate::windows::foreground::{get_foreground_app, ActiveAppInfo};
use crate::windows::window_control;
use std::sync::Arc;
use tauri::{AppHandle, State, WebviewWindow};

pub struct AppState {
    pub settings: Arc<SettingsManager>,
    pub profiles: Arc<ProfileManager>,
    pub models: Arc<ModelManager>,
    pub pipeline: Arc<AudioPipeline>,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppSettings {
    state.settings.get()
}

#[tauri::command]
pub fn save_settings(new_settings: AppSettings, state: State<AppState>) -> AppResult<()> {
    state.settings.save(new_settings)
}

#[tauri::command]
pub fn get_api_key(provider: String) -> AppResult<Option<String>> {
    get_secret(&format!("api_key_{provider}"))
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> AppResult<()> {
    if key.trim().is_empty() {
        delete_secret(&format!("api_key_{provider}"))
    } else {
        set_secret(&format!("api_key_{provider}"), key.trim())
    }
}

#[tauri::command]
pub async fn test_groq_connection(api_key: String) -> AppResult<String> {
    if api_key.trim().is_empty() {
        return Err(AppError::Credential("API key cannot be empty".into()));
    }
    let provider = GroqSpeechProvider::new(api_key, "whisper-large-v3-turbo".to_string());
    provider.test_connection().await
}

#[tauri::command]
pub fn list_models(state: State<AppState>) -> Vec<ModelInfo> {
    state.models.list_models()
}

#[tauri::command]
pub async fn download_model(
    model_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    state.models.download_model(&model_id, app_handle).await
}

#[tauri::command]
pub fn delete_model(model_id: String, state: State<AppState>) -> AppResult<()> {
    state.models.delete_model(&model_id)
}

#[tauri::command]
pub fn get_profile(process_name: String, state: State<AppState>) -> Option<AppProfile> {
    state.profiles.get(&process_name)
}

#[tauri::command]
pub fn save_profile(profile: AppProfile, state: State<AppState>) -> AppResult<()> {
    state.profiles.save(profile)
}

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> Vec<AppProfile> {
    state.profiles.list()
}

#[tauri::command]
pub fn delete_profile(process_name: String, state: State<AppState>) -> AppResult<()> {
    state.profiles.delete(&process_name)
}

#[tauri::command]
pub fn get_current_foreground_app() -> Option<ActiveAppInfo> {
    get_foreground_app()
}

#[tauri::command]
pub fn set_click_through(ignore: bool, window: WebviewWindow) -> AppResult<()> {
    window_control::set_click_through(&window, ignore)
}

#[tauri::command]
pub fn save_overlay_bounds(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    process_name: Option<String>,
    state: State<AppState>,
) -> AppResult<()> {
    let mut current_settings = state.settings.get();
    current_settings.overlay_width = width;
    current_settings.overlay_height = height;
    let _ = state.settings.save(current_settings);

    if let Some(proc) = process_name {
        if !proc.is_empty() {
            let profile = AppProfile {
                process_name: proc,
                x,
                y,
                width,
                height,
                font_size: state.settings.get().overlay_font_size,
                opacity: state.settings.get().overlay_opacity,
                translation_enabled: state.settings.get().translation_enabled,
                target_language: Some(state.settings.get().translation_target_lang),
            };
            let _ = state.profiles.save(profile);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn start_transcription(app_handle: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let settings = state.settings.get();

    // 1. Build speech provider (concrete enum, avoids dyn async fn incompatibility)
    let speech_provider: SpeechProviderKind = match settings.transcription_provider.as_str() {
        "local" => {
            let model_dir = state.models.get_model_dir(&settings.local_model_name);
            if !state.models.is_model_installed(&settings.local_model_name) {
                return Err(AppError::Model(format!(
                    "The selected local model '{}' is not installed. Please download it in Settings.",
                    settings.local_model_name
                )));
            }
            SpeechProviderKind::Local(LocalWhisperProvider::new(model_dir))
        }
        "sherpa" => {
            let model_dir = state.models.get_model_dir(&settings.sherpa_model_name);
            if !state.models.is_model_installed(&settings.sherpa_model_name) {
                return Err(AppError::Model(format!(
                    "The Sherpa model '{}' is not installed. Download it in Local Models.",
                    settings.sherpa_model_name
                )));
            }
            SpeechProviderKind::Sherpa(SherpaStreamingProvider::new(model_dir))
        }
        _ => {
            // Groq
            let key = get_secret("api_key_groq")?
                .ok_or_else(|| AppError::Credential("Groq API key not configured. Please enter it in Settings.".into()))?;
            if key.trim().is_empty() {
                return Err(AppError::Credential("Groq API key is empty. Please enter it in Settings.".into()));
            }
            SpeechProviderKind::Groq(GroqSpeechProvider::new(key, settings.groq_model_name.clone()))
        }
    };

    // 2. Build translation provider if enabled (concrete enum)
    let translation_provider: Option<TranslationProviderKind> = if settings.translation_enabled {
        let prov_name = settings.translation_provider.as_str();
        let key = get_secret(&format!("api_key_{prov_name}"))?
            .ok_or_else(|| AppError::Credential(format!("{prov_name} API key not configured for translation.")))?;

        match prov_name {
            "openai" => Some(TranslationProviderKind::Cloud(
                CloudTranslationProvider::openai(key, settings.translation_model.clone()),
            )),
            "openrouter" => Some(TranslationProviderKind::Cloud(
                CloudTranslationProvider::openrouter(key, settings.translation_model.clone()),
            )),
            _ => Some(TranslationProviderKind::Cloud(
                CloudTranslationProvider::groq(key, settings.translation_model.clone()),
            )),
        }
    } else {
        None
    };

    state.pipeline.start(
        app_handle,
        settings.audio_mode,
        settings.pinned_pid,
        speech_provider,
        translation_provider,
        settings.translation_target_lang,
        settings.vad_threshold,
        settings.vad_min_speech_ms,
        settings.vad_min_silence_ms,
        settings.transcription_language,
    ).await;

    Ok(())
}

#[tauri::command]
pub async fn stop_transcription(state: State<'_, AppState>) -> AppResult<()> {
    state.pipeline.stop().await;
    Ok(())
}

#[tauri::command]
pub fn is_transcribing(state: State<AppState>) -> bool {
    state.pipeline.is_running()
}

#[tauri::command]
pub fn close_window(window: WebviewWindow) -> AppResult<()> {
    window
        .close()
        .map_err(|e| AppError::Window(format!("Failed to close window: {e}")))
}
