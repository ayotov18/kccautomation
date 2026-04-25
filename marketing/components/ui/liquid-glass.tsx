'use client';

import { forwardRef, useCallback, type ReactNode, type MouseEvent, type CSSProperties } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  children: ReactNode;
  className?: string;
  intensity?: 'soft' | 'standard' | 'prominent';
  interactive?: boolean;
  as?: 'div' | 'section' | 'article';
  style?: CSSProperties;
  onMouseEnter?: (e: MouseEvent<HTMLDivElement>) => void;
  onMouseLeave?: (e: MouseEvent<HTMLDivElement>) => void;
};

/**
 * Liquid Glass primitive — WWDC 2025 material approximation.
 * Four layers: (1) tinted translucent fill, (2) backdrop blur+saturate,
 * (3) dual inset shadows (top light + bottom dark) for refractive edge,
 * (4) pointer-tracked specular highlight.
 */
export const LiquidGlass = forwardRef<HTMLDivElement, Props>(function LiquidGlass(
  { children, className, intensity = 'standard', interactive = true, style, onMouseEnter, onMouseLeave },
  ref,
) {
  const onMove = useCallback((e: MouseEvent<HTMLDivElement>) => {
    if (!interactive) return;
    const r = e.currentTarget.getBoundingClientRect();
    e.currentTarget.style.setProperty('--mx', `${((e.clientX - r.left) / r.width) * 100}%`);
    e.currentTarget.style.setProperty('--my', `${((e.clientY - r.top) / r.height) * 100}%`);
  }, [interactive]);

  const tintAndInset = {
    soft: {
      background: 'color-mix(in oklch, white 3%, transparent)',
      backdropFilter: 'blur(16px) saturate(160%)',
      WebkitBackdropFilter: 'blur(16px) saturate(160%)',
      boxShadow:
        'inset 0 1px 0 0 rgb(255 255 255 / 0.14), inset 0 -1px 0 0 rgb(0 0 0 / 0.3), inset 0 0 0 1px rgb(255 255 255 / 0.05), 0 14px 40px -16px rgb(0 0 0 / 0.6)',
    },
    standard: {
      background: 'color-mix(in oklch, white 5%, transparent)',
      backdropFilter: 'blur(24px) saturate(180%)',
      WebkitBackdropFilter: 'blur(24px) saturate(180%)',
      boxShadow:
        'inset 0 1px 0 0 rgb(255 255 255 / 0.22), inset 0 -1px 0 0 rgb(0 0 0 / 0.38), inset 0 0 0 1px rgb(255 255 255 / 0.07), 0 20px 60px -20px rgb(0 0 0 / 0.7)',
    },
    prominent: {
      background: 'color-mix(in oklch, white 8%, transparent)',
      backdropFilter: 'blur(32px) saturate(200%)',
      WebkitBackdropFilter: 'blur(32px) saturate(200%)',
      boxShadow:
        'inset 0 1px 0 0 rgb(255 255 255 / 0.28), inset 0 -1px 0 0 rgb(0 0 0 / 0.4), inset 0 0 0 1px rgb(255 255 255 / 0.1), 0 30px 80px -24px rgb(0 0 0 / 0.75)',
    },
  }[intensity];

  return (
    <div
      ref={ref}
      className={cn('relative overflow-hidden rounded-2xl', className)}
      style={{ ...tintAndInset, ...style }}
      onMouseMove={onMove}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      {/* Pointer-tracked specular highlight */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 rounded-[inherit]"
        style={{
          background:
            'radial-gradient(200px circle at var(--mx,30%) var(--my,-10%), rgb(255 255 255 / 0.16), transparent 55%)',
          mixBlendMode: 'plus-lighter',
        }}
      />
      {children}
    </div>
  );
});
