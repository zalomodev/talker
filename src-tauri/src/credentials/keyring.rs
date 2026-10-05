use crate::error::{AppError, AppResult};
use crate::profiles::dirs_profile;
use keyring::Entry;
use std::fs;
use std::path::PathBuf;

pub const SERVICE_NAME: &str = "Talker-App";

fn keys_file() -> PathBuf {
    let base = dirs_profile();
    if let Some(parent) = base.parent() {
        let _ = parent;
    }
    let _ = fs::create_dir_all(&base);
    base.join("api_keys.json")
}

fn load_keys_file() -> std::collections::HashMap<String, String> {
    let path = keys_file();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    }
}

fn write_keys_file(map: &std::collections::HashMap<String, String>) {
    if let Ok(json) = serde_json::to_string_pretty(map) {
        let _ = fs::write(keys_file(), json);
    }
}

pub fn get_secret(key: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| AppError::Credential(format!("Failed to initialize keyring entry for {key}: {e}")))?;

    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => {
            Ok(load_keys_file().get(key).cloned())
        }
        Err(e) => Err(AppError::Credential(format!("Failed to retrieve credential for {key}: {e}"))),
    }
}

pub fn set_secret(key: &str, secret: &str) -> AppResult<()> {
    match Entry::new(SERVICE_NAME, key) {
        Ok(entry) => {
            let _ = entry.set_password(secret);
        }
        Err(_) => {}
    }

    let mut map = load_keys_file();
    map.insert(key.to_string(), secret.to_string());
    write_keys_file(&map);

    Ok(())
}

pub fn delete_secret(key: &str) -> AppResult<()> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, key) {
        let _ = entry.delete_credential();
    }
    let mut map = load_keys_file();
    map.remove(key);
    write_keys_file(&map);
    Ok(())
}
