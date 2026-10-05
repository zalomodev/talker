import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Download, Trash2, CheckCircle2 } from 'lucide-react';
import { DownloadProgress, ModelInfo } from '../types';

export const ModelManagerTab: React.FC = () => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [activeDownload, setActiveDownload] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchModels = async () => {
    try {
      const list = await invoke<ModelInfo[]>('list_models');
      setModels(list);
    } catch (e: any) {
      setError(String(e));
    }
  };

  useEffect(() => {
    fetchModels();

    const unlistenPromise = listen<DownloadProgress>('model-download-progress', (event) => {
      const p = event.payload;
      setActiveDownload(p);
      if (p.finished) {
        setActiveDownload(null);
        fetchModels();
      }
      if (p.error) {
        setError(p.error);
        setActiveDownload(null);
      }
    });

    return () => {
      unlistenPromise.then((un) => un());
    };
  }, []);

  const handleDownload = async (modelId: string) => {
    setError(null);
    try {
      await invoke('download_model', { modelId });
    } catch (e: any) {
      setError(String(e));
      setActiveDownload(null);
    }
  };

  const handleDelete = async (modelId: string) => {
    setError(null);
    try {
      await invoke('delete_model', { modelId });
      await fetchModels();
    } catch (e: any) {
      setError(String(e));
    }
  };

  const formatMB = (bytes: number) => {
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-neutral-800 dark:text-neutral-200">
          Local Whisper Models
        </h3>
        <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
          Local models run 100% offline on your machine with no internet required.
        </p>
      </div>

      {error && (
        <div className="p-2.5 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-900 rounded text-xs text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      <div className="space-y-2.5">
        {models.map((model) => {
          const isDownloading = activeDownload?.model_id === model.id;

          return (
            <div
              key={model.id}
              className="p-3 bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700 rounded-lg flex flex-col gap-2"
            >
              <div className="flex items-center justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold text-neutral-900 dark:text-neutral-100">
                      {model.name}
                    </span>
                    {model.installed && (
                      <span className="inline-flex items-center gap-1 text-[10px] font-medium text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/60 px-1.5 py-0.5 rounded">
                        <CheckCircle2 size={10} /> Installed
                      </span>
                    )}
                  </div>
                  <p className="text-[11px] text-neutral-500 dark:text-neutral-400 mt-0.5">
                    {model.description}
                  </p>
                </div>

                <div>
                  {model.installed ? (
                    <button
                      onClick={() => handleDelete(model.id)}
                      className="p-1.5 text-neutral-400 hover:text-red-500 transition-colors"
                      title="Delete local weights"
                    >
                      <Trash2 size={13} />
                    </button>
                  ) : (
                    <button
                      onClick={() => handleDownload(model.id)}
                      disabled={isDownloading}
                      className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900 rounded hover:opacity-90 disabled:opacity-50 transition-opacity"
                    >
                      <Download size={11} />
                      {isDownloading ? 'Downloading...' : 'Download'}
                    </button>
                  )}
                </div>
              </div>

              {/* Byte-Accurate Progress Bar */}
              {isDownloading && activeDownload && (
                <div className="space-y-1 pt-1">
                  <div className="w-full bg-neutral-200 dark:bg-neutral-700 h-1.5 rounded-full overflow-hidden">
                    <div
                      className="bg-neutral-800 dark:bg-neutral-200 h-full transition-all duration-150"
                      style={{ width: `${activeDownload.percent.toFixed(1)}%` }}
                    />
                  </div>
                  <div className="flex justify-between text-[10px] text-neutral-500 dark:text-neutral-400">
                    <span>{activeDownload.percent.toFixed(1)}%</span>
                    <span>
                      {formatMB(activeDownload.downloaded_bytes)} /{' '}
                      {formatMB(activeDownload.total_bytes)}
                    </span>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
