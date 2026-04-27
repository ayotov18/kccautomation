'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';
import { Check, X, ArrowLeft, ArrowRight, CheckCheck } from 'lucide-react';
import type { KssSuggestion } from '@/types';
import { describeExtractionMethod } from '@/types';

/**
 * Focused single-card reviewer for AI suggestions.
 *
 * UX goals:
 *   - One item at a time, zero cognitive load from a scroll list.
 *   - Auto-advance on accept/reject so the user keeps their hand on one button.
 *   - Keyboard-first (A / R / ← / →) for power users.
 *   - Persistent state of decisions in the parent store via callbacks.
 */
interface Props {
  suggestions: KssSuggestion[];
  pendingAccepts: Set<string>;
  pendingRejects: Set<string>;
  onAccept: (id: string) => void;
  onReject: (id: string) => void;
  onUndo: (id: string) => void;
  onCommit: () => void;
  onClose: () => void;
}

export function SuggestionsReviewer({
  suggestions,
  pendingAccepts,
  pendingRejects,
  onAccept,
  onReject,
  onUndo,
  onCommit,
  onClose,
}: Props) {
  const [idx, setIdx] = useState(0);
  const total = suggestions.length;
  const reviewed = pendingAccepts.size + pendingRejects.size;
  const acceptedCount = pendingAccepts.size;
  const rejectedCount = pendingRejects.size;
  const progress = total > 0 ? Math.round((reviewed / total) * 100) : 0;

  const item = suggestions[idx];
  const staged = item
    ? pendingAccepts.has(item.id)
      ? 'accepted'
      : pendingRejects.has(item.id)
      ? 'rejected'
      : null
    : null;
  const allDone = reviewed === total && total > 0;

  const advance = useCallback(
    (dir: 1 | -1) => {
      if (total === 0) return;
      setIdx((cur) => {
        // Prefer landing on the next un-reviewed item; fall back to +/- 1
        for (let step = 1; step <= total; step++) {
          const next = (cur + dir * step + total) % total;
          const s = suggestions[next];
          if (!pendingAccepts.has(s.id) && !pendingRejects.has(s.id)) return next;
        }
        return (cur + dir + total) % total;
      });
    },
    [suggestions, pendingAccepts, pendingRejects, total],
  );

  const handleAccept = useCallback(() => {
    if (!item) return;
    onAccept(item.id);
    // Give the staged pill a moment to flash, then advance.
    setTimeout(() => advance(1), 220);
  }, [item, onAccept, advance]);

  const handleReject = useCallback(() => {
    if (!item) return;
    onReject(item.id);
    setTimeout(() => advance(1), 220);
  }, [item, onReject, advance]);

  const handleUndo = useCallback(() => {
    if (!item) return;
    onUndo(item.id);
  }, [item, onUndo]);

  // Keyboard controls
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') return onClose();
      if (allDone) return;
      if (e.key === 'ArrowRight') { e.preventDefault(); advance(1); }
      else if (e.key === 'ArrowLeft') { e.preventDefault(); advance(-1); }
      else if (e.key === 'a' || e.key === 'A') { e.preventDefault(); handleAccept(); }
      else if (e.key === 'r' || e.key === 'R') { e.preventDefault(); handleReject(); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [advance, allDone, handleAccept, handleReject, onClose]);

  const confColor = useMemo(() => {
    if (!item) return { bg: 'var(--oe-bg-secondary)', fg: 'var(--oe-text-secondary)' };
    const c = item.confidence ?? 0;
    if (c < 0.4) return { bg: 'var(--oe-error-bg)', fg: 'var(--oe-error)' };
    if (c < 0.7) return { bg: 'var(--oe-warning-bg)', fg: 'var(--oe-warning)' };
    return { bg: 'var(--oe-blue-subtle)', fg: 'var(--oe-blue)' };
  }, [item]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />

      <div
        className="relative w-full max-w-[640px] rounded-2xl overflow-hidden oe-fade-in flex flex-col"
        style={{
          background: 'var(--oe-bg-elevated)',
          border: '1px solid var(--oe-border-light)',
          boxShadow: 'var(--oe-shadow-lg)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 pt-5 pb-3">
          <div className="min-w-0">
            <h2 className="text-base font-semibold" style={{ color: 'var(--oe-text-primary)' }}>
              Преглед на AI предложения
            </h2>
            <p className="text-[11px] mt-0.5" style={{ color: 'var(--oe-text-tertiary)' }}>
              Позиция {allDone ? total : idx + 1} от {total} · {acceptedCount} приети · {rejectedCount} отхвърлени
            </p>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-surface-secondary text-content-tertiary hover:text-content-primary transition-colors"
            aria-label="Затвори"
          >
            <X size={16} strokeWidth={2} />
          </button>
        </div>

        {/* Progress rail */}
        <div className="mx-6 h-1 rounded-full overflow-hidden" style={{ background: 'var(--oe-bg-secondary)' }}>
          <div
            className="h-full transition-all duration-300 ease-out"
            style={{ width: `${progress}%`, background: 'var(--oe-blue)' }}
          />
        </div>

        {/* Card body */}
        <div className="px-6 py-5 min-h-[260px] flex flex-col">
          {allDone ? (
            <div className="flex-1 flex flex-col items-center justify-center text-center py-6">
              <div
                className="w-14 h-14 rounded-full flex items-center justify-center mb-4"
                style={{ background: 'var(--oe-success-bg)', color: 'var(--oe-success)' }}
              >
                <CheckCheck size={28} strokeWidth={2} />
              </div>
              <h3 className="text-base font-semibold mb-1" style={{ color: 'var(--oe-text-primary)' }}>
                Прегледа е готов
              </h3>
              <p className="text-xs mb-5" style={{ color: 'var(--oe-text-secondary)' }}>
                {acceptedCount} приети · {rejectedCount} отхвърлени. Натиснете по-долу за запазване.
              </p>
              <button
                onClick={() => {
                  onClose();
                  onCommit();
                }}
                className="oe-btn-primary oe-btn-lg"
              >
                Запази промените
              </button>
            </div>
          ) : item ? (
            <>
              <div className="flex items-start justify-between gap-3 mb-3">
                <h3
                  className="text-[15px] font-semibold leading-snug flex-1"
                  style={{ color: 'var(--oe-text-primary)' }}
                >
                  {item.description}
                </h3>
                <div className="shrink-0 flex items-center gap-1.5">
                  <ExtractionPill
                    method={item.extraction_method ?? null}
                    geometryConfidence={item.geometry_confidence ?? null}
                  />
                  <span
                    className="text-[10px] px-2 py-1 rounded-full font-bold tracking-wide"
                    style={{ background: confColor.bg, color: confColor.fg }}
                    title="Обща увереност на реда (цена × геометрия)"
                  >
                    {Math.round((item.confidence ?? 0) * 100)}%
                  </span>
                </div>
              </div>

              {item.reasoning && (
                <p
                  className="text-xs italic mb-4 leading-relaxed"
                  style={{ color: 'var(--oe-text-tertiary)' }}
                >
                  {item.reasoning}
                </p>
              )}

              <div className="grid grid-cols-4 gap-2 mb-4">
                <Metric label="Количество" value={`${formatNum(item.quantity)} ${item.unit}`} />
                <Metric label="Материали" value={`${formatNum((item.material_price ?? 0) * item.quantity)} €`} />
                <Metric label="Труд" value={`${formatNum((item.labor_price ?? 0) * item.quantity)} €`} />
                <Metric label="Общо" value={`${formatNum(item.total_eur)} €`} emphasis />
              </div>

              {staged && (
                <div
                  className="flex items-center justify-between gap-2 py-2 px-3 rounded-lg mb-4"
                  style={{
                    background: staged === 'accepted' ? 'var(--oe-success-bg)' : 'var(--oe-error-bg)',
                    color: staged === 'accepted' ? 'var(--oe-success)' : 'var(--oe-error)',
                  }}
                >
                  <span className="text-xs font-medium">
                    {staged === 'accepted' ? '✓ Staged за приемане' : '✗ Staged за отхвърляне'}
                  </span>
                  <button
                    onClick={handleUndo}
                    className="text-xs underline underline-offset-2 hover:opacity-80"
                  >
                    Отмени
                  </button>
                </div>
              )}

              <div className="mt-auto flex items-center justify-between gap-3">
                <button
                  onClick={() => advance(-1)}
                  className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs transition-colors hover:bg-surface-secondary text-content-secondary"
                  aria-label="Предишна (←)"
                >
                  <ArrowLeft size={14} /> Назад
                </button>

                <div className="flex items-center gap-2">
                  <button
                    onClick={handleReject}
                    disabled={staged === 'rejected'}
                    className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
                    style={{ background: 'var(--oe-error-bg)', color: 'var(--oe-error)' }}
                  >
                    <X size={14} /> Отхвърли <kbd className="kbd">R</kbd>
                  </button>
                  <button
                    onClick={handleAccept}
                    disabled={staged === 'accepted'}
                    className="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
                    style={{ background: 'var(--oe-success-bg)', color: 'var(--oe-success)' }}
                  >
                    <Check size={14} /> Приеми <kbd className="kbd">A</kbd>
                  </button>
                </div>

                <button
                  onClick={() => advance(1)}
                  className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs transition-colors hover:bg-surface-secondary text-content-secondary"
                  aria-label="Следваща (→)"
                >
                  Напред <ArrowRight size={14} />
                </button>
              </div>
            </>
          ) : null}
        </div>

        {/* Footer */}
        <div
          className="px-6 py-3 border-t text-xs flex items-center justify-between"
          style={{ borderColor: 'var(--oe-border-light)', color: 'var(--oe-text-tertiary)' }}
        >
          <span>
            ← → навигация · <kbd className="kbd">A</kbd> приеми · <kbd className="kbd">R</kbd> отхвърли
          </span>
          <button onClick={onClose} className="oe-btn-ghost text-xs">Затвори</button>
        </div>
      </div>

      <style jsx>{`
        .kbd {
          display: inline-block;
          min-width: 14px;
          padding: 1px 5px;
          font-size: 10px;
          font-family: var(--oe-font-mono, ui-monospace);
          background: rgba(255, 255, 255, 0.08);
          border: 1px solid rgba(255, 255, 255, 0.12);
          border-radius: 3px;
          line-height: 1.2;
          vertical-align: middle;
        }
      `}</style>
    </div>
  );
}

function Metric({ label, value, emphasis = false }: { label: string; value: string; emphasis?: boolean }) {
  return (
    <div
      className="rounded-lg px-3 py-2"
      style={{
        background: emphasis ? 'var(--oe-blue-subtle)' : 'var(--oe-bg-secondary)',
        border: '1px solid var(--oe-border-light)',
      }}
    >
      <div className="text-[10px] uppercase tracking-wider" style={{ color: 'var(--oe-text-tertiary)' }}>
        {label}
      </div>
      <div
        className="text-sm font-semibold font-mono mt-0.5 truncate"
        style={{ color: emphasis ? 'var(--oe-blue)' : 'var(--oe-text-primary)' }}
      >
        {value}
      </div>
    </div>
  );
}

function formatNum(n: number | null | undefined) {
  if (n == null || !Number.isFinite(n)) return '—';
  if (Math.abs(n) >= 1000) return n.toLocaleString('bg-BG', { maximumFractionDigits: 0 });
  return n.toLocaleString('bg-BG', { maximumFractionDigits: 2 });
}

/**
 * Extraction-method provenance pill.
 * Tone:
 *   - trust  → sky (measured polyline / counted blocks / text annotation)
 *   - assume → slate (length × assumed height / derived from primary)
 *   - flag   → amber (assumed default / ai-inferred — needs review)
 */
function ExtractionPill({
  method,
  geometryConfidence,
}: {
  method: import('@/types').ExtractionMethod | null;
  geometryConfidence: number | null;
}) {
  const m = describeExtractionMethod(method);
  const palette = {
    trust:  { bg: 'rgba(14,165,233,0.12)', fg: 'rgb(14,165,233)' },
    assume: { bg: 'rgba(100,116,139,0.14)', fg: 'rgb(100,116,139)' },
    flag:   { bg: 'rgba(245,158,11,0.15)',  fg: 'rgb(217,119,6)' },
  }[m.tone];
  const confSuffix =
    geometryConfidence == null ? '' : ` · геом. ${Math.round(geometryConfidence * 100)}%`;
  return (
    <span
      className="text-[10px] px-2 py-1 rounded-full font-medium tracking-wide cursor-help"
      style={{ background: palette.bg, color: palette.fg }}
      title={`${m.title}${confSuffix}`}
    >
      {m.label}
    </span>
  );
}
