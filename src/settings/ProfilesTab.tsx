import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Trash2, Monitor } from 'lucide-react';
import { AppProfile } from '../types';

export const ProfilesTab: React.FC = () => {
  const [profiles, setProfiles] = useState<AppProfile[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchProfiles = async () => {
    try {
      const list = await invoke<AppProfile[]>('list_profiles');
      setProfiles(list);
    } catch (e: any) {
      setError(String(e));
    }
  };

  useEffect(() => {
    fetchProfiles();
  }, []);

  const handleDelete = async (processName: string) => {
    try {
      await invoke('delete_profile', { processName });
      await fetchProfiles();
    } catch (e: any) {
      setError(String(e));
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <h3 className="text-sm font-semibold text-neutral-800 dark:text-neutral-200">
          Saved Application Profiles
        </h3>
        <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
          Talker remembers your chosen overlay placement, font size, and dimensions for each game or app.
        </p>
      </div>

      {error && (
        <div className="p-2.5 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-900 rounded text-xs text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      {profiles.length === 0 ? (
        <div className="p-6 text-center text-xs text-neutral-400 dark:text-neutral-500 border border-dashed border-neutral-200 dark:border-neutral-700 rounded-lg">
          No application profiles saved yet. Move or resize Talker while an app is active to save a layout.
        </div>
      ) : (
        <div className="space-y-2">
          {profiles.map((p) => (
            <div
              key={p.process_name}
              className="p-3 bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700 rounded-lg flex items-center justify-between"
            >
              <div className="flex items-center gap-2.5">
                <Monitor size={15} className="text-neutral-400" />
                <div>
                  <div className="text-xs font-semibold text-neutral-900 dark:text-neutral-100">
                    {p.process_name}
                  </div>
                  <div className="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono">
                    pos: ({p.x}, {p.y}) &bull; size: {p.width}x{p.height} &bull; font: {p.font_size}px
                  </div>
                </div>
              </div>

              <button
                onClick={() => handleDelete(p.process_name)}
                className="p-1.5 text-neutral-400 hover:text-red-500 transition-colors"
                title="Delete profile"
              >
                <Trash2 size={13} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
