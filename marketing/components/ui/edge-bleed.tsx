'use client';

import { cn } from '@/lib/cn';

type Props = {
  direction?: 'top' | 'bottom' | 'both';
  /** Height of each fade band in rem. */
  size?: number;
  className?: string;
};

/** Drop a mask fade at the top, bottom, or both edges of a section so it dissolves into the neighbors. */
export function EdgeBleed({ direction = 'both', size = 8, className }: Props) {
  const mask =
    direction === 'top'
      ? `linear-gradient(to bottom, transparent, #000 ${size}rem)`
      : direction === 'bottom'
        ? `linear-gradient(to top, transparent, #000 ${size}rem)`
        : `linear-gradient(to bottom, transparent, #000 ${size}rem, #000 calc(100% - ${size}rem), transparent)`;

  return (
    <div
      aria-hidden
      className={cn('pointer-events-none absolute inset-0', className)}
      style={{
        maskImage: mask,
        WebkitMaskImage: mask,
      }}
    />
  );
}

/** A single-section-spanning blur band — use to dissolve a section's edge into the next one. */
export function ProgressiveSeam({
  direction = 'bottom',
  height = 160,
  className,
}: {
  direction?: 'top' | 'bottom';
  height?: number;
  className?: string;
}) {
  const base = direction === 'bottom' ? 'bottom-0' : 'top-0';
  const layers = [
    { blur: 1, from: 0, to: 30 },
    { blur: 2, from: 10, to: 45 },
    { blur: 4, from: 20, to: 60 },
    { blur: 8, from: 40, to: 80 },
    { blur: 14, from: 60, to: 100 },
  ];
  return (
    <div
      aria-hidden
      className={cn('pointer-events-none absolute left-0 right-0', base, className)}
      style={{ height }}
    >
      {layers.map((l, i) => (
        <div
          key={i}
          className="absolute inset-0"
          style={{
            backdropFilter: `blur(${l.blur}px)`,
            WebkitBackdropFilter: `blur(${l.blur}px)`,
            maskImage:
              direction === 'bottom'
                ? `linear-gradient(to bottom, transparent ${l.from}%, #000 ${l.to}%)`
                : `linear-gradient(to top, transparent ${l.from}%, #000 ${l.to}%)`,
            WebkitMaskImage:
              direction === 'bottom'
                ? `linear-gradient(to bottom, transparent ${l.from}%, #000 ${l.to}%)`
                : `linear-gradient(to top, transparent ${l.from}%, #000 ${l.to}%)`,
          }}
        />
      ))}
    </div>
  );
}

/** Amber gleam that crosses a section seam — signature moment, use sparingly. */
export function AccentGleam({
  position = { left: '20%', bottom: '-20%' },
  size = 700,
  opacity = 0.18,
  className,
}: {
  position?: { left?: string; right?: string; top?: string; bottom?: string };
  size?: number;
  opacity?: number;
  className?: string;
}) {
  return (
    <div
      aria-hidden
      className={cn('pointer-events-none absolute', className)}
      style={{
        width: size,
        aspectRatio: '1',
        background: `radial-gradient(closest-side, oklch(0.72 0.16 55 / ${opacity}), transparent 70%)`,
        filter: 'blur(40px)',
        ...position,
      }}
    />
  );
}
