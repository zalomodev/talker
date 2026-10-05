import { create } from 'zustand';

interface OverlayState {
  isEditMode: boolean;
  isSettingsOpen: boolean;
  setEditMode: (edit: boolean) => Promise<void>;
  toggleEditMode: () => Promise<void>;
  setSettingsOpen: (open: boolean) => Promise<void>;
}

export const useOverlayStore = create<OverlayState>((set, get) => ({
  isEditMode: false,
  isSettingsOpen: false,

  setEditMode: async (edit: boolean) => {
    set({ isEditMode: edit });
  },

  toggleEditMode: async () => {
    const next = !get().isEditMode;
    set({ isEditMode: next });
  },

  setSettingsOpen: async (open: boolean) => {
    set({ isSettingsOpen: open });
  },
}));
