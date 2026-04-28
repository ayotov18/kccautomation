'use client';

import { ReactNode, useEffect, useRef, useState } from 'react';
import { ChevronRight } from 'lucide-react';
import { clsx } from 'clsx';

/**
 * Dense, single-line accordion row — Linear/Notion-style data row.
 * Target height: ~40 px closed. Caller owns open state.
 */
interface Props {
  number: string;
  title: string;
  itemCount: number;
  total: number;
  isOpen: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function KssSectionAccordion({
  number,
  title,
  itemCount,
  total,
  isOpen,
  onToggle,
  children,
}: Props) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [maxH, setMaxH] = useState<number | 'none'>(isOpen ? 'none' : 0);

  useEffect(() => {
    const el = bodyRef.current;
    if (!el) return;
    if (isOpen) {
      setMaxH(el.scrollHeight);
      const t = setTimeout(() => setMaxH('none'), 220);
      return () => clearTimeout(t);
    } else {
      setMaxH(el.scrollHeight);
      requestAnimationFrame(() => setMaxH(0));
    }
  }, [isOpen]);

  return (
    <section className="border-b border-border-light/60 last:border-b-0">
      <button
        onClick={onToggle}
        aria-expanded={isOpen}
        className={clsx(
          'w-full flex items-center gap-3 pl-3 pr-4 py-2.5 text-left transition-colors group',
          'hover:bg-surface-secondary/60 focus-visible:outline-none focus-visible:bg-surface-secondary/60',
          isOpen && 'bg-surface-secondary/40',
        )}
      >
        <ChevronRight
          size={14}
          strokeWidth={2.25}
          className={clsx(
            'flex-none text-content-tertiary transition-transform duration-150',
            isOpen && 'rotate-90',
          )}
        />
        <span className="flex-none w-10 text-right font-mono text-[11px] tracking-wide text-[color:var(--oe-accent)]/80 uppercase">
          {number}
        </span>
        <span className="flex-1 min-w-0 text-sm font-medium text-content-primary truncate">
          {title}
        </span>
        <span className="flex-none text-[10px] font-mono px-1.5 py-0.5 rounded bg-surface-tertiary/60 text-content-tertiary">
          {itemCount}
        </span>
        <span className="flex-none text-right font-mono text-sm text-content-primary tabular-nums">
          {formatLv(total)}
          <span className="text-content-tertiary ml-1 text-[10px] uppercase tracking-wider">€</span>
        </span>
      </button>

      <div
        ref={bodyRef}
        style={{
          maxHeight: maxH === 'none' ? undefined : `${maxH}px`,
          overflow: maxH === 'none' ? 'visible' : 'hidden',
          transition: 'max-height 200ms ease-out, opacity 150ms ease-out',
          opacity: isOpen ? 1 : 0,
        }}
      >
        <div className="px-4 pb-4 pt-1">{children}</div>
      </div>
    </section>
  );
}

function formatLv(n: number) {
  if (!Number.isFinite(n)) return '—';
  return n.toLocaleString('bg-BG', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}
