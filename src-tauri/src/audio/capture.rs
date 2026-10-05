use crate::error::{AppError, AppResult};
use flexaudio::{open, OutputFormat, ProcessMode, SourceKind, Stream, StreamConfig};
use log::info;

pub enum AudioSource {
    Process(u32),
    SystemLoopback,
}

pub struct AudioCapture {
    stream: Stream,
    source_desc: String,
}

impl AudioCapture {
    pub fn new(source: AudioSource) -> AppResult<Self> {
        let output = OutputFormat {
            sample_rate: 16000,
            channels: 1, // mono 16kHz for VAD and Whisper
        };

        let (config, desc) = match source {
            AudioSource::Process(pid) => {
                let config = StreamConfig {
                    kind: SourceKind::ProcessLoopback,
                    target_pid: Some(pid),
                    mode: ProcessMode::Include,
                    output,
                    ..Default::default()
                };
                (config, format!("Process PID: {pid}"))
            }
            AudioSource::SystemLoopback => {
                let config = StreamConfig {
                    kind: SourceKind::SystemLoopback,
                    exclude_self: true,
                    output,
                    ..Default::default()
                };
                (config, "Windows System Audio Loopback".to_string())
            }
        };

        info!("Opening audio capture stream for {desc}");
        let mut stream = open(config)
            .map_err(|e| AppError::Audio(format!("Failed to open WASAPI capture for {desc}: {e}")))?;

        stream
            .start()
            .map_err(|e| AppError::Audio(format!("Failed to start WASAPI stream for {desc}: {e}")))?;

        Ok(Self {
            stream,
            source_desc: desc,
        })
    }

    pub fn poll_chunk(&mut self) -> Option<Vec<f32>> {
        self.stream.poll_chunk().map(|chunk| chunk.data)
    }

    pub fn stop(&mut self) {
        info!("Stopping audio capture stream for {}", self.source_desc);
        self.stream.stop();
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}
