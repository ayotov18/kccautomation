'use client';

import { clsx } from 'clsx';

/**
 * Modern skeleton loading block. Uses the existing `oe-skeleton` shimmer
 * animation from globals.css. Default rounded radius matches cards.
 *
 * Use:
 *   <Skeleton className="w-40 h-4" />
 *   <SkeletonText lines={3} />
 *   <SkeletonCard />
 *   <SkeletonRow cols={[80, '100%', 60, 100, 100, 100, 120]} />
 */
export function Skeleton({ className, style }: { className?: string; style?: React.CSSProperties }) {
  return <div className={clsx('oe-skeleton', className)} style={style} aria-hidden="true" />;
}

export function SkeletonText({ lines = 1, className }: { lines?: number; className?: string }) {
  return (
    <div className={clsx('space-y-2', className)} aria-hidden="true">
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          className="h-3.5"
          style={{ width: `${100 - (i === lines - 1 ? 30 : i * 8)}%` }}
        />
      ))}
    </div>
  );
}

export function SkeletonCard({ className }: { className?: string }) {
  return (
    <div
      className={clsx(
        'oe-card p-4 flex flex-col items-center gap-2',
        className,
      )}
      aria-hidden="true"
    >
      <Skeleton className="w-20 h-7" />
      <Skeleton className="w-24 h-3" />
    </div>
  );
}

/** Sized table-row skeleton — widths accept px or "%" */
export function SkeletonRow({
  cols,
  className,
}: {
  cols: (number | string)[];
  className?: string;
}) {
  return (
    <tr className={className} aria-hidden="true">
      {cols.map((w, i) => (
        <td key={i} className="px-3 py-2">
          <Skeleton
            className="h-3"
            style={{ width: typeof w === 'number' ? `${w}px` : w }}
          />
        </td>
      ))}
    </tr>
  );
}
