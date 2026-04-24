'use client';

import Link from 'next/link';
import { ChevronRight } from 'lucide-react';

/**
 * Thin, filename-aware breadcrumb. UUIDs in the path are replaced with the
 * drawing's human-readable filename so the trail reads like:
 *
 *   Drawings › ESTUDO-BANGALO-R00.dwg › View
 *
 * Segments are passed in by the caller — the page knows what makes sense.
 */
export interface Crumb {
  label: string;
  href?: string;
}

export function Breadcrumbs({ items }: { items: Crumb[] }) {
  return (
    <nav
      aria-label="Breadcrumb"
      className="flex items-center gap-1.5 text-[13px] text-content-secondary min-w-0"
    >
      {items.map((crumb, idx) => {
        const last = idx === items.length - 1;
        return (
          <span key={idx} className="flex items-center gap-1.5 min-w-0">
            {crumb.href && !last ? (
              <Link
                href={crumb.href}
                className="hover:text-content-primary transition-colors truncate max-w-[240px]"
                title={crumb.label}
              >
                {crumb.label}
              </Link>
            ) : (
              <span
                className={last ? 'text-content-primary font-medium truncate max-w-[320px]' : 'truncate max-w-[240px]'}
                title={crumb.label}
              >
                {crumb.label}
              </span>
            )}
            {!last && (
              <ChevronRight
                size={12}
                strokeWidth={2}
                className="text-content-tertiary flex-none"
              />
            )}
          </span>
        );
      })}
    </nav>
  );
}
