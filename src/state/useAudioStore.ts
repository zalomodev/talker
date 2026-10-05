import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ActiveAppInfo, AudioStatus } from '../types';

interface AudioState {
  isTranscribing: boolean;
  audioStatus: AudioStatus | null;
  activeApp: ActiveAppInfo | null;
  error: string | null;
  initListeners: () => () => void;
  startTranscription: () => Promise<void>;
  stopTranscription: () => Promise<void>;
  toggleTranscription: () => Promise<void>;
}

export const useAudioStore = create<AudioState>((set, get) => ({
  isTranscribing: false,
  audioStatus: null,
  activeApp: null,
  error: null,

  initListeners: () => {
    let unlistenAudio: (() => void) | undefined;
    let unlistenApp: (() => void) | undefined;

    listen<AudioStatus>('audio-status', (event) => {
      set({
        audioStatus: event.payload,
        isTranscribing: event.payload.is_active,
        error: event.payload.error || null,
      });
    }).then((un) => {
      unlistenAudio = un;
    });

    listen<ActiveAppInfo>('active-app-changed', (event) => {
      set({ activeApp: event.payload });
    }).then((un) => {
      unlistenApp = un;
    });

    invoke<boolean>('is_transcribing').then((active) => {
      set({ isTranscribing: active });
    });

    return () => {
      if (unlistenAudio) unlistenAudio();
      if (unlistenApp) unlistenApp();
    };
  },

  startTranscription: async () => {
    set({ error: null });
    try {
      await invoke('start_transcription');
      set({ isTranscribing: true });
    } catch (err: any) {
      set({ isTranscribing: false, error: String(err) });
    }
  },

  stopTranscription: async () => {
    try {
      await invoke('stop_transcription');
      set({ isTranscribing: false });
    } catch (err: any) {
      set({ error: String(err) });
    }
  },

  toggleTranscription: async () => {
    if (get().isTranscribing) {
      await get().stopTranscription();
    } else {
      await get().startTranscription();
    }
  },
}));
