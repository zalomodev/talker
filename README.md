# Talker

> [Español](README.md) · [English](README.en.md) · [中文](README.zh-CN.md) · [日本語](README.ja.md) · [Português (BR)](README.pt-BR.md)

Overlay para Windows que **transcribe en vivo** el audio de tus apps y videos, y lo **traduce** en tiempo real.

## Qué hace

- 🎙️ Transcripción en tiempo real sobre el video o app que estés viendo.
- 🌍 Traducción automática con IA (ej. español → inglés).
- 💻 100% local con [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (modelos streaming ES/EN), o en la nube con Groq.
- 🪟 Overlay flotante minimalista: typewriter en vivo, idioma detectado y tiempo de transcripción.
- 🎯 Captura el sonido del sistema o sigue la app activa.
- 📦 Los modelos se descargan solos desde la app (pestaña Modelos).

## Instalación

1. Descarga el `.msi` o el `.exe` desde la pestaña **Releases**.
2. Instala / ejecuta, abre Ajustes → Voz → elige **Sherpa (tiempo real, local)**.
3. En Modelos descarga **Sherpa Spanish** (o el de tu idioma).
4. Activa el **ON** y reproduce tu video.

## Modelos incluidos

| Modelo | Idioma | Fuente |
|---|---|---|
| `sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06` | Español | Hugging Face |
| `sherpa-onnx-streaming-zipformer-en-20M-2023-02-17` | Inglés | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-en-2023-06-26` | Inglés | GitHub Releases |
| `sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20` | Chino + Inglés | GitHub Releases |

## Compilar desde fuente

Requisitos: Windows, [Rust](https://rustup.rs/) (MSVC), Node 20+ y `pnpm`.

```bash
pnpm install
pnpm tauri dev     # modo desarrollo
pnpm tauri build   # genera .exe + .msi
```

## Licencia

Proyecto abierto bajo **Apache License 2.0** — puedes hacer lo que quieras con él: usarlo, modificarlo, venderlo, sin pedir permiso. Ver [LICENSE](LICENSE).
