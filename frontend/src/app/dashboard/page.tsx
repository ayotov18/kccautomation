'use client';

import { useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { ArrowUpRight, FileText, FolderArchive, Tag, Upload, Sparkles } from 'lucide-react';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';
import { Sparkline } from '@/components/ui/Sparkline';

// Synthetic 14-day spark data — proxies usage trend until we wire up
// actual analytics. Stable per session so the cards don't dance on every
// re-render.
function pseudoSpark(seed: number, len = 14): number[] {
  const out: number[] = [];
  let v = seed;
  for (let i = 0; i < len; i++) {
    v = Math.max(0, v + Math.sin(i * 1.3 + seed) * 0.6 + (i / len));
    out.push(v);
  }
  return out;
}

export default function DashboardPage() {
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [offerCount, setOfferCount] = useState(0);
  const [corpusRows, setCorpusRows] = useState(0);

  useEffect(() => {
    api.listDrawings().then(setDrawings).catch(() => {});
    api
      .listCorpusImports()
      .then((d) => {
        setOfferCount(d.imports.length);
        setCorpusRows(d.total_corpus_rows);
      })
      .catch(() => {});
  }, []);

  const drawingCount = drawings.length;
  const recent = useMemo(
    () =>
      [...drawings]
        .sort(
          (a, b) =>
            new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
        )
        .slice(0, 5),
    [drawings],
  );

  const kpis = [
    {
      eyebrow: 'Drawings',
      value: drawingCount,
      hint: 'DWG · DXF imports',
      href: '/files',
      spark: pseudoSpark(drawingCount + 2),
      tone: 'accent' as const,
    },
    {
      eyebrow: 'Offers',
      value: offerCount,
      hint: 'XLSX uploaded to library',
      href: '/files?type=offers',
      spark: pseudoSpark(offerCount + 5),
      tone: 'info' as const,
    },
    {
      eyebrow: 'Priced rows',
      value: corpusRows,
      hint: 'available for RAG',
      href: '/prices',
      spark: pseudoSpark(corpusRows / 50 + 4),
      tone: 'success' as const,
    },
    {
      eyebrow: 'Reports',
      value: drawingCount,
      hint: 'one KCC per drawing',
      href: '/files',
      spark: pseudoSpark(drawingCount + 7),
      tone: 'warning' as const,
    },
  ];

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-10 space-y-10">
        {/* Hero */}
        <header className="space-y-3">
          <div className="oe-eyebrow">Workspace</div>
          <h1 className="text-[34px] leading-[1.05] font-semibold tracking-[-0.025em] text-content-primary">
            Welcome back. <span className="oe-display text-content-secondary">Let&rsquo;s ship a quote.</span>
          </h1>
          <p className="text-[13.5px] text-content-tertiary max-w-xl">
            Two surfaces:{' '}
            <Link href="/files" className="text-content-secondary hover:text-content-primary underline decoration-content-tertiary/40 underline-offset-4">
              Files
            </Link>{' '}
            (drawings + offers, KCC opens from each drawing) and{' '}
            <Link href="/prices" className="text-content-secondary hover:text-content-primary underline decoration-content-tertiary/40 underline-offset-4">
              Prices
            </Link>{' '}
            (your library, defaults, norms).
          </p>
        </header>

        {/* KPI strip */}
        <section className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {kpis.map((k) => (
            <Link
              key={k.eyebrow}
              href={k.href}
              className="oe-kpi group hover:border-[color:var(--oe-border)]"
            >
              <div className="flex items-center justify-between">
                <span className="oe-eyebrow">{k.eyebrow}</span>
                <ArrowUpRight
                  size={13}
                  className="text-content-quaternary opacity-0 group-hover:opacity-100 transition-opacity"
                />
              </div>
              <div className="oe-kpi-value">{k.value.toLocaleString('en-GB')}</div>
              <div className="flex items-end justify-between gap-3 mt-1">
                <span className="oe-kpi-label">{k.hint}</span>
                <span
                  className="flex-none"
                  style={{
                    color:
                      k.tone === 'accent'
                        ? 'var(--oe-accent)'
                        : k.tone === 'info'
                          ? 'var(--oe-info)'
                          : k.tone === 'success'
                            ? 'var(--oe-success)'
                            : 'var(--oe-warning)',
                  }}
                >
                  <Sparkline data={k.spark} width={72} height={22} />
                </span>
              </div>
            </Link>
          ))}
        </section>

        {/* Recent drawings */}
        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <div className="oe-eyebrow">Recent</div>
              <h2 className="oe-section-title mt-1">Drawings</h2>
            </div>
            <Link href="/files" className="oe-btn-ghost oe-btn-sm">
              All files →
            </Link>
          </div>

          {recent.length === 0 ? (
            <EmptyState />
          ) : (
            <div className="oe-card overflow-hidden">
              <table className="oe-table">
                <thead>
                  <tr>
                    <th>Filename</th>
                    <th className="!text-right">Entities</th>
                    <th>Format</th>
                    <th>Imported</th>
                    <th className="w-24"></th>
                  </tr>
                </thead>
                <tbody>
                  {recent.map((d) => (
                    <tr key={d.id}>
                      <td className="font-medium">{d.filename}</td>
                      <td className="oe-num text-content-secondary">
                        {(d.entity_count ?? 0).toLocaleString('en-GB')}
                      </td>
                      <td className="text-content-tertiary uppercase text-[11px] font-numeric">
                        {d.original_format}
                      </td>
                      <td className="text-content-tertiary text-[12px]">
                        {new Date(d.created_at).toLocaleDateString('en-GB')}
                      </td>
                      <td className="text-right">
                        <Link
                          href={`/drawings/${d.id}/kss`}
                          className="text-[12px] text-content-secondary hover:text-content-primary"
                        >
                          Open KCC →
                        </Link>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>

        {/* Quick actions */}
        <section className="space-y-3">
          <div className="oe-eyebrow">Quick actions</div>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <ActionTile
              icon={<Upload size={15} />}
              label="Upload drawing"
              hint="DWG or DXF · multi-module"
              href="/files?upload=drawing"
            />
            <ActionTile
              icon={<Tag size={15} />}
              label="Upload offer"
              hint="XLSX → price corpus"
              href="/files?upload=offer"
            />
            <ActionTile
              icon={<FolderArchive size={15} />}
              label="Browse files"
              hint="drawings + offers"
              href="/files"
            />
          </div>
        </section>
      </div>
    </div>
  );
}

function ActionTile({
  icon,
  label,
  hint,
  href,
}: {
  icon: React.ReactNode;
  label: string;
  hint: string;
  href: string;
}) {
  return (
    <Link
      href={href}
      className="oe-card-interactive flex items-center gap-3 px-4 py-3"
    >
      <span
        className="flex-none w-8 h-8 rounded-full flex items-center justify-center"
        style={{
          background: 'var(--oe-bg-tertiary)',
          color: 'var(--oe-text-secondary)',
        }}
      >
        {icon}
      </span>
      <div className="min-w-0 flex-1">
        <div className="text-[13.5px] text-content-primary truncate">{label}</div>
        <div className="text-[11.5px] text-content-tertiary truncate">{hint}</div>
      </div>
      <ArrowUpRight size={14} className="text-content-quaternary" />
    </Link>
  );
}

function EmptyState() {
  return (
    <div className="oe-card px-6 py-16 text-center">
      <FileText
        size={20}
        className="mx-auto text-content-quaternary"
        strokeWidth={1.5}
      />
      <h3 className="mt-4 oe-display text-[24px] text-content-secondary">
        Nothing here yet.
      </h3>
      <p className="mt-1 text-[12.5px] text-content-tertiary">
        Drop a DWG or paste a XLSX offer to get started.
      </p>
      <div className="mt-5 flex items-center justify-center gap-2">
        <Link href="/files?upload=drawing" className="oe-btn-primary oe-btn-sm">
          <Sparkles size={13} /> Upload drawing
        </Link>
        <Link href="/files?upload=offer" className="oe-btn-secondary oe-btn-sm">
          <Tag size={13} /> Upload offer
        </Link>
      </div>
    </div>
  );
}
