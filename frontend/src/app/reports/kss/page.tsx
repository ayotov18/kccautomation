'use client';

/**
 * Reports hub. Lists all drawings so the user can jump straight to any
 * KSS report at /drawings/{id}/kss. Kept deliberately simple — the
 * unified hub for drawings is /files; this page just gives the sidebar
 * Reports link a real destination.
 */

import { useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';

export default function KssReportsPage() {
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');

  useEffect(() => {
    api
      .listDrawings()
      .then((d) => setDrawings(d))
      .catch(() => setDrawings([]))
      .finally(() => setLoading(false));
  }, []);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return drawings;
    return drawings.filter((d) => d.filename.toLowerCase().includes(q));
  }, [drawings, search]);

  return (
    <div className="oe-fade-in">
      <div className="max-w-5xl mx-auto px-6 py-8 space-y-6">
        <div className="flex items-baseline justify-between gap-4 flex-wrap">
          <div>
            <h1 className="text-[26px] font-semibold tracking-tight text-content-primary">
              KSS reports
            </h1>
            <p className="mt-1 text-[12.5px] text-content-tertiary">
              Pick a drawing to open its KSS report. New drawings come from{' '}
              <Link href="/files" className="underline hover:text-content-secondary">
                Files
              </Link>
              .
            </p>
          </div>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search drawings…"
            className="bg-surface-tertiary border border-border-light rounded-full px-3 py-1.5 text-sm w-full md:w-72 outline-none"
          />
        </div>

        {loading ? (
          <p className="text-content-tertiary text-sm">Loading…</p>
        ) : filtered.length === 0 ? (
          <p className="text-content-tertiary text-sm italic">
            {search ? 'No matches.' : 'No drawings yet — upload one from Files.'}
          </p>
        ) : (
          <ul className="rounded-lg border border-border-light/50 overflow-hidden divide-y divide-border-light/30">
            {filtered.map((d) => (
              <li key={d.id}>
                <Link
                  href={`/drawings/${d.id}/kss`}
                  className="flex items-center justify-between gap-4 px-4 py-3 hover:bg-surface-secondary/40 transition-colors"
                >
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-sm text-content-primary">
                      {d.filename}
                    </div>
                    <div className="text-[11px] text-content-tertiary mt-0.5">
                      <span className="uppercase">{d.original_format}</span>
                      {d.units && <> · {d.units}</>}
                      {d.entity_count != null && (
                        <> · <span className="font-numeric">{d.entity_count}</span> entities</>
                      )}
                      {' · '}
                      {new Date(d.created_at).toLocaleDateString('en-GB')}
                    </div>
                  </div>
                  <span className="text-xs text-content-tertiary">Open KSS →</span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
