'use client';

import { cn } from '@/lib/cn';

type Props = {
  direction?: 'top' | 'bottom' | 'left' | 'right';
  blurIntensity?: number;
  className?: string;
  layers?: number;
};

/** Layered progressive blur at page seams — fakes depth-of-field on scroll. */
export function ProgressiveBlur({
  direction = 'bottom',
  blurIntensity = 12,
  className,
  layers = 6,
}: Props) {
  const gradient = (from: number, to: number) => {
    switch (direction) {
      case 'top':
        return `linear-gradient(to top, transparent ${100 - to}%, black ${100 - from}%, black 100%)`;
      case 'bottom':
        return `linear-gradient(to bottom, transparent ${100 - to}%, black ${100 - from}%, black 100%)`;
      case 'left':
        return `linear-gradient(to left, transparent ${100 - to}%, black ${100 - from}%, black 100%)`;
      case 'right':
        return `linear-gradient(to right, transparent ${100 - to}%, black ${100 - from}%, black 100%)`;
    }
  };

  return (
    <div
      aria-hidden
      className={cn('pointer-events-none absolute inset-0', className)}
      style={{ zIndex: 5 }}
    >
      {Array.from({ length: layers }).map((_, i) => {
        const from = (i * 100) / layers;
        const to = ((i + 1) * 100) / layers;
        const mask = gradient(from, to);
        return (
          <div
            key={i}
            className="absolute inset-0"
            style={{
              backdropFilter: `blur(${(blurIntensity * (i + 1)) / layers}px)`,
              WebkitMaskImage: mask,
              maskImage: mask,
            }}
          />
        );
      })}
    </div>
  );
}
