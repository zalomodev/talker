import React, { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window';
import { OverlayView } from './overlay/OverlayView';
import { SettingsModal } from './settings/SettingsModal';
import { useSettingsStore } from './state/useSettingsStore';
import { useTranscriptStore } from './state/useTranscriptStore';
import { useAudioStore } from './state/useAudioStore';
import { ActiveAppInfo, AppProfile } from './types';

export const App: React.FC = () => {
  const loadSettings = useSettingsStore((state) => state.loadSettings);
  const addTranscriptEvent = useTranscriptStore((state) => state.addTranscriptEvent);
  const initAudioListeners = useAudioStore((state) => state.initListeners);

  useEffect(() => {
    loadSettings();
    const cleanupAudio = initAudioListeners();

    // Listen to real-time transcription events
    let unlistenTranscript: (() => void) | undefined;
    listen<any>('transcription-event', (event) => {
      addTranscriptEvent(event.payload);
    }).then((un) => {
      unlistenTranscript = un;
    });

    // Listen for foreground application changes to restore per-app layout profiles
    let unlistenApp: (() => void) | undefined;
    listen<ActiveAppInfo>('active-app-changed', async (event) => {
      const app = event.payload;
      if (!app || !app.process_name) return;

      try {
        const profile = await invoke<AppProfile | null>('get_profile', {
          processName: app.process_name,
        });

        if (profile) {
          const appWindow = getCurrentWindow();
          await appWindow.setPosition(new PhysicalPosition(profile.x, profile.y));
          await appWindow.setSize(new PhysicalSize(profile.width, profile.height));
        }
      } catch (err) {
        console.error('Failed restoring profile layout:', err);
      }
    }).then((un) => {
      unlistenApp = un;
    });

    return () => {
      cleanupAudio();
      if (unlistenTranscript) unlistenTranscript();
      if (unlistenApp) unlistenApp();
    };
  }, [loadSettings, addTranscriptEvent, initAudioListeners]);

  return (
    <div className="relative h-screen w-screen bg-transparent overflow-hidden">
      <OverlayView />
      <SettingsModal />
    </div>
  );
};

export default App;
