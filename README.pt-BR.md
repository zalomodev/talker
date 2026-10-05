# Talker

> [Español](README.es.md) · [English](README.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [Português (BR)](README.pt-BR.md)

Overlay para Windows que **transcreve ao vivo** o áudio dos seus apps e vídeos, e **traduz** em tempo real.

## O que faz

- 🎙️ Transcrição em tempo real sobre o vídeo ou app que você está assistindo.
- 🌍 Tradução automática com IA (ex.: espanhol → inglês).
- 💻 100% local com [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (modelos de streaming ES/EN), ou na nuvem com Groq.
- 🪟 Overlay flutuante minimalista: typewriter ao vivo, idioma detectado e tempo de transcrição.
- 🎯 Captura o som do sistema ou segue o app ativo.
- 📦 Os modelos baixam sozinhos dentro do app (aba Modelos).

## Instalação

1. Baixe o `.msi` ou o `.exe` na aba **Releases**.
2. Instale / execute, abra Ajustes → Voz → escolha **Sherpa (tempo real, local)**.
3. Em Modelos, baixe o **Sherpa Spanish** (ou o do seu idioma).
4. Ligue o **ON** e dê play no vídeo.

## Modelos incluídos

| Modelo | Idioma | Fonte |
|---|---|---|
| `sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06` | Espanhol | Hugging Face |
| `sherpa-onnx-streaming-zipformer-en-20M-2023-02-17` | Inglês | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | Inglês | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | Chinês + Inglês | GitHub Releases |

## Compilar do código-fonte

Requisitos: Windows, [Rust](https://rustup.rs/) (MSVC), Node 20+ e `pnpm`.

```bash
pnpm install
pnpm tauri dev     # modo desenvolvimento
pnpm tauri build   # gera .exe + .msi
```

## Licença

Projeto aberto sob a **Apache License 2.0** — faça o que quiser com ele: usar, modificar, vender, sem pedir permissão. Veja [LICENSE](LICENSE).
