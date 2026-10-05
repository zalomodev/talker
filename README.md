# Talker

> [Español](README.es.md) · [English](README.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [Português (BR)](README.pt-BR.md)

A Windows overlay that **transcribes live** audio from your apps and videos, and **translates** it in real time.

## What it does

- 🎙️ Real-time transcription over the video or app you are watching.
- 🌍 Automatic AI translation (e.g. Spanish → English).
- 💻 100% local with [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (ES/EN streaming models), or cloud via Groq.
- 🪟 Minimal floating overlay: live typewriter, detected language, transcription latency.
- 🎯 Captures system sound or follows the active app.
- 📦 Models download themselves from inside the app (Models tab).

## Install

1. Download the `.msi` or `.exe` from the **Releases** tab.
2. Install / run, open Settings → Speech → pick **Sherpa (real-time, local)**.
3. Under Models, download **Sherpa Spanish** (or your language).
4. Toggle **ON** and play your video.

## Included models

| Model | Language | Source |
|---|---|---|
| `sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06` | Spanish | Hugging Face |
| `sherpa-onnx-streaming-zipformer-en-20M-2023-02-17` | English | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | English | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | Chinese + English | GitHub Releases |

## Build from source

Requirements: Windows, [Rust](https://rustup.rs/) (MSVC), Node 20+ and `pnpm`.

```bash
pnpm install
pnpm tauri dev     # development mode
pnpm tauri build   # produces .exe + .msi
```

## License

Open project under the **Apache License 2.0** — do whatever you want with it: use, modify, sell, no permission needed. See [LICENSE](LICENSE).
