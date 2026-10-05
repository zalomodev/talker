import React, { memo, useMemo } from 'react';
import { useTranscriptStore } from '../state/useTranscriptStore';
import { useSettingsStore } from '../state/useSettingsStore';
import { TranscriptItem } from '../types';

interface LineProps {
  item: TranscriptItem;
  fontSize: number;
  translationEnabled: boolean;
  reducedMotion: boolean;
  distanceFromEnd: number;
}

/** Settled line: memoized so live partials never re-render it. */
const FinalLine = memo<LineProps>(function FinalLine({
  item,
  fontSize,
  translationEnabled,
  reducedMotion,
  distanceFromEnd,
}) {
  let opacity = 1.0;
  if (distanceFromEnd === 1) opacity = 0.85;
  else if (distanceFromEnd === 2) opacity = 0.65;
  else if (distanceFromEnd === 3) opacity = 0.45;
  else if (distanceFromEnd >= 4) opacity = 0.25;

  return (
    <div
      style={{
        opacity,
        fontSize: `${fontSize}px`,
        transition: reducedMotion ? 'none' : 'opacity 0.25s ease, transform 0.25s ease',
      }}
      className={`my-1.5 leading-relaxed font-normal tracking-wide ${reducedMotion ? '' : 'animate-slide-up'}`}
    >
      {translationEnabled && item.translation ? (
        <div>
          <div className="text-neutral-900 dark:text-neutral-100 font-medium">
            {item.translation}
          </div>
          <div className="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5 opacity-75">
            {item.text}
          </div>
        </div>
      ) : (
        <div className="text-neutral-900 dark:text-neutral-100">{item.text}</div>
      )}
    </div>
  );
});

interface PartialProps {
  text: string;
  fontSize: number;
  reducedMotion: boolean;
}

/**
 * Live typewriter: words are keyed by position+token, so when the hypothesis
 * grows only the new tail mounts (with a fade) instead of re-rendering the
 * whole line. A blinking caret marks the live edge.
 */
const LivePartial = memo<PartialProps>(function LivePartial({ text, fontSize, reducedMotion }) {
  const tokens = useMemo(() => text.split(/(\s+)/), [text]);
  return (
    <div
      style={{
        fontSize: `${fontSize}px`,
        transition: reducedMotion ? 'none' : 'opacity 0.15s ease',
      }}
      className="my-1 text-neutral-600 dark:text-neutral-400 italic tracking-wide"
    >
      {tokens.map((tok, i) =>
        tok === '' ? null : (
          <span key={`${i}-${tok}`} className={reducedMotion ? '' : 'animate-fade-in'}>
            {tok}
          </span>
        ),
      )}
      {!reducedMotion && (
        <span className="inline-block w-[2px] h-[1em] bg-current align-[-0.15em] ml-0.5 animate-blink" />
      )}
    </div>
  );
});

export const TranscriptDisplay: React.FC = () => {
  const visibleItems = useTranscriptStore((state) => state.visibleItems);
  const activePartial = useTranscriptStore((state) => state.activePartial);
  const settings = useSettingsStore((state) => state.settings);

  const fontSize = settings?.overlay_font_size || 18;
  const translationEnabled = settings?.translation_enabled || false;
  const reducedMotion = settings?.overlay_reduced_motion || false;
  const total = visibleItems.length;

  return (
    <div className="flex-1 flex flex-col justify-end px-4 py-2 overflow-hidden text-center select-none">
      {/* Settled / Confirmed flowing transcription lines */}
      <div className="flex flex-col items-center justify-end w-full">
        {visibleItems.map((item, idx) => (
          <FinalLine
            key={item.id}
            item={item}
            fontSize={fontSize}
            translationEnabled={translationEnabled}
            reducedMotion={reducedMotion}
            distanceFromEnd={total - 1 - idx}
          />
        ))}

        {/* Live Partial Unstable Transcript Reveal */}
        {activePartial && (
          <LivePartial
            key="live-partial"
            text={activePartial.text}
            fontSize={fontSize}
            reducedMotion={reducedMotion}
          />
        )}
      </div>
    </div>
  );
};
