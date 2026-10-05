# Talker

> [Español](README.es.md) · [English](README.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [Português (BR)](README.pt-BR.md)

Windows 悬浮窗，**实时转录**你正在观看的应用 / 视频音频，并**实时翻译**。

## 功能

- 🎙️ 在视频或应用上实时显示转录文本。
- 🌍 AI 自动翻译（例如：西班牙语 → 英语）。
- 💻 使用 [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) 100% 本地运行（西 / 英流式模型），也支持 Groq 云端。
- 🪟 极简悬浮窗：实时打字机效果、检测到的语言、转录耗时。
- 🎯 采集系统声音，或跟随当前活动应用。
- 📦 模型在应用内一键下载（Models 页面）。

## 安装

1. 从 **Releases** 页面下载 `.msi` 或 `.exe`。
2. 安装 / 运行，打开 Settings → Speech → 选择 **Sherpa (real-time, local)**。
3. 在 Models 中下载 **Sherpa Spanish**（或你的语言）。
4. 打开 **ON** 开关，播放视频。

## 内置模型

| 模型 | 语言 | 来源 |
|---|---|---|
| `sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06` | 西班牙语 | Hugging Face |
| `sherpa-onnx-streaming-zipformer-en-20M-2023-02-17` | 英语 | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | 英语 | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | 中文 + 英语 | GitHub Releases |

## 从源码构建

要求：Windows、[Rust](https://rustup.rs/)（MSVC）、Node 20+、`pnpm`。

```bash
pnpm install
pnpm tauri dev     # 开发模式
pnpm tauri build   # 生成 .exe + .msi
```

## 许可证

基于 **Apache License 2.0** 的开源项目——你可以对它做任何事：使用、修改、出售，无需许可。见 [LICENSE](LICENSE)。
