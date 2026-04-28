'use client';

/**
 * Drawing detail page.
 *
 * Replaced the 5-widget raw-stat grid (entity distribution, annotation
 * chips, blocks list, active layers, drawing info) with a single
 * AI-generated bilingual summary that the operator redacts before signing.
 *
 * Top action row uses the unified rounded-full button system. View Drawing
 * is the primary action; KCC generation is a single secondary button that
 * opens the prepare flow (mode chooser lives there).
 */

import { useEffect, useState, useCallback } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { api } from '@/lib/api';
import { Breadcrumbs } from '@/components/layout/Breadcrumbs';
import { DrawingSummary } from '@/components/drawing/DrawingSummary';

export default function DrawingOverviewPage() {
  const params = useParams();
  const router = useRouter();
  const drawingId = params.id as string;

  const [summary, setSummary] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchSummary = useCallback(async () => {
    try {
      const data = await api.getDrawingSummary(drawingId);
      setSummary(data);
    } catch {
      /* drawing not found is handled below */
    }
    setLoading(false);
  }, [drawingId]);

  useEffect(() => {
    fetchSummary();
  }, [fetchSummary]);

  useEffect(() => {
    const name = (summary?.drawing as Record<string, unknown> | undefined)?.filename as
      | string
      | undefined;
    document.title = name ? `${name} · KCC` : 'Drawing · KCC';
  }, [summary]);

  if (loading) {
    return (
      <div className="oe-fade-in">
        <div className="max-w-5xl mx-auto px-6 py-12">
          <div className="text-content-tertiary text-sm">Loading…</div>
        </div>
      </div>
    );
  }

  const drawing = (summary?.drawing ?? {}) as Record<string, unknown>;
  const kss = (summary?.kss ?? {}) as Record<string, unknown>;
  const kssGenerated = kss?.status === 'generated';
  const filename = (drawing.filename as string) ?? 'Drawing';
  const entityCount = (drawing.entity_count as number) ?? 0;
  const format = (drawing.format as string) ?? '';
  const uploadedRaw = drawing.created_at as string | undefined;
  const uploaded = uploadedRaw
    ? new Date(uploadedRaw).toLocaleDateString('en-GB', {
        day: 'numeric',
        month: 'short',
        year: 'numeric',
      })
    : null;

  const handleAiKss = async () => {
    try {
      await api.triggerAiKssResearch(drawingId);
      router.push(`/drawings/${drawingId}/kss/prepare`);
    } catch {
      /* swallow — prepare page surfaces the failure */
    }
  };

  return (
    <div className="oe-fade-in">
      <div className="max-w-5xl mx-auto px-6 py-8 space-y-6">
        {/* Header — left: breadcrumb + title + meta. right: action group. */}
        <div className="flex items-start justify-between gap-6">
          <div className="min-w-0 flex-1">
            <Breadcrumbs
              items={[
                { label: 'Drawings', href: '/drawings' },
                { label: filename },
              ]}
            />
            <h1 className="mt-2 text-[26px] font-semibold tracking-tight text-content-primary truncate">
              {filename}
            </h1>
            <p className="mt-1 text-[12.5px] text-content-tertiary">
              <span className="font-numeric">{format}</span>
              <span className="mx-1.5 opacity-40">·</span>
              <span className="font-numeric">{entityCount.toLocaleString('en-GB')}</span>{' '}
              entities
              {uploaded && (
                <>
                  <span className="mx-1.5 opacity-40">·</span>
                  Uploaded {uploaded}
                </>
              )}
            </p>
          </div>

          {/* Single action row. All same shape, primary reserved for the
              one CTA the user is here to do (open the viewer or KCC).
              Linear-style: avoid two primaries side-by-side. */}
          <div className="flex items-center gap-2 shrink-0">
            <button
              onClick={() => router.push(`/drawings/${drawingId}/viewer`)}
              className="oe-btn-secondary"
            >
              View drawing
            </button>
            {kssGenerated ? (
              <button
                onClick={() => router.push(`/drawings/${drawingId}/kss`)}
                className="oe-btn-primary"
              >
                Open KCC
              </button>
            ) : (
              <button onClick={handleAiKss} className="oe-btn-primary">
                Generate KCC
              </button>
            )}
          </div>
        </div>

        {/* KCC status pill — only when ready. */}
        {kssGenerated && (
          <div className="flex items-center justify-between gap-4 rounded-2xl border border-border-light/40 bg-surface-secondary/40 px-5 py-3">
            <div className="flex items-center gap-3">
              <span className="inline-block w-1.5 h-1.5 rounded-full bg-semantic-success" />
              <div>
                <p className="text-[13px] text-content-primary">
                  KCC report ready
                  {(kss.ai_enhanced as boolean) && (
                    <span className="ml-2 text-[10.5px] font-medium uppercase tracking-wider text-content-tertiary">
                      AI-enhanced
                    </span>
                  )}
                </p>
                <p className="text-[12px] text-content-tertiary mt-0.5">
                  <span className="font-numeric">{kss.item_count as number}</span> items
                  <span className="mx-1.5 opacity-40">·</span>
                  Total{' '}
                  <span className="font-numeric">
                    {((kss.total_with_vat_eur as number) ?? 0).toFixed(2)} €
                  </span>{' '}
                  inc. VAT
                </p>
              </div>
            </div>
            <button
              onClick={() => router.push(`/drawings/${drawingId}/kss`)}
              className="oe-btn-ghost oe-btn-sm"
            >
              Open
            </button>
          </div>
        )}

        {/* AI summary — replaces the 5 raw-stats widgets. */}
        <DrawingSummary drawingId={drawingId} />
      </div>
    </div>
  );
}
