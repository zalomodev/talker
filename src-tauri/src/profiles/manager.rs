use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProfile {
    pub process_name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub font_size: u32,
    pub opacity: f32,
    pub translation_enabled: bool,
    pub target_language: Option<String>,
}

pub struct ProfileManager {
    file_path: PathBuf,
    profiles: Mutex<HashMap<String, AppProfile>>,
}

impl ProfileManager {
    pub fn new() -> Self {
        let base_dir = dirs_profile();
        let file_path = base_dir.join("profiles.json");
        let profiles = Self::load_from_disk(&file_path).unwrap_or_default();

        Self {
            file_path,
            profiles: Mutex::new(profiles),
        }
    }

    fn load_from_disk(path: &PathBuf) -> Option<HashMap<String, AppProfile>> {
        if !path.exists() {
            return None;
        }
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    fn persist(&self) -> AppResult<()> {
        let guard = self.profiles.lock().unwrap();
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&*guard)
            .map_err(|e| AppError::Settings(format!("Failed to serialize profiles: {e}")))?;
        fs::write(&self.file_path, json)
            .map_err(|e| AppError::Io(format!("Failed to write profiles to disk: {e}")))?;
        Ok(())
    }

    pub fn get(&self, process_name: &str) -> Option<AppProfile> {
        let guard = self.profiles.lock().unwrap();
        guard.get(process_name).cloned()
    }

    pub fn save(&self, profile: AppProfile) -> AppResult<()> {
        {
            let mut guard = self.profiles.lock().unwrap();
            guard.insert(profile.process_name.clone(), profile);
        }
        self.persist()
    }

    pub fn list(&self) -> Vec<AppProfile> {
        let guard = self.profiles.lock().unwrap();
        guard.values().cloned().collect()
    }

    pub fn delete(&self, process_name: &str) -> AppResult<()> {
        {
            let mut guard = self.profiles.lock().unwrap();
            guard.remove(process_name);
        }
        self.persist()
    }
}

pub fn dirs_profile() -> PathBuf {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        PathBuf::from(app_data).join("Talker")
    } else {
        PathBuf::from("./data")
    }
}
