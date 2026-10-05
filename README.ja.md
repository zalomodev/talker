# Talker

> [Español](README.es.md) · [English](README.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [Português (BR)](README.pt-BR.md)

視聴中のアプリ / 動画の音声を**リアルタイムで文字起こし**し、**翻訳**する Windows 用オーバーレイ。

## 機能

- 🎙️ 動画やアプリの上にリアルタイムで文字起こしを表示。
- 🌍 AI による自動翻訳（例：スペイン語 → 英語）。
- 💻 [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) により 100% ローカル動作（西 / 英ストリーミングモデル）。Groq クラウドにも対応。
- 🪟 ミニマルなフローティング表示：ライブタイプライター、検出言語、文字起こし時間。
- 🎯 システム音声の取得、またはアクティブアプリの追従。
- 📦 モデルはアプリ内からワンクリックでダウンロード（Models タブ）。

## インストール

1. **Releases** ページから `.msi` または `.exe` をダウンロード。
2. インストール / 起動し、Settings → Speech → **Sherpa (real-time, local)** を選択。
3. Models で **Sherpa Spanish**（または使用言語）をダウンロード。
4. **ON** にして動画を再生。

## 収録モデル

| モデル | 言語 | 配布元 |
|---|---|---|
| `sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06` | スペイン語 | Hugging Face |
| `sherpa-onnx-streaming-zipformer-en-20M-2023-02-17` | 英語 | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | 英語 | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | 中国語 + 英語 | GitHub Releases |

## ソースからビルド

要件：Windows、[Rust](https://rustup.rs/)（MSVC）、Node 20+、`pnpm`。

```bash
pnpm install
pnpm tauri dev     # 開発モード
pnpm tauri build   # .exe + .msi を生成
```

## ライセンス

**Apache License 2.0** のオープンプロジェクト——利用・改変・販売など自由に使えます。許可は不要です。[LICENSE](LICENSE) を参照。
