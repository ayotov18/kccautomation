'use client';

import { cn } from '@/lib/cn';

type Props = {
  size?: number;
  duration?: number;
  delay?: number;
  colorFrom?: string;
  colorTo?: string;
  className?: string;
};

/** Port of magicui/border-beam — a traveling light seam around the element's border. */
export function BorderBeam({
  size = 200,
  duration = 8,
  delay = 0,
  colorFrom = 'oklch(0.82 0.19 62)',
  colorTo = 'oklch(0.72 0.16 55 / 0)',
  className,
}: Props) {
  return (
    <div
      style={
        {
          '--size': `${size}px`,
          '--duration': `${duration}s`,
          '--delay': `-${delay}s`,
          '--color-from': colorFrom,
          '--color-to': colorTo,
        } as React.CSSProperties
      }
      className={cn(
        'pointer-events-none absolute inset-0 rounded-[inherit] [border:1px_solid_transparent] ![mask-clip:padding-box,border-box] ![mask-composite:intersect] [mask:linear-gradient(transparent,transparent),linear-gradient(transparent,transparent)]',
        'after:absolute after:aspect-square after:w-[var(--size)] after:animate-[beam_var(--duration)_linear_infinite] after:[offset-anchor:90%_50%] after:[offset-path:rect(0_auto_auto_0_round_var(--size))] after:[background:linear-gradient(to_left,var(--color-from),var(--color-to))] after:[animation-delay:var(--delay)]',
        className,
      )}
    />
  );
}
