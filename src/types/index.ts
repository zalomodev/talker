export interface AppSettings {
  theme: 'light' | 'dark';
  audio_mode: 'auto_foreground' | 'pinned_process' | 'system_loopback';
  pinned_pid: number | null;
  transcription_provider: 'local' | 'groq' | 'sherpa';
  local_model_name: string;
  groq_model_name: string;
  sherpa_model_name: string;
  transcription_language: string | null;
  vad_threshold: number;
  vad_min_speech_ms: number;
  vad_min_silence_ms: number;
  translation_enabled: boolean;
  translation_provider: 'groq' | 'openai' | 'openrouter';
  translation_model: string;
  translation_target_lang: string;
  overlay_font_size: number;
  overlay_opacity: number;
  overlay_reduced_motion: boolean;
  overlay_show_tempo: boolean;
  overlay_width: number;
  overlay_height: number;
}

export interface TranscriptItem {
  id: number;
  text: string;
  translation?: string | null;
  is_final: boolean;
  language?: string | null;
  latencyMs?: number | null;
  timestamp: number;
}

export interface AudioStatus {
  is_active: boolean;
  source_description: string;
  active_pid?: number | null;
  active_app?: string | null;
  vad_speech_detected: boolean;
  error?: string | null;
}

export interface ActiveAppInfo {
  pid: number;
  process_name: string;
  window_title: string;
  executable_path: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  estimated_size_bytes: number;
  installed: boolean;
  path?: string | null;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  finished: boolean;
  error?: string | null;
}

export interface AppProfile {
  process_name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  font_size: number;
  opacity: number;
  translation_enabled: boolean;
  target_language?: string | null;
}
