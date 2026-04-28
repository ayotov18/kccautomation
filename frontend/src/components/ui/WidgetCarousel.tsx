'use client';

import { ReactNode, useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { clsx } from 'clsx';

export interface WidgetSlide {
  id: string;
  label: string;
  content: ReactNode;
}

/**
 * Horizontal paginated carousel of dashboard widgets.
 *
 *   ┌─────────────────────────────────────────────────┐
 *   │  Title · widget label                       ← → │
 *   │                                                 │
 *   │            slide content                        │
 *   │                                                 │
 *   │                 ● ○ ○                           │
 *   └─────────────────────────────────────────────────┘
 *
 * Nav: arrow buttons · clickable dots · keyboard ← → · horizontal scroll-snap
 * Accessibility: aria-label on controls, aria-selected on dots, live region for label
 */
export function WidgetCarousel({
  slides,
  storageKey,
  className,
  minHeight = 280,
}: {
  slides: WidgetSlide[];
  /** localStorage key to persist which slide the user was last on. */
  storageKey?: string;
  className?: string;
  /** Min-height of the content area in px. */
  minHeight?: number;
}) {
  const [index, setIndex] = useState(() => {
    if (typeof window === 'undefined' || !storageKey) return 0;
    const raw = window.localStorage.getItem(storageKey);
    const parsed = raw ? parseInt(raw, 10) : 0;
    return Number.isFinite(parsed) && parsed >= 0 && parsed < slides.length ? parsed : 0;
  });

  useEffect(() => {
    if (storageKey && typeof window !== 'undefined') {
      window.localStorage.setItem(storageKey, String(index));
    }
  }, [index, storageKey]);

  const go = useCallback(
    (dir: 1 | -1) => setIndex((i) => (i + dir + slides.length) % slides.length),
    [slides.length],
  );

  // Keyboard when carousel region is focused or hovered
  const rootRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const onKey = (e: KeyboardEvent) => {
      // Only when focus is inside the carousel (don't steal ← → from inputs elsewhere)
      if (!root.contains(document.activeElement)) return;
      if (e.key === 'ArrowLeft') { e.preventDefault(); go(-1); }
      else if (e.key === 'ArrowRight') { e.preventDefault(); go(1); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [go]);

  const current = slides[index];

  return (
    <div
      ref={rootRef}
      tabIndex={0}
      role="region"
      aria-roledescription="carousel"
      aria-label="KCC report widgets"
      className={clsx(
        'oe-card overflow-hidden focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--oe-accent)]/40',
        className,
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border-light bg-surface-elevated/60">
        <div className="flex items-center gap-2 text-xs text-content-secondary">
          <span className="uppercase tracking-wider font-medium">
            {current?.label}
          </span>
          <span className="text-content-tertiary">
            · {index + 1} от {slides.length}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            type="button"
            onClick={() => go(-1)}
            aria-label="Предишен"
            className="oe-btn-ghost oe-btn-sm oe-btn-icon"
          >
            <ChevronLeft size={14} strokeWidth={2.25} />
          </button>
          <button
            type="button"
            onClick={() => go(1)}
            aria-label="Следващ"
            className="oe-btn-ghost oe-btn-sm oe-btn-icon"
          >
            <ChevronRight size={14} strokeWidth={2.25} />
          </button>
        </div>
      </div>

      {/* Sliding rail */}
      <div className="relative">
        <div
          className="flex transition-transform duration-300 ease-out"
          style={{
            transform: `translateX(-${index * 100}%)`,
            minHeight: `${minHeight}px`,
          }}
          aria-live="polite"
        >
          {slides.map((s, i) => (
            <div
              key={s.id}
              className="w-full flex-none"
              aria-hidden={i !== index}
            >
              {s.content}
            </div>
          ))}
        </div>
      </div>

      {/* Dots */}
      <div className="flex items-center justify-center gap-1.5 py-3 border-t border-border-light/60">
        {slides.map((s, i) => {
          const active = i === index;
          return (
            <button
              key={s.id}
              type="button"
              onClick={() => setIndex(i)}
              aria-label={`${s.label} (${i + 1} от ${slides.length})`}
              aria-selected={active}
              className={clsx(
                'transition-all duration-200 rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[color:var(--oe-accent)]/40',
                active
                  ? 'w-5 h-1.5 bg-[color:var(--oe-accent)]'
                  : 'w-1.5 h-1.5 bg-content-tertiary/40 hover:bg-content-tertiary/70',
              )}
            />
          );
        })}
      </div>
    </div>
  );
}
