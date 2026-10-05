import React, { useEffect, useState } from 'react';
import { X, Check, AlertCircle } from 'lucide-react';
import { useSettingsStore } from '../state/useSettingsStore';
import { useOverlayStore } from '../state/useOverlayStore';
import { useAudioStore } from '../state/useAudioStore';
import { ModelManagerTab } from './ModelManagerTab';
import { ProfilesTab } from './ProfilesTab';
import { getStrings } from '../i18n';

export const SettingsModal: React.FC = () => {
  const isSettingsOpen = useOverlayStore((state) => state.isSettingsOpen);
  const setSettingsOpen = useOverlayStore((state) => state.setSettingsOpen);

  const settings = useSettingsStore((state) => state.settings);
  const updateSettings = useSettingsStore((state) => state.updateSettings);
  const getApiKey = useSettingsStore((state) => state.getApiKey);
  const setApiKey = useSettingsStore((state) => state.setApiKey);
  const testGroqConnection = useSettingsStore((state) => state.testGroqConnection);

  const activeApp = useAudioStore((state) => state.activeApp);

  const [groqKey, setGroqKey] = useState('');
  const [groqTestStatus, setGroqTestStatus] = useState<string | null>(null);
  const [groqTestLoading, setGroqTestLoading] = useState(false);
  const [groqTestError, setGroqTestError] = useState<string | null>(null);
  const [transKey, setTransKey] = useState('');

  useEffect(() => {
    if (isSettingsOpen) {
      getApiKey('groq').then((k) => setGroqKey(k || ''));
      if (settings?.translation_provider) {
        getApiKey(settings.translation_provider).then((k) => setTransKey(k || ''));
      }
    }
  }, [isSettingsOpen, settings?.translation_provider]);

  if (!isSettingsOpen || !settings) return null;

  const T = getStrings(settings.ui_language);

  const handleSaveGroqKey = async () => {
    await setApiKey('groq', groqKey);
  };

  const handleTestGroq = async () => {
    setGroqTestLoading(true);
    setGroqTestStatus(null);
    setGroqTestError(null);
    try {
      const res = await testGroqConnection(groqKey);
      setGroqTestStatus(res);
      await setApiKey('groq', groqKey);
    } catch (e: any) {
      setGroqTestError(String(e));
    } finally {
      setGroqTestLoading(false);
    }
  };

  const handleSaveTransKey = async () => {
    if (settings.translation_provider) {
      await setApiKey(settings.translation_provider, transKey);
    }
  };

  const selectClass =
    'w-full px-2 py-1.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-md focus:outline-none';
  const labelClass = 'font-medium text-neutral-700 dark:text-neutral-300 block mb-1';

  return (
    <div className="animate-slide-down absolute top-9 right-2 w-72 max-h-[calc(100%-3rem)] bg-white/95 dark:bg-talker-dark-surface/95 border border-neutral-200 dark:border-talker-dark-border rounded-xl shadow-2xl z-50 flex flex-col text-[11px] text-neutral-600 dark:text-neutral-300 font-sans overflow-hidden">
      <div className="px-3 py-2 border-b border-neutral-200 dark:border-talker-dark-border flex items-center justify-between">
        <span className="font-semibold text-neutral-800 dark:text-neutral-200">{T.settingsTitle}</span>
        <div className="flex items-center gap-1.5">
          <select
            value={settings.ui_language}
            onChange={(e) => updateSettings({ ui_language: e.target.value as any })}
            title={T.languageLabel}
            className="px-1 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-[11px] focus:outline-none"
          >
            <option value="es">ES</option>
            <option value="en">EN</option>
          </select>
          <button
            onClick={() => setSettingsOpen(false)}
            className="p-0.5 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200"
          >
            <X size={13} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto divide-y divide-neutral-100 dark:divide-neutral-800">
        <section className="p-3 space-y-2">
          <h4 className="text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{T.audioHeader}</h4>
          <select
            value={settings.audio_mode}
            onChange={(e) => updateSettings({ audio_mode: e.target.value as any })}
            className={selectClass}
          >
            <option value="auto_foreground">{T.audioFollowApp}</option>
            <option value="system_loopback">{T.audioSystem}</option>
          </select>
          <div className="font-mono text-[10px] text-neutral-500 truncate">
            {activeApp?.process_name ? T.audioTarget(activeApp.process_name) : T.audioNoTarget}
          </div>
          <div>
            <div className="flex justify-between"><label className={labelClass}>{T.vadLabel}</label><span className="font-mono text-neutral-500">{settings.vad_threshold.toFixed(2)}</span></div>
            <input type="range" min="0.2" max="0.8" step="0.05" value={settings.vad_threshold}
              onChange={(e) => updateSettings({ vad_threshold: parseFloat(e.target.value) })}
              className="w-full accent-neutral-800 dark:accent-neutral-200" />
          </div>
        </section>

        <section className="p-3 space-y-2">
          <h4 className="text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{T.speechHeader}</h4>
          <select
            value={settings.transcription_provider}
            onChange={(e) => updateSettings({ transcription_provider: e.target.value as any })}
            className={selectClass}
          >
            <option value="groq">{T.providerGroq}</option>
            <option value="sherpa">{T.providerSherpa}</option>
            <option value="local">{T.providerLocal}</option>
          </select>

          {settings.transcription_provider === 'groq' && (
            <div className="space-y-2">
              <div className="flex gap-1.5">
                <input type="password" placeholder={T.groqKeyPlaceholder} value={groqKey}
                  onChange={(e) => setGroqKey(e.target.value)} onBlur={handleSaveGroqKey}
                  className="flex-1 px-2 py-1.5 font-mono bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-md focus:outline-none" />
                <button onClick={handleTestGroq} disabled={groqTestLoading || !groqKey}
                  className="px-2 bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900 rounded-md disabled:opacity-50">
                  {groqTestLoading ? '…' : T.testButton}
                </button>
              </div>
              {groqTestStatus && <div className="p-1.5 bg-emerald-50 dark:bg-emerald-950/50 border border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300 rounded flex items-center gap-1"><Check size={11} />{groqTestStatus}</div>}
              {groqTestError && <div className="p-1.5 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-900 text-red-700 dark:text-red-300 rounded flex items-center gap-1"><AlertCircle size={11} />{groqTestError}</div>}
              <select value={settings.groq_model_name}
                onChange={(e) => updateSettings({ groq_model_name: e.target.value })} className={selectClass}>
                <option value="whisper-large-v3-turbo">whisper-large-v3-turbo</option>
                <option value="whisper-large-v3">whisper-large-v3</option>
              </select>
            </div>
          )}

          {settings.transcription_provider === 'local' && (
            <select value={settings.local_model_name}
              onChange={(e) => updateSettings({ local_model_name: e.target.value })} className={selectClass}>
              <option value="openai/whisper-tiny">Whisper Tiny</option>
              <option value="openai/whisper-base">Whisper Base</option>
              <option value="openai/whisper-small">Whisper Small</option>
            </select>
          )}

          {settings.transcription_provider === 'sherpa' && (
            <select value={settings.sherpa_model_name}
              onChange={(e) => updateSettings({ sherpa_model_name: e.target.value })} className={selectClass}>
              <option value="sherpa-onnx-streaming-zipformer-es-kroko-2025-08-06">Sherpa Spanish</option>
              <option value="sherpa-onnx-streaming-zipformer-en-20M-2023-02-17">Sherpa Small</option>
              <option value="sherpa-onnx-streaming-zipformer-en-2023-06-26">Sherpa Decent</option>
              <option value="sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20">Sherpa Bilingual</option>
            </select>
          )}

          {settings.transcription_provider === 'local' && (
            <p className="text-[10px] text-neutral-500 dark:text-neutral-400">
              {T.whisperNote(settings.local_model_name)}
            </p>
          )}

          <select value={settings.transcription_language || 'auto'}
            onChange={(e) => updateSettings({ transcription_language: e.target.value === 'auto' ? null : e.target.value })}
            className={selectClass}>
            <option value="auto">{T.langAuto}</option>
            <option value="en">{T.lang_en}</option>
            <option value="es">{T.lang_es}</option>
            <option value="fr">{T.lang_fr}</option>
            <option value="de">{T.lang_de}</option>
            <option value="it">{T.lang_it}</option>
            <option value="ja">{T.lang_ja}</option>
            <option value="ko">{T.lang_ko}</option>
            <option value="zh">{T.lang_zh}</option>
            <option value="pt">{T.lang_pt}</option>
            <option value="ru">{T.lang_ru}</option>
          </select>
        </section>

        <section className="p-3 space-y-2">
          <div className="flex justify-between items-center">
            <h4 className="text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{T.translationHeader}</h4>
            <input type="checkbox" checked={settings.translation_enabled}
              onChange={(e) => updateSettings({ translation_enabled: e.target.checked })}
              className="h-3 w-3 accent-neutral-900 dark:accent-neutral-100" />
          </div>
          {settings.translation_enabled && (
            <div className="space-y-2">
              <input type="text" value={settings.translation_target_lang}
                onChange={(e) => updateSettings({ translation_target_lang: e.target.value })}
                placeholder={T.translationTargetPlaceholder} className={selectClass} />
              <select value={settings.translation_provider}
                onChange={(e) => updateSettings({ translation_provider: e.target.value as any })} className={selectClass}>
                <option value="groq">Groq</option>
                <option value="openai">OpenAI</option>
                <option value="openrouter">OpenRouter</option>
              </select>
              <select value={settings.translation_model}
                onChange={(e) => updateSettings({ translation_model: e.target.value })} className={selectClass}>
                <option value="openai/gpt-oss-120b">openai/gpt-oss-120b</option>
                <option value="llama-3.3-70b-versatile">llama-3.3-70b-versatile</option>
              </select>
              <input type="password" placeholder={T.transKeyPlaceholder} value={transKey}
                onChange={(e) => setTransKey(e.target.value)} onBlur={handleSaveTransKey}
                className="w-full px-2 py-1.5 font-mono bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-md focus:outline-none" />
            </div>
          )}
        </section>

        <section className="p-3 space-y-3">
          <h4 className="text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{T.appearanceHeader}</h4>
          <div className="flex gap-1.5">
            {(['light', 'dark'] as const).map((t) => (
              <button key={t} onClick={() => updateSettings({ theme: t })}
                className={`flex-1 py-1.5 rounded-md border text-[11px] ${settings.theme === t ? 'border-neutral-900 dark:border-neutral-100' : 'border-neutral-200 dark:border-neutral-700'}`}>
                {t === 'light' ? T.themeLight : T.themeDark}
              </button>
            ))}
          </div>
          <div>
            <div className="flex justify-between"><label className={labelClass}>{T.fontSize}</label><span className="font-mono text-neutral-500">{settings.overlay_font_size}px</span></div>
            <input type="range" min="13" max="32" step="1" value={settings.overlay_font_size}
              onChange={(e) => updateSettings({ overlay_font_size: parseInt(e.target.value) })}
              className="w-full accent-neutral-800 dark:accent-neutral-200" />
          </div>
          <div>
            <div className="flex justify-between"><label className={labelClass}>{T.opacity}</label><span className="font-mono text-neutral-500">{Math.round(settings.overlay_opacity * 100)}%</span></div>
            <input type="range" min="0.4" max="1.0" step="0.05" value={settings.overlay_opacity}
              onChange={(e) => updateSettings({ overlay_opacity: parseFloat(e.target.value) })}
              className="w-full accent-neutral-800 dark:accent-neutral-200" />
          </div>
          <label className="flex items-center justify-between">
            <span className="font-medium text-neutral-700 dark:text-neutral-300">{T.showTempo}</span>
            <input type="checkbox" checked={settings.overlay_show_tempo ?? true}
              onChange={(e) => updateSettings({ overlay_show_tempo: e.target.checked })}
              className="h-3 w-3 accent-neutral-900 dark:accent-neutral-100" />
          </label>
          <label className="flex items-center justify-between">
            <span className="font-medium text-neutral-700 dark:text-neutral-300">{T.reducedMotion}</span>
            <input type="checkbox" checked={settings.overlay_reduced_motion}
              onChange={(e) => updateSettings({ overlay_reduced_motion: e.target.checked })}
              className="h-3 w-3 accent-neutral-900 dark:accent-neutral-100" />
          </label>
        </section>

        <section className="p-3">
          <ModelManagerTab />
        </section>

        <section className="p-3">
          <ProfilesTab />
        </section>
      </div>
    </div>
  );
};
