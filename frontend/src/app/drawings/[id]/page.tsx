'use client';

import { useEffect, useState, useCallback } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { api } from '@/lib/api';
import { Breadcrumbs } from '@/components/layout/Breadcrumbs';

export default function DrawingOverviewPage() {
  const params = useParams();
  const router = useRouter();
  const drawingId = params.id as string;

  const [summary, setSummary] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [genJobId, setGenJobId] = useState<string | null>(null);

  const fetchSummary = useCallback(async () => {
    try {
      const data = await api.getDrawingSummary(drawingId);
      setSummary(data);
    } catch { /* */ }
    setLoading(false);
  }, [drawingId]);

  useEffect(() => { fetchSummary(); }, [fetchSummary]);

  useEffect(() => {
    const name = (summary?.drawing as Record<string, unknown> | undefined)?.filename as
      | string
      | undefined;
    document.title = name ? `${name} · KCC` : 'Drawing · KCC';
  }, [summary]);

  // Poll KSS generation job
  useEffect(() => {
    if (!genJobId) return;
    const interval = setInterval(async () => {
      try {
        const job = await api.getJob(genJobId);
        if (job.status === 'done') {
          clearInterval(interval);
          router.push(`/drawings/${drawingId}/kss`);
        } else if (job.status === 'failed') {
          clearInterval(interval);
          setGenerating(false);
          setGenJobId(null);
        }
      } catch { /* */ }
    }, 1500);
    return () => clearInterval(interval);
  }, [genJobId, drawingId, router]);

  const handleGenerateKss = async () => {
    setGenerating(true);
    try {
      const { job_id } = await api.generateKss(drawingId);
      setGenJobId(job_id);
    } catch {
      setGenerating(false);
    }
  };

  if (loading) {
    return (
      <div className="oe-fade-in">
<div className="max-w-5xl mx-auto px-6 py-12">
          <div className="animate-pulse text-content-tertiary">Loading drawing summary...</div>
        </div>
      </div>
    );
  }

  const drawing = (summary?.drawing ?? {}) as Record<string, unknown>;
  const analysis = (summary?.analysis ?? {}) as Record<string, unknown>;
  const kss = (summary?.kss ?? {}) as Record<string, unknown>;
  const analysisAvailable = analysis?.available === true;
  const kssGenerated = kss?.status === 'generated';
  const filename = (drawing.filename as string) ?? 'Drawing';

  const annotations = (analysis?.annotations as string[]) ?? [];
  const blocks = (analysis?.blocks as { name: string; entity_count: number }[]) ?? [];
  const entitiesPerLayer = (analysis?.entities_per_layer ?? {}) as Record<string, number>;

  return (
    <div className="oe-fade-in">
<div className="max-w-5xl mx-auto px-6 py-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="min-w-0">
            <Breadcrumbs
              items={[
                { label: 'Drawings', href: '/drawings' },
                { label: filename },
              ]}
            />
            <h1 className="text-2xl font-bold mt-2 truncate">{filename}</h1>
            <p className="text-sm text-content-tertiary">
              {drawing.format as string} | {(drawing.entity_count as number) ?? '?'} entities | Uploaded {new Date(drawing.created_at as string).toLocaleDateString('bg-BG')}
            </p>
          </div>
          <div className="flex gap-3">
            <button
              onClick={() => router.push(`/drawings/${drawingId}/viewer`)}
              className="oe-btn-primary oe-btn-lg"
            >
              View Drawing
            </button>
            {kssGenerated ? (
              <button
                onClick={() => router.push(`/drawings/${drawingId}/kss`)}
                className="oe-btn-primary oe-btn-lg"
              >
                View KSS Report
              </button>
            ) : (
              <>
                <button
                  onClick={handleGenerateKss}
                  disabled={generating}
                  className="oe-btn-secondary oe-btn-lg"
                >
                  {generating ? 'Generating…' : 'Standard KSS'}
                </button>
                <button
                  onClick={async () => {
                    try {
                      await api.triggerAiKssResearch(drawingId);
                      router.push(`/drawings/${drawingId}/kss/prepare`);
                    } catch { /* */ }
                  }}
                  className="oe-btn-primary oe-btn-lg"
                >
                  AI KSS (Opus 4.6)
                </button>
              </>
            )}
          </div>
        </div>

        {/* KSS Status Banner */}
        {kssGenerated && (
          <div className="bg-sky-900/20 border border-sky-800/50 rounded-lg p-4 flex items-center justify-between">
            <div>
              <div className="flex items-center gap-2">
                <span className="text-sky-300 font-medium">KSS Report Ready</span>
                {(kss.ai_enhanced as boolean) && (
                  <span className="px-1.5 py-0.5 bg-sky-900/50 text-sky-300 rounded text-[10px] font-medium">AI-enhanced</span>
                )}
              </div>
              <p className="text-sm text-content-secondary mt-1">
                {kss.item_count as number} items | Total: {((kss.total_with_vat_lv as number) ?? 0).toFixed(2)} € (с ДДС)
              </p>
            </div>
            <button
              onClick={() => router.push(`/drawings/${drawingId}/kss`)}
              className="oe-btn-primary"
            >
              Open Report
            </button>
          </div>
        )}

        {/* Drawing Analysis Summary */}
        {analysisAvailable ? (
          <div className="grid grid-cols-2 gap-6">
            {/* Metadata Card */}
            <section className="oe-card p-5 space-y-3">
              <h2 className="text-sm font-semibold text-content-secondary uppercase tracking-wider">Drawing Info</h2>
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div><span className="text-content-tertiary">Format:</span> <span>{drawing.format as string}</span></div>
                <div><span className="text-content-tertiary">Units:</span> <span>{(analysis.insert_units as number) === 6 ? 'Meters' : (analysis.insert_units as number) === 4 ? 'mm' : 'Unitless'}</span></div>
                <div><span className="text-content-tertiary">Layers:</span> <span>{analysis.layer_count as number}</span></div>
                <div><span className="text-content-tertiary">Blocks:</span> <span>{analysis.block_count as number}</span></div>
                <div><span className="text-content-tertiary">Dimensions:</span> <span>{analysis.dimension_count as number ?? 0}</span></div>
                <div><span className="text-content-tertiary">Version:</span> <span>{analysis.version as string ?? '?'}</span></div>
              </div>
            </section>

            {/* Entity Distribution */}
            <section className="oe-card p-5 space-y-3">
              <h2 className="text-sm font-semibold text-content-secondary uppercase tracking-wider">Entity Distribution</h2>
              <div className="space-y-1.5 text-sm max-h-40 overflow-y-auto">
                {Object.entries(analysis.entity_type_counts as Record<string, number> ?? {})
                  .sort(([,a], [,b]) => (b as number) - (a as number))
                  .map(([type_name, count]) => (
                    <div key={type_name} className="flex justify-between">
                      <span className="text-content-secondary">{type_name}</span>
                      <span className="font-mono text-xs">{count as number}</span>
                    </div>
                  ))}
              </div>
            </section>

            {/* Annotations */}
            {annotations.length > 0 && (
              <section className="oe-card p-5 space-y-3">
                <h2 className="text-sm font-semibold text-content-secondary uppercase tracking-wider">Annotations ({annotations.length})</h2>
                <div className="flex flex-wrap gap-2">
                  {annotations.filter(a => a && a !== 'None').slice(0, 20).map((text, i) => (
                    <span key={i} className="px-2 py-1 bg-surface-tertiary rounded text-xs text-content-primary">{text}</span>
                  ))}
                </div>
              </section>
            )}

            {/* Named Blocks */}
            {blocks.length > 0 && (
              <section className="oe-card p-5 space-y-3">
                <h2 className="text-sm font-semibold text-content-secondary uppercase tracking-wider">Blocks ({blocks.length})</h2>
                <div className="space-y-1 text-sm max-h-40 overflow-y-auto">
                  {blocks.slice(0, 15).map((b, i) => (
                    <div key={i} className="flex justify-between">
                      <span className="text-content-primary truncate mr-2">{b.name}</span>
                      <span className="text-content-tertiary text-xs">{b.entity_count} entities</span>
                    </div>
                  ))}
                </div>
              </section>
            )}

            {/* Active Layers */}
            <section className="oe-card p-5 space-y-3 col-span-2">
              <h2 className="text-sm font-semibold text-content-secondary uppercase tracking-wider">Active Layers</h2>
              <div className="flex flex-wrap gap-2">
                {Object.entries(entitiesPerLayer)
                  .sort(([,a], [,b]) => (b as number) - (a as number))
                  .map(([name, count]) => (
                    <span key={name} className="px-2 py-1 bg-surface-tertiary rounded text-xs">
                      <span className="text-content-primary">{name}</span>
                      <span className="text-content-tertiary ml-1">({count})</span>
                    </span>
                  ))}
              </div>
            </section>
          </div>
        ) : (
          <div className="oe-card p-8 text-center text-content-tertiary">
            Deep analysis not available yet. Re-upload the drawing to trigger automatic analysis.
          </div>
        )}
      </div>
    </div>
  );
}
