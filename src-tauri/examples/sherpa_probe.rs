//! Isolated probe: create a sherpa-online recognizer with explicit files,
//! feed it silence + a real test wav, print results. Run with:
//!   cargo run --example sherpa_probe --no-default-features -- <model_dir>
use sherpa_onnx::{LinearResampler, OnlineRecognizer, OnlineRecognizerConfig, Wave};

fn build_config(dir: &str, encoder: &str, decoder: &str, joiner: &str) -> OnlineRecognizerConfig {
    let mut config = OnlineRecognizerConfig::default();
    config.model_config.transducer.encoder = Some(format!("{dir}/{encoder}"));
    config.model_config.transducer.decoder = Some(format!("{dir}/{decoder}"));
    config.model_config.transducer.joiner = Some(format!("{dir}/{joiner}"));
    config.model_config.tokens = Some(format!("{dir}/tokens.txt"));
    config.model_config.num_threads = 1;
    config.decoding_method = Some("greedy_search".to_string());
    config.enable_endpoint = true;
    config
}

fn decode_all(recognizer: &OnlineRecognizer, sample_rate: i32, pcm: &[f32]) -> String {
    let stream = recognizer.create_stream();
    stream.accept_waveform(sample_rate, pcm);
    stream.input_finished();
    let mut steps = 0;
    while recognizer.is_ready(&stream) {
        recognizer.decode(&stream);
        steps += 1;
        if steps > 10_000 {
            break;
        }
    }
    let text = recognizer
        .get_result(&stream)
        .map(|r| r.text)
        .unwrap_or_default();
    eprintln!("decode steps: {steps}");
    text
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: sherpa_probe <model_dir> [encoder] [decoder] [joiner]");
    let enc = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "encoder-epoch-99-avg-1.onnx".into());
    let dec = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "decoder-epoch-99-avg-1.onnx".into());
    let joi = std::env::args()
        .nth(4)
        .unwrap_or_else(|| "joiner-epoch-99-avg-1.onnx".into());

    // 1) fp32 trio
    eprintln!("=== fp32 trio ===");
    let config = build_config(&dir, &enc, &dec, &joi);
    let recognizer = OnlineRecognizer::create(&config).expect("create fp32 recognizer");
    eprintln!("recognizer created OK");
    let silence = vec![0.0f32; 16000];
    let text = decode_all(&recognizer, 16000, &silence);
    eprintln!("silence result: {text:?}");

    // 1b) official mixed trio: int8 encoder/joiner + fp32 decoder
    // (skipped when file names were given explicitly)
    if std::env::args().nth(2).is_none() {
        eprintln!("=== mixed int8 trio ===");
        let config = build_config(
            &dir,
            "encoder-epoch-99-avg-1.int8.onnx",
            "decoder-epoch-99-avg-1.onnx",
            "joiner-epoch-99-avg-1.int8.onnx",
        );
        match OnlineRecognizer::create(&config) {
            Some(r) => {
                eprintln!("mixed recognizer created OK");
                let text = decode_all(&r, 16000, &silence);
                eprintln!("mixed silence result: {text:?}");
            }
            None => eprintln!("mixed recognizer FAILED"),
        }
    }

    // 2) real test wav (16k): test_wavs/ dir or test-0.wav in model root
    let mut wavs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(format!("{dir}/test_wavs")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "wav").unwrap_or(false) {
                wavs.push(path);
            }
        }
    }
    let root_wav = std::path::PathBuf::from(format!("{dir}/test-0.wav"));
    if root_wav.exists() {
        wavs.push(root_wav);
    }
    wavs.sort();
    for path in wavs.into_iter().take(1) {
        eprintln!("=== wav: {} ===", path.display());
        if let Some(wave) = Wave::read(path.to_str().expect("utf8")) {
            eprintln!("samples: {} @ {}", wave.samples().len(), wave.sample_rate());
            let (rate, samples) = if wave.sample_rate() == 16000 {
                (16000, wave.samples().to_vec())
            } else {
                let rs = LinearResampler::create(wave.sample_rate(), 16000).expect("resampler");
                (16000, rs.resample(wave.samples(), true))
            };
            eprintln!("resampled to {} samples @ 16k", samples.len());
            let text = decode_all(&recognizer, rate, &samples);
            eprintln!("result: {text:?}");
        }
    }
    eprintln!("PROBE DONE");
}
