use crate::error::{AppError, AppResult};
use crate::profiles::dirs_profile;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub estimated_size_bytes: u64,
    pub installed: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f32,
    pub finished: bool,
    pub error: Option<String>,
}

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Self {
        let base = dirs_profile();
        let models_dir = base.join("models");
        let _ = fs::create_dir_all(&models_dir);
        Self { models_dir }
    }

    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }

    pub fn get_model_dir(&self, model_id: &str) -> PathBuf {
        let sanitized = model_id.replace('/', "_");
        self.models_dir.join(sanitized)
    }

    pub fn is_model_installed(&self, model_id: &str) -> bool {
        let dir = self.get_model_dir(model_id);
        if model_id.starts_with("sherpa-onnx") {
            if !dir.exists() {
                return false;
            }
            return super::find_model_file(&dir, "encoder").is_some()
                && super::find_model_file(&dir, "decoder").is_some()
                && super::find_model_file(&dir, "joiner").is_some()
                && super::find_tokens_file(&dir).is_some();
        }

        let safetensors = dir.join("model.safetensors");
        let config = dir.join("config.json");
        let tokenizer = dir.join("tokenizer.json");

        safetensors.exists()
            && config.exists()
            && tokenizer.exists()
            && safetensors.metadata().map(|m| m.len() > 10_000).unwrap_or(false)
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        let known = vec![
            (
                "sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06",
                "Sherpa Zipformer Spanish",
                "Spanish streaming, best for Spanish speech.",
                160_000_000u64,
            ),
            (
                "sherpa-onnx-streaming-zipformer-en-20M-2023-02-17",
                "Sherpa Zipformer Small",
                "Tiniest CPU model, ultra-low latency streaming.",
                60_000_000u64,
            ),
            (
                "sherpa-onnx-streaming-zipformer-en-2023-06-26",
                "Sherpa Zipformer Decent",
                "Balanced streaming accuracy and size.",
                230_000_000u64,
            ),
            (
                "sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20",
                "Sherpa Zipformer Bilingual (Large)",
                "English + Chinese, highest quality of the three.",
                600_000_000u64,
            ),
        ];

        known
            .into_iter()
            .map(|(id, name, desc, size)| {
                let installed = self.is_model_installed(id);
                let path = if installed {
                    Some(self.get_model_dir(id).to_string_lossy().to_string())
                } else {
                    None
                };
                ModelInfo {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: desc.to_string(),
                    estimated_size_bytes: size,
                    installed,
                    path,
                }
            })
            .collect()
    }

    pub async fn download_model(
        &self,
        model_id: &str,
        app_handle: AppHandle,
    ) -> AppResult<()> {
        let target_dir = self.get_model_dir(model_id);
        fs::create_dir_all(&target_dir)
            .map_err(|e| AppError::Io(format!("Failed to create model directory: {e}")))?;

        if model_id.starts_with("sherpa-onnx") {
            // HuggingFace-hosted models ship as loose files, not a tarball.
            if model_id == "sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06" {
                return self
                    .download_sherpa_hf_files(
                        model_id,
                        "csukuangfj/sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06",
                        &["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"],
                        app_handle,
                    )
                    .await;
            }
            return self
                .download_sherpa_model(model_id, app_handle)
                .await;
        }

        let files = [
            ("model.safetensors", format!("https://huggingface.co/{model_id}/resolve/main/model.safetensors")),
            ("config.json", format!("https://huggingface.co/{model_id}/resolve/main/config.json")),
            ("tokenizer.json", format!("https://huggingface.co/{model_id}/resolve/main/tokenizer.json")),
        ];

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| AppError::Model(format!("Failed to create HTTP client: {e}")))?;

        for (filename, url) in files {
            let part_path = target_dir.join(format!("{filename}.part"));
            let final_path = target_dir.join(filename);

            if final_path.exists() {
                continue;
            }

            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| AppError::Model(format!("Failed to connect to Hugging Face for {filename}: {e}")))?;

            if !resp.status().is_success() {
                return Err(AppError::Model(format!(
                    "Download returned status {} for {url}",
                    resp.status()
                )));
            }

            let total_size = resp.content_length().unwrap_or(0);
            let mut downloaded: u64 = 0;
            let mut file = File::create(&part_path)
                .map_err(|e| AppError::Io(format!("Failed to create part file {part_path:?}: {e}")))?;

            let mut stream = resp.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result
                    .map_err(|e| AppError::Model(format!("Network error during {filename} download: {e}")))?;

                file.write_all(&chunk)
                    .map_err(|e| AppError::Io(format!("Failed writing chunk to disk: {e}")))?;

                downloaded += chunk.len() as u64;

                let percent = if total_size > 0 {
                    (downloaded as f32 / total_size as f32) * 100.0
                } else {
                    0.0
                };

                let _ = app_handle.emit(
                    "model-download-progress",
                    DownloadProgress {
                        model_id: model_id.to_string(),
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        percent,
                        finished: false,
                        error: None,
                    },
                );
            }

            file.flush()
                .map_err(|e| AppError::Io(format!("Failed to flush {filename}: {e}")))?;
            drop(file);

            // Atomic rename from .part to final
            fs::rename(&part_path, &final_path)
                .map_err(|e| AppError::Io(format!("Failed finalizing {filename}: {e}")))?;
        }

        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model_id: model_id.to_string(),
                downloaded_bytes: 1,
                total_bytes: 1,
                percent: 100.0,
                finished: true,
                error: None,
            },
        );

        Ok(())
    }

    /// Download loose-file sherpa models from HuggingFace (no tarball).
    async fn download_sherpa_hf_files(
        &self,
        model_id: &str,
        hf_repo: &str,
        files: &[&str],
        app_handle: AppHandle,
    ) -> AppResult<()> {
        let target_dir = self.get_model_dir(model_id);
        fs::create_dir_all(&target_dir)
            .map_err(|e| AppError::Io(format!("Failed to create model directory: {e}")))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(1800))
            .build()
            .map_err(|e| AppError::Model(format!("Failed to create HTTP client: {e}")))?;

        let total_files = files.len() as u64;
        for (idx, filename) in files.iter().enumerate() {
            let final_path = target_dir.join(filename);
            if final_path.exists() {
                continue;
            }
            let url = format!("https://huggingface.co/{hf_repo}/resolve/main/{filename}");
            let part_path = target_dir.join(format!("{filename}.part"));

            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| AppError::Model(format!("Failed to download {filename}: {e}")))?;
            if !resp.status().is_success() {
                return Err(AppError::Model(format!(
                    "Download failed HTTP {} for {url}",
                    resp.status()
                )));
            }

            let total_size = resp.content_length().unwrap_or(0);
            let mut downloaded: u64 = 0;
            let mut file = File::create(&part_path)
                .map_err(|e| AppError::Io(format!("Failed to create part file: {e}")))?;
            let mut stream = resp.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result
                    .map_err(|e| AppError::Model(format!("Network error downloading {filename}: {e}")))?;
                file.write_all(&chunk)
                    .map_err(|e| AppError::Io(format!("Failed to write download: {e}")))?;
                downloaded += chunk.len() as u64;
                // Progress across all files.
                let overall = if total_size > 0 {
                    ((idx as f32 + downloaded as f32 / total_size as f32) / total_files as f32) * 100.0
                } else {
                    0.0
                };
                let _ = app_handle.emit(
                    "model-download-progress",
                    DownloadProgress {
                        model_id: model_id.to_string(),
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        percent: overall,
                        finished: false,
                        error: None,
                    },
                );
            }
            file.flush().ok();
            drop(file);
            fs::rename(&part_path, &final_path)
                .map_err(|e| AppError::Io(format!("Failed finalizing {filename}: {e}")))?;
        }

        if !self.is_model_installed(model_id) {
            let _ = fs::remove_dir_all(&target_dir);
            return Err(AppError::Model(format!(
                "Download incomplete for {model_id}. Please try again."
            )));
        }

        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model_id: model_id.to_string(),
                downloaded_bytes: 1,
                total_bytes: 1,
                percent: 100.0,
                finished: true,
                error: None,
            },
        );

        Ok(())
    }

    async fn download_sherpa_model(
        &self,
        model_id: &str,
        app_handle: AppHandle,
    ) -> AppResult<()> {
        let target_dir = self.get_model_dir(model_id);
        fs::create_dir_all(&target_dir)
            .map_err(|e| AppError::Io(format!("Failed to create model directory: {e}")))?;

        let url = format!(
            "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/{}.tar.bz2",
            model_id
        );

        let tmp = self.models_dir.join(format!("{model_id}.tar.bz2"));

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(1800))
            .build()
            .map_err(|e| AppError::Model(format!("Failed to create HTTP client: {e}")))?;

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Model(format!("Failed to download {model_id}: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::Model(format!(
                "Download failed HTTP {} for {}",
                resp.status(),
                url
            )));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut file = File::create(&tmp)
            .map_err(|e| AppError::Io(format!("Failed to create temp file: {e}")))?;

        let mut stream = resp.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| AppError::Model(format!("Network error downloading {url}: {e}")))?;
            file.write_all(&chunk)
                .map_err(|e| AppError::Io(format!("Failed to write download: {e}")))?;
            downloaded += chunk.len() as u64;
            let percent = if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };
            let _ = app_handle.emit(
                "model-download-progress",
                DownloadProgress {
                    model_id: model_id.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes: total_size,
                    percent,
                    finished: false,
                    error: None,
                },
            );
        }
        file.flush().ok();
        drop(file);

        let tar_bz2 = fs::read(&tmp)
            .map_err(|e| AppError::Io(format!("Failed to read downloaded archive: {e}")))?;
        let bz = bzip2::read::BzDecoder::new(&tar_bz2[..]);
        let mut archive = tar::Archive::new(bz);
        archive
            .unpack(&self.models_dir)
            .map_err(|e| AppError::Model(format!("Failed to extract model archive: {e}")))?;

        // The process can die mid-extraction (or the download can be cut),
        // leaving a partial install behind. Verify completeness and clean up
        // so the next attempt starts fresh.
        if !self.is_model_installed(model_id) {
            let _ = fs::remove_dir_all(&target_dir);
            return Err(AppError::Model(format!(
                "Extraction incomplete for {model_id}: missing encoder/decoder/joiner/tokens. Please download again."
            )));
        }

        let _ = fs::remove_file(&tmp);

        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model_id: model_id.to_string(),
                downloaded_bytes: 1,
                total_bytes: 1,
                percent: 100.0,
                finished: true,
                error: None,
            },
        );

        Ok(())
    }

    pub fn delete_model(&self, model_id: &str) -> AppResult<()> {
        let dir = self.get_model_dir(model_id);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .map_err(|e| AppError::Io(format!("Failed to remove model directory: {e}")))?;
        }
        Ok(())
    }
}
