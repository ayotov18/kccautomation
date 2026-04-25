'use client';

import { useCallback, type ReactNode, type MouseEvent } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  children: ReactNode;
  className?: string;
  /** Size of the spotlight in px */
  size?: number;
  /** Color, any CSS color */
  color?: string;
  /** Base tint */
  tint?: string;
};

/**
 * Cursor-tracking spotlight + glass. Updates CSS vars --mx/--my on mousemove.
 * Pair with LiquidGlass for full widget shine.
 */
export function SpotlightCard({
  children,
  className,
  size = 420,
  color = 'oklch(0.72 0.16 55 / 0.18)',
  tint = 'color-mix(in oklch, white 3%, transparent)',
}: Props) {
  const onMove = useCallback((e: MouseEvent<HTMLDivElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    e.currentTarget.style.setProperty('--mx', `${e.clientX - r.left}px`);
    e.currentTarget.style.setProperty('--my', `${e.clientY - r.top}px`);
  }, []);

  return (
    <div
      onMouseMove={onMove}
      className={cn(
        'group relative overflow-hidden rounded-2xl border border-white/[0.06] transition-colors',
        'hover:border-white/[0.12]',
        className,
      )}
      style={{
        background: tint,
        boxShadow:
          'inset 0 1px 0 0 rgb(255 255 255 / 0.08), inset 0 -1px 0 0 rgb(0 0 0 / 0.3), 0 16px 50px -18px rgb(0 0 0 / 0.5)',
      }}
    >
      {/* Cursor spotlight */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-500 group-hover:opacity-100"
        style={{
          background: `radial-gradient(${size}px circle at var(--mx) var(--my), ${color}, transparent 60%)`,
        }}
      />
      {/* Subtle refractive top light on hover */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-white/30 to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100"
      />
      {children}
    </div>
  );
}
