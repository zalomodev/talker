pub mod manager;

pub use manager::*;


pub fn find_model_file(dir: &std::path::PathBuf, keyword: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(keyword) && name.ends_with(".onnx") {
                return Some(entry.path());
            }
            if entry.path().is_dir() {
                if let Some(found) = find_model_file(&entry.path(), keyword) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub fn find_tokens_file(dir: &std::path::PathBuf) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name == "tokens.txt" || name.ends_with("bpe.model")) && entry.path().is_file() {
                return Some(entry.path());
            }
            if entry.path().is_dir() {
                if let Some(found) = find_tokens_file(&entry.path()) {
                    return Some(found);
                }
            }
        }
    }
    None
}