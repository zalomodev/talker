import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types';

interface SettingsState {
  settings: AppSettings | null;
  isLoading: boolean;
  error: string | null;
  loadSettings: () => Promise<void>;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  getApiKey: (provider: string) => Promise<string | null>;
  setApiKey: (provider: string, key: string) => Promise<void>;
  testGroqConnection: (key: string) => Promise<string>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  isLoading: true,
  error: null,

  loadSettings: async () => {
    try {
      const data = await invoke<AppSettings>('get_settings');
      set({ settings: data, isLoading: false, error: null });
      if (data.theme === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    } catch (err: any) {
      set({ isLoading: false, error: String(err) });
    }
  },

  updateSettings: async (partial: Partial<AppSettings>) => {
    const current = get().settings;
    if (!current) return;
    const updated = { ...current, ...partial };
    set({ settings: updated });

    if (partial.theme) {
      if (partial.theme === 'dark') {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }

    try {
      await invoke('save_settings', { newSettings: updated });
    } catch (err: any) {
      set({ error: String(err) });
    }
  },

  getApiKey: async (provider: string) => {
    return await invoke<string | null>('get_api_key', { provider });
  },

  setApiKey: async (provider: string, key: string) => {
    await invoke('set_api_key', { provider, key });
  },

  testGroqConnection: async (apiKey: string) => {
    return await invoke<string>('test_groq_connection', { apiKey });
  },
}));
