import React from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { Settings, Move, Lock, X, Timer } from 'lucide-react';
import { TranscriptDisplay } from './TranscriptDisplay';
import { useAudioStore } from '../state/useAudioStore';
import { useOverlayStore } from '../state/useOverlayStore';
import { useSettingsStore } from '../state/useSettingsStore';
import { useTranscriptStore } from '../state/useTranscriptStore';

export const OverlayView: React.FC = () => {
  const isTranscribing = useAudioStore((state) => state.isTranscribing);
  const speaking = useAudioStore((state) => state.audioStatus?.vad_speech_detected ?? false);
  const lastLanguage = useTranscriptStore((state) => state.lastLanguage);
  const lastLatencyMs = useTranscriptStore((state) => state.lastLatencyMs);

  const tempoLabel =
    lastLatencyMs == null
      ? null
      : lastLatencyMs < 1000
        ? `${Math.round(lastLatencyMs)}ms`
        : `${(lastLatencyMs / 1000).toFixed(1)}s`;
  const toggleTranscription = useAudioStore((state) => state.toggleTranscription);
  const activeApp = useAudioStore((state) => state.activeApp);
  const error = useAudioStore((state) => state.error);

  const isEditMode = useOverlayStore((state) => state.isEditMode);
  const toggleEditMode = useOverlayStore((state) => state.toggleEditMode);
  const setSettingsOpen = useOverlayStore((state) => state.setSettingsOpen);

  const settings = useSettingsStore((state) => state.settings);
  const opacity = settings?.overlay_opacity || 0.95;
  const showTempo = settings?.overlay_show_tempo ?? true;

  const handleToggleEdit = async () => {
    if (isEditMode) {
      // Save current window position and size
      try {
        const appWindow = getCurrentWindow();
        const pos = await appWindow.outerPosition();
        const size = await appWindow.outerSize();
        await invoke('save_overlay_bounds', {
          x: pos.x,
          y: pos.y,
          width: size.width,
          height: size.height,
          processName: activeApp?.process_name || null,
        });
      } catch (e) {
        console.error('Failed saving overlay bounds:', e);
      }
    }
    await toggleEditMode();
  };

  const handleClose = async () => {
    try {
      await invoke('close_window');
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div
      style={{
        backgroundColor:
          settings?.theme === 'dark'
            ? `rgba(32, 32, 32, ${opacity})`
            : `rgba(255, 255, 255, ${opacity})`,
      }}
      className={`h-screen w-screen flex flex-col rounded-xl overflow-hidden transition-all duration-200 select-none shadow-lg ${
        isEditMode
          ? 'border-2 border-dashed border-neutral-400 dark:border-neutral-500'
          : 'border border-neutral-200/60 dark:border-neutral-700/60'
      }`}
    >
      {/* Extremely minimal title bar */}
      <div
        data-tauri-drag-region
        className="h-8 px-3.5 flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400 border-b border-neutral-100 dark:border-neutral-800/80 cursor-default"
      >
        <div data-tauri-drag-region className="flex items-center gap-2">
          <span
            data-tauri-drag-region
            className="font-semibold tracking-wider text-neutral-800 dark:text-neutral-200 text-[11px]"
          >
            TALKER
          </span>
          {/* Live speaking indicator */}
          {isTranscribing && speaking && (
            <span className="relative flex h-2 w-2" title="Speech detected">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
              <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
            </span>
          )}
          {/* Detected source-language badge */}
          {lastLanguage && (
            <span
              key={lastLanguage}
              className="text-[10px] font-mono text-neutral-500 dark:text-neutral-400 bg-neutral-100 dark:bg-neutral-800 px-1.5 py-0.5 rounded animate-fade-in"
            >
              {lastLanguage.toUpperCase()}
            </span>
          )}
          {/* Transcription tempo: how long the last one took */}
          {showTempo && tempoLabel && (
            <span
              key={tempoLabel}
              title="Tiempo de transcripción"
              className="flex items-center gap-1 text-[10px] font-mono text-neutral-500 dark:text-neutral-400 bg-neutral-100 dark:bg-neutral-800 px-1.5 py-0.5 rounded animate-fade-in"
            >
              <Timer size={10} />
              {tempoLabel}
            </span>
          )}
          {isEditMode && (
            <span className="text-[10px] text-amber-700 dark:text-amber-400 bg-amber-100/80 dark:bg-amber-950/60 px-1.5 py-0.5 rounded font-mono">
              Moviendo
            </span>
          )}
        </div>

        {/* Unobtrusive System-Style Controls */}
        <div className="flex items-center gap-3">
          {/* Real ON / OFF toggle */}
          <div className="flex items-center gap-1.5">
            <span
              className={`text-[11px] font-medium transition-colors ${
                isTranscribing
                  ? 'text-neutral-900 dark:text-neutral-100'
                  : 'text-neutral-400 dark:text-neutral-500'
              }`}
            >
              {isTranscribing ? 'On' : 'Off'}
            </span>
            <button
              onClick={toggleTranscription}
              title={isTranscribing ? 'Dejar de escuchar' : 'Escuchar'}
              className={`relative inline-flex h-4 w-7 items-center rounded-full transition-colors focus:outline-none ${
                isTranscribing
                  ? 'bg-neutral-800 dark:bg-neutral-200'
                  : 'bg-neutral-300 dark:bg-neutral-600'
              }`}
            >
              <span
                className={`inline-block h-2.5 w-2.5 transform rounded-full bg-white dark:bg-neutral-900 transition-transform ${
                  isTranscribing ? 'translate-x-3.5' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          {/* Edit / Lock position button */}
          <button
            onClick={handleToggleEdit}
            title={isEditMode ? 'Fijar posición' : 'Mover ventana'}
            className="p-1 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 rounded transition-colors"
          >
            {isEditMode ? <Lock size={13} /> : <Move size={13} />}
          </button>

          {/* Settings button */}
          <button
            onClick={() => setSettingsOpen(true)}
            title="Ajustes"
            className="p-1 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 rounded transition-colors"
          >
            <Settings size={13} />
          </button>

          {/* Close button in edit mode */}
          {isEditMode && (
            <button
              onClick={handleClose}
              title="Cerrar Talker"
              className="p-1 text-neutral-400 hover:text-red-500 rounded transition-colors"
            >
              <X size={13} />
            </button>
          )}
        </div>
      </div>

      {/* Main Reading Surface */}
      <div className="flex-1 flex flex-col relative overflow-hidden">
        <TranscriptDisplay />

        {/* Real error banner if something fails honestly */}
        {error && (
          <div className="absolute bottom-2 left-4 right-4 bg-red-50/90 dark:bg-red-950/80 border border-red-200 dark:border-red-900 rounded-md p-2 text-xs text-red-700 dark:text-red-300 text-center animate-fade-in">
            {error}
          </div>
        )}
      </div>
    </div>
  );
};
