import { create } from 'zustand';
import { TranscriptItem } from '../types';

interface TranscriptState {
  visibleItems: TranscriptItem[];
  activePartial: TranscriptItem | null;
  internalHistory: TranscriptItem[];
  lastLanguage: string | null;
  lastLatencyMs: number | null;
  addTranscriptEvent: (payload: {
    id: number;
    text: string;
    translation?: string | null;
    is_final: boolean;
    language?: string | null;
    latency_ms?: number | null;
  }) => void;
  clear: () => void;
}

const MAX_VISIBLE_ITEMS = 5;
const MAX_INTERNAL_HISTORY = 60;

export const useTranscriptStore = create<TranscriptState>((set) => ({
  visibleItems: [],
  activePartial: null,
  internalHistory: [],
  lastLanguage: null,
  lastLatencyMs: null,

  addTranscriptEvent: (payload) => {
    const now = Date.now();
    const item: TranscriptItem = {
      id: payload.id,
      text: payload.text,
      translation: payload.translation,
      is_final: payload.is_final,
      language: payload.language,
      latencyMs: payload.latency_ms ?? null,
      timestamp: now,
    };

    if (!payload.is_final) {
      set({ activePartial: item, lastLanguage: payload.language ?? null, lastLatencyMs: payload.latency_ms ?? null });
    } else {
      set((state) => {
        // Filter out if duplicate ID exists, otherwise append
        const filtered = state.visibleItems.filter((i) => i.id !== payload.id);
        const nextVisible = [...filtered, item];

        // Keep visible bounded to prevent UI saturation
        const boundedVisible =
          nextVisible.length > MAX_VISIBLE_ITEMS
            ? nextVisible.slice(nextVisible.length - MAX_VISIBLE_ITEMS)
            : nextVisible;

        // Keep rolling internal context for linguistics
        const nextHistory = [...state.internalHistory, item];
        const boundedHistory =
          nextHistory.length > MAX_INTERNAL_HISTORY
            ? nextHistory.slice(nextHistory.length - MAX_INTERNAL_HISTORY)
            : nextHistory;

        return {
          visibleItems: boundedVisible,
          activePartial: null,
          internalHistory: boundedHistory,
          lastLanguage: payload.language ?? state.lastLanguage,
          lastLatencyMs: payload.latency_ms ?? state.lastLatencyMs,
        };
      });
    }
  },

  clear: () => set({ visibleItems: [], activePartial: null, lastLatencyMs: null }),
}));
