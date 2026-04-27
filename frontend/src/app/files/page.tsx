'use client';

/**
 * Unified file manager.
 *
 * One screen for everything the user has uploaded or generated:
 *   - Drawings (DWG/DXF source files)
 *   - KSS reports (Excel exports)
 *   - Price library imports (XLSX offers, drawing-linked when applicable)
 *
 * Eliminates the "where is my file?" hunt across /drawings, /reports, and
 * /prices. Kind filter switches the view; everything is one searchable
 * table with consistent actions.
 */

import { useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';

type FileKind = 'all' | 'drawing' | 'offer';

interface UnifiedFileRow {
  kind: 'drawing' | 'offer';
  id: string;
  name: string;
  meta: string;
  created_at: string;
  href?: string;
  download?: () => Promise<Blob>;
  delete?: () => Promise<void>;
  badge?: string;
}

export default function FilesPage() {
  const [kind, setKind] = useState<FileKind>('all');
  const [search, setSearch] = useState('');
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [imports, setImports] = useState<
    Array<{
      id: string;
      filename: string;
      sheet_count: number;
      row_count: number;
      imported_at: string;
      drawing_id: string | null;
      drawing_filename: string | null;
    }>
  >([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void Promise.all([
      api.listDrawings().catch(() => [] as Drawing[]),
      api.listCorpusImports().catch(() => ({ imports: [], total_corpus_rows: 0 })),
    ]).then(([d, i]) => {
      setDrawings(d);
      setImports(i.imports);
      setLoading(false);
    });
  }, []);

  const rows: UnifiedFileRow[] = useMemo(() => {
    const drawingRows: UnifiedFileRow[] = drawings.map((d) => ({
      kind: 'drawing',
      id: d.id,
      name: d.filename,
      meta: `${d.original_format ?? 'dwg'} · ${
        d.entity_count ? d.entity_count.toLocaleString('en-GB') + ' entities' : '–'
      }`,
      created_at: d.created_at,
      href: `/drawings/${d.id}`,
    }));
    const offerRows: UnifiedFileRow[] = imports.map((imp) => ({
      kind: 'offer',
      id: imp.id,
      name: imp.filename,
      meta: `${imp.sheet_count} sheet${imp.sheet_count === 1 ? '' : 's'} · ${
        imp.row_count
      } priced rows${imp.drawing_filename ? ` · linked to ${imp.drawing_filename}` : ''}`,
      created_at: imp.imported_at,
      delete: async () => {
        await api.deleteCorpusImport(imp.id);
        setImports((prev) => prev.filter((p) => p.id !== imp.id));
      },
      badge: imp.drawing_id ? 'linked' : 'corpus',
    }));

    const all = [...drawingRows, ...offerRows].sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );
    const filtered = kind === 'all' ? all : all.filter((r) => r.kind === kind);
    if (!search.trim()) return filtered;
    const q = search.toLowerCase();
    return filtered.filter(
      (r) => r.name.toLowerCase().includes(q) || r.meta.toLowerCase().includes(q),
    );
  }, [drawings, imports, kind, search]);

  const counts = useMemo(() => {
    const c = { all: 0, drawing: 0, offer: 0 };
    c.drawing = drawings.length;
    c.offer = imports.length;
    c.all = c.drawing + c.offer;
    return c;
  }, [drawings, imports]);

  const downloadBlob = async (blob: Blob, filename: string) => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
        <div>
          <h1 className="text-[26px] font-semibold tracking-tight text-content-primary">Files</h1>
          <p className="mt-1 text-[12.5px] text-content-tertiary">
            Drawings, KSS reports, and uploaded offers — all in one place.
          </p>
        </div>

        {/* Filter pills + search */}
        <div className="oe-card p-3 flex items-center gap-3 flex-wrap">
          <div className="inline-flex rounded-full border border-border-light/60 p-0.5 bg-surface-secondary/40">
            {([
              ['all', 'All'],
              ['drawing', 'Drawings'],
              ['offer', 'Offers'],
            ] as Array<[FileKind, string]>).map(([k, label]) => {
              const active = kind === k;
              return (
                <button
                  key={k}
                  onClick={() => setKind(k)}
                  className={`px-3 py-1 text-xs rounded-full transition-colors ${
                    active
                      ? 'bg-content-primary text-content-inverse font-medium'
                      : 'text-content-tertiary hover:text-content-secondary'
                  }`}
                >
                  {label}
                  <span className="ml-1.5 font-numeric text-[10px] opacity-70">
                    {counts[k]}
                  </span>
                </button>
              );
            })}
          </div>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by name or metadata"
            className="flex-1 min-w-[200px] bg-surface-tertiary border border-border-light rounded-full px-3 py-1.5 text-sm outline-none"
          />
        </div>

        {/* Files table */}
        <div className="oe-card overflow-hidden">
          {loading ? (
            <div className="p-6 text-xs text-content-tertiary italic">Loading…</div>
          ) : rows.length === 0 ? (
            <div className="p-12 text-center">
              <p className="text-sm text-content-tertiary">
                {kind === 'all'
                  ? 'No files yet. Upload a drawing or an offer to get started.'
                  : `No ${kind === 'drawing' ? 'drawings' : 'offers'} yet.`}
              </p>
              {kind === 'all' && (
                <div className="mt-4 flex items-center justify-center gap-2">
                  <Link href="/drawings/upload" className="oe-btn-primary oe-btn-sm">
                    Upload drawing
                  </Link>
                  <Link href="/prices" className="oe-btn-secondary oe-btn-sm">
                    Upload offer
                  </Link>
                </div>
              )}
            </div>
          ) : (
            <table className="w-full text-sm">
              <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider bg-surface-secondary/40">
                <tr>
                  <th className="px-4 py-2.5">Name</th>
                  <th className="px-4 py-2.5 w-24">Type</th>
                  <th className="px-4 py-2.5">Details</th>
                  <th className="px-4 py-2.5 w-24">Date</th>
                  <th className="px-4 py-2.5 w-28 text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr
                    key={`${r.kind}-${r.id}`}
                    className="border-t border-border-light/30 hover:bg-surface-secondary/20"
                  >
                    <td className="px-4 py-2.5 truncate max-w-sm" title={r.name}>
                      {r.href ? (
                        <Link
                          href={r.href}
                          className="text-content-primary hover:text-sky-300"
                        >
                          {r.name}
                        </Link>
                      ) : (
                        <span className="text-content-primary">{r.name}</span>
                      )}
                    </td>
                    <td className="px-4 py-2.5">
                      <KindBadge kind={r.kind} customLabel={r.badge} />
                    </td>
                    <td className="px-4 py-2.5 text-[12.5px] text-content-tertiary truncate max-w-md">
                      {r.meta}
                    </td>
                    <td className="px-4 py-2.5 text-[12px] text-content-tertiary">
                      {new Date(r.created_at).toLocaleDateString('en-GB', {
                        day: 'numeric',
                        month: 'short',
                        year: 'numeric',
                      })}
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="inline-flex items-center gap-1.5">
                        {r.href && (
                          <Link href={r.href} className="oe-btn-ghost oe-btn-sm">
                            Open
                          </Link>
                        )}
                        {r.download && (
                          <button
                            onClick={async () => {
                              const blob = await r.download!();
                              downloadBlob(blob, `${r.name}.xlsx`);
                            }}
                            className="oe-btn-ghost oe-btn-sm"
                          >
                            Download
                          </button>
                        )}
                        {r.delete && (
                          <button
                            onClick={async () => {
                              if (!confirm(`Delete "${r.name}"?`)) return;
                              await r.delete!();
                            }}
                            className="text-[11px] text-red-400 hover:text-red-300 px-2"
                          >
                            ✕
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}

function KindBadge({
  kind,
  customLabel,
}: {
  kind: 'drawing' | 'offer';
  customLabel?: string;
}) {
  const styles: Record<typeof kind, string> = {
    drawing: 'bg-sky-500/10 text-sky-300 ring-1 ring-sky-500/20',
    offer: 'bg-amber-500/10 text-amber-200 ring-1 ring-amber-500/20',
  };
  const label = customLabel ?? (kind === 'drawing' ? 'DWG' : 'offer');
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-[10.5px] uppercase tracking-wider font-medium ${styles[kind]}`}
    >
      {label}
    </span>
  );
}
