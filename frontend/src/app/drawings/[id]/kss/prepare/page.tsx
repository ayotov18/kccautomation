'use client';

import { useEffect, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { api } from '@/lib/api';
import type { AiResearchItem } from '@/types';

export default function AiKssPrepare() {
  const params = useParams();
  const router = useRouter();
  const drawingId = params.id as string;

  const [status, setStatus] = useState<string>('loading');
  const [progress, setProgress] = useState(0);
  const [items, setItems] = useState<AiResearchItem[]>([]);
  const [generating, setGenerating] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Poll status during research phase
  useEffect(() => {
    const poll = setInterval(async () => {
      try {
        const s = await api.getAiKssStatus(drawingId);
        setStatus(s.status);
        setProgress(s.progress);

        if (s.status === 'ready') {
          clearInterval(poll);
          // Load research items from Redis
          const researchItems = await api.getAiKssResearchItems(drawingId);
          setItems(researchItems);
        } else if (s.status === 'complete') {
          clearInterval(poll);
          router.push(`/drawings/${drawingId}/kss`);
        } else if (s.status === 'failed') {
          clearInterval(poll);
          setErrorMsg(s.error ?? 'Pipeline failed without an error message.');
        }
      } catch { /* session not found yet */ }
    }, 2000);
    return () => clearInterval(poll);
  }, [drawingId, router]);

  const handleToggle = async (itemId: string, approved: boolean) => {
    setItems(prev => prev.map(i => i.id === itemId ? { ...i, approved } : i));
    await api.updateAiKssItem(drawingId, itemId, { approved: approved.toString() });
  };

  const handleEdit = async (itemId: string, field: string, value: string) => {
    setItems(prev => prev.map(i => i.id === itemId ? { ...i, [field]: value, edited: true } : i));
    await api.updateAiKssItem(drawingId, itemId, { [field]: value });
  };

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      await api.triggerAiKssGeneration(drawingId);
      setStatus('generating');
      // Poll for completion
      const poll = setInterval(async () => {
        const s = await api.getAiKssStatus(drawingId);
        if (s.status === 'complete') {
          clearInterval(poll);
          router.push(`/drawings/${drawingId}/kss`);
        }
      }, 2000);
    } catch {
      setGenerating(false);
    }
  };

  // Group items by SEK group
  const grouped = items.reduce((acc, item) => {
    const group = item.sek_group || 'Other';
    if (!acc[group]) acc[group] = [];
    acc[group].push(item);
    return acc;
  }, {} as Record<string, AiResearchItem[]>);

  const approvedCount = items.filter(i => i.approved).length;

  if (status === 'failed') {
    const isAuth = /401|user not found|unauthori[sz]ed|api key/i.test(errorMsg ?? '');
    return (
      <div className="oe-fade-in">
        <div className="max-w-2xl mx-auto px-6 py-12">
          <div className="rounded-xl border border-red-500/30 bg-red-500/5 p-6">
            <h2 className="text-xl font-bold text-red-300 mb-2">Price research failed</h2>
            <p className="text-content-secondary text-sm mb-4">
              {isAuth
                ? 'The OpenRouter API key is rejected (401). Rotate OPENROUTER_API_KEY in the worker service and try again.'
                : 'The KSS research pipeline could not finish. Details below — check the worker logs for the full trace.'}
            </p>
            <pre className="text-xs text-red-200 bg-black/30 rounded p-3 overflow-x-auto whitespace-pre-wrap break-all">
              {errorMsg}
            </pre>
            <div className="mt-4 flex gap-2">
              <button
                onClick={() => router.push(`/drawings/${drawingId}`)}
                className="px-3 py-1.5 text-sm rounded-lg bg-surface-tertiary hover:bg-gray-700"
              >
                Back to drawing
              </button>
              <button
                onClick={() => location.reload()}
                className="px-3 py-1.5 text-sm rounded-lg bg-sky-500/20 hover:bg-sky-500/30 text-sky-200"
              >
                Retry
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (status === 'loading' || status === 'researching') {
    return (
      <div className="oe-fade-in">
<div className="max-w-4xl mx-auto px-6 py-12 text-center">
          <div className="mb-6">
            <div className="w-16 h-16 mx-auto mb-4 border-4 border-sky-500 border-t-transparent rounded-full animate-spin" />
            <h2 className="text-xl font-bold mb-2">Perplexity is researching prices...</h2>
            <p className="text-content-tertiary text-sm">Searching Bulgarian construction price sources</p>
          </div>
          <div className="w-full max-w-md mx-auto bg-surface-tertiary rounded-full h-2">
            <div className="bg-sky-500 h-2 rounded-full transition-all" style={{ width: `${progress}%` }} />
          </div>
          <p className="text-xs text-content-tertiary mt-2">{progress}%</p>
        </div>
      </div>
    );
  }

  if (status === 'generating') {
    return (
      <div className="oe-fade-in">
<div className="max-w-4xl mx-auto px-6 py-12 text-center">
          <div className="w-16 h-16 mx-auto mb-4 border-4 border-sky-400 border-t-transparent rounded-full animate-spin" />
          <h2 className="text-xl font-bold mb-2">Opus 4.6 is generating your KSS...</h2>
          <p className="text-content-tertiary text-sm">Creating Количествено-Стойностна Сметка from reviewed data</p>
        </div>
      </div>
    );
  }

  return (
    <div className="oe-fade-in">
<div className="max-w-5xl mx-auto px-6 py-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <button onClick={() => router.push(`/drawings/${drawingId}`)} className="text-xs text-content-tertiary hover:text-content-primary mb-2 block">&larr; Back to Drawing</button>
            <h1 className="text-2xl font-bold">
              AI Price Research
              <span className="ml-2 px-2 py-1 bg-sky-900/40 text-sky-300 rounded text-xs font-medium">Perplexity</span>
            </h1>
            <p className="text-sm text-content-tertiary mt-1">Review, edit, and approve prices before generating KSS with Opus 4.6</p>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-xs text-content-tertiary">{approvedCount} of {items.length} items approved</span>
            <button
              onClick={handleGenerate}
              disabled={generating || approvedCount === 0}
              className="px-5 py-2.5 bg-sky-500/90 hover:bg-sky-400 disabled:bg-gray-700 disabled:text-gray-500 text-gray-900 rounded-lg text-sm font-medium transition-colors"
            >
              {generating ? 'Generating...' : 'Generate KSS with Opus 4.6'}
            </button>
          </div>
        </div>

        {/* Research items grouped by SEK group */}
        {Object.entries(grouped).sort(([a], [b]) => a.localeCompare(b)).map(([group, groupItems]) => (
          <section key={group} className="oe-card overflow-hidden">
            <div className="px-4 py-3 bg-surface-tertiary/50 font-bold text-sm text-content-primary">
              {group}
            </div>
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-content-tertiary text-xs border-b border-border-light">
                  <th className="px-4 py-2 w-8">✓</th>
                  <th className="px-4 py-2">Описание</th>
                  <th className="px-4 py-2 w-16">Ед.</th>
                  <th className="px-4 py-2 w-24 text-right">Материал</th>
                  <th className="px-4 py-2 w-24 text-right">Труд</th>
                  <th className="px-4 py-2 w-28 text-right">Ед. цена €</th>
                  <th className="px-4 py-2 w-32 text-right text-content-tertiary">Пазарен диапазон</th>
                  <th className="px-4 py-2 w-24">Източник</th>
                  <th className="px-4 py-2 w-16 text-right">Conf.</th>
                </tr>
              </thead>
              <tbody>
                {groupItems.map(item => {
                  const total = item.price_lv ?? (
                    (item.material_price_lv ?? 0) + (item.labor_price_lv ?? 0)
                  );
                  const hasRange =
                    item.price_min_lv != null &&
                    item.price_max_lv != null &&
                    item.price_min_lv < item.price_max_lv;
                  return (
                    <tr key={item.id} className={`border-b border-border-light/30 ${!item.approved ? 'opacity-40' : ''} ${item.edited ? 'bg-sky-900/10' : ''}`}>
                      <td className="px-4 py-2">
                        <input
                          type="checkbox"
                          checked={item.approved}
                          onChange={e => handleToggle(item.id, e.target.checked)}
                          className="accent-sky-500"
                        />
                      </td>
                      <td className="px-4 py-2">
                        <input
                          type="text"
                          value={item.description}
                          onChange={e => handleEdit(item.id, 'description', e.target.value)}
                          className="bg-transparent border-none outline-none w-full text-content-primary focus:text-sky-200"
                          title={item.notes ?? undefined}
                        />
                      </td>
                      <td className="px-4 py-2 text-content-secondary">{item.unit}</td>
                      <td className="px-4 py-2 text-right">
                        <input
                          type="number"
                          step="0.01"
                          value={item.material_price_lv ?? ''}
                          onChange={e => handleEdit(item.id, 'material_price_lv', e.target.value)}
                          className="bg-transparent border-none outline-none w-20 text-right text-content-secondary focus:text-sky-200 font-mono"
                        />
                      </td>
                      <td className="px-4 py-2 text-right">
                        <input
                          type="number"
                          step="0.01"
                          value={item.labor_price_lv ?? ''}
                          onChange={e => handleEdit(item.id, 'labor_price_lv', e.target.value)}
                          className="bg-transparent border-none outline-none w-20 text-right text-content-secondary focus:text-sky-200 font-mono"
                        />
                      </td>
                      <td className="px-4 py-2 text-right font-mono text-sky-200">
                        {total > 0 ? total.toFixed(2) : '—'}
                      </td>
                      <td className="px-4 py-2 text-right text-content-tertiary text-xs font-mono">
                        {hasRange
                          ? `${item.price_min_lv!.toFixed(0)}–${item.price_max_lv!.toFixed(0)}`
                          : '—'}
                      </td>
                      <td className="px-4 py-2">
                        {item.source_url ? (
                          <a
                            href={item.source_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-sky-300 hover:text-sky-200 text-xs underline decoration-dotted"
                            title={item.source_url}
                          >
                            link
                          </a>
                        ) : (
                          <span className="text-content-tertiary text-xs">—</span>
                        )}
                      </td>
                      <td className="px-4 py-2 text-right text-content-tertiary">
                        {item.confidence != null ? `${(item.confidence * 100).toFixed(0)}%` : '-'}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </section>
        ))}
      </div>
    </div>
  );
}
