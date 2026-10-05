pub mod audio;
pub mod commands;
pub mod credentials;
pub mod error;
pub mod models;
pub mod profiles;
pub mod settings;
pub mod transcription;
pub mod translation;
pub mod windows;

use commands::AppState;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use crate::windows::foreground::get_foreground_app;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let settings = Arc::new(settings::SettingsManager::new());
    let profiles = Arc::new(profiles::ProfileManager::new());
    let models = Arc::new(models::ModelManager::new());
    let pipeline = Arc::new(audio::pipeline::AudioPipeline::new());

    let app_state = AppState {
        settings,
        profiles,
        models,
        pipeline,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_api_key,
            commands::set_api_key,
            commands::test_groq_connection,
            commands::list_models,
            commands::download_model,
            commands::delete_model,
            commands::get_profile,
            commands::save_profile,
            commands::list_profiles,
            commands::delete_profile,
            commands::get_current_foreground_app,
            commands::set_click_through,
            commands::save_overlay_bounds,
            commands::start_transcription,
            commands::stop_transcription,
            commands::is_transcribing,
            commands::close_window,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Background task: Track active foreground application changes
            tauri::async_runtime::spawn(async move {
                let mut last_pid: Option<u32> = None;
                loop {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    if let Some(app_info) = get_foreground_app() {
                        if Some(app_info.pid) != last_pid {
                            last_pid = Some(app_info.pid);
                            let _ = handle.emit("active-app-changed", app_info);
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Talker application");
}
