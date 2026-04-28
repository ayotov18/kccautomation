'use client';

/**
 * Unified file manager.
 *
 * One screen for everything the user has uploaded — drawings (DWG/DXF) and
 * price-library imports (XLSX offers). Filter pills switch the view; one
 * search box filters by name or metadata. Upload from the header.
 *
 * Dashboard quick actions deep-link here with `?upload=drawing` or
 * `?upload=offer` and we auto-open the file picker on mount.
 */

import { Suspense, useEffect, useMemo, useRef, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { ArrowUpRight, FileText, Layers, Sparkles, Upload } from 'lucide-react';
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
  delete?: () => Promise<void>;
  badge?: string;
}

export default function FilesPage() {
  return (
    <Suspense fallback={<div className="oe-fade-in p-10 text-content-tertiary text-sm">Loading…</div>}>
      <FilesPageInner />
    </Suspense>
  );
}

function FilesPageInner() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [kind, setKind] = useState<FileKind>(() => {
    const t = searchParams.get('type');
    return t === 'offers' ? 'offer' : t === 'drawings' ? 'drawing' : 'all';
  });
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
  const [uploadingKind, setUploadingKind] = useState<'drawing' | 'offer' | null>(null);
  const [uploadMsg, setUploadMsg] = useState<{ tone: 'ok' | 'err'; text: string } | null>(null);
  const drawingInputRef = useRef<HTMLInputElement | null>(null);
  const offerInputRef = useRef<HTMLInputElement | null>(null);

  const refresh = async () => {
    const [d, i] = await Promise.all([
      api.listDrawings().catch(() => [] as Drawing[]),
      api.listCorpusImports().catch(() => ({ imports: [], total_corpus_rows: 0 })),
    ]);
    setDrawings(d);
    setImports(i.imports);
    setLoading(false);
  };

  useEffect(() => {
    void refresh();
  }, []);

  // Honour ?upload=drawing|offer deep-links from the dashboard.
  useEffect(() => {
    const target = searchParams.get('upload');
    if (target === 'drawing') {
      drawingInputRef.current?.click();
      router.replace('/files', { scroll: false });
    } else if (target === 'offer') {
      offerInputRef.current?.click();
      router.replace('/files', { scroll: false });
    }
  }, [searchParams, router]);

  const handleDrawingUpload = async (file: File) => {
    setUploadingKind('drawing');
    setUploadMsg(null);
    try {
      const result = await api.uploadDrawing(file);
      setUploadMsg({
        tone: 'ok',
        text: result.duplicate
          ? `"${file.name}" already uploaded.`
          : `Uploaded "${file.name}" — analyzing now.`,
      });
      await refresh();
      setTimeout(() => router.push(`/drawings/${result.drawing_id}`), 800);
    } catch (err) {
      setUploadMsg({
        tone: 'err',
        text: `Failed: ${err instanceof Error ? err.message : 'Upload failed'}`,
      });
    }
    setUploadingKind(null);
  };

  const handleOfferUpload = async (file: File) => {
    setUploadingKind('offer');
    setUploadMsg(null);
    try {
      const result = await api.importPriceCorpus(file, { onConflict: 'add' });
      if (result.kind === 'conflict') {
        setUploadMsg({
          tone: 'ok',
          text: `Possible duplicate — open Prices to choose Skip / Replace / Add.`,
        });
      } else {
        setUploadMsg({
          tone: 'ok',
          text: result.deduped
            ? `"${file.name}" already imported (${result.row_count} rows reused).`
            : `Imported ${result.row_count} priced rows from "${file.name}".`,
        });
        await refresh();
      }
    } catch (err) {
      setUploadMsg({
        tone: 'err',
        text: `Failed: ${err instanceof Error ? err.message : 'Upload failed'}`,
      });
    }
    setUploadingKind(null);
  };

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
      delete: async () => {
        await api.deleteDrawing(d.id);
        setDrawings((prev) => prev.filter((x) => x.id !== d.id));
      },
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

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-10 space-y-6">
        {/* Header */}
        <header className="flex items-start justify-between gap-4 flex-wrap">
          <div className="min-w-0 space-y-2">
            <div className="oe-eyebrow">Workspace</div>
            <h1 className="text-[26px] font-semibold tracking-[-0.025em] text-content-primary">
              Files
            </h1>
            <p className="text-[12.5px] text-content-tertiary max-w-xl">
              Drawings and uploaded offers — all in one place. KCC reports open
              from each drawing.
            </p>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <button
              onClick={() => drawingInputRef.current?.click()}
              disabled={uploadingKind !== null}
              className="oe-btn-primary"
            >
              <Upload size={13} />
              {uploadingKind === 'drawing' ? 'Uploading…' : 'Upload drawing'}
            </button>
            <input
              ref={drawingInputRef}
              type="file"
              accept=".dxf,.dwg,application/dxf,application/octet-stream"
              className="hidden"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) handleDrawingUpload(f);
                e.target.value = '';
              }}
            />
            <button
              onClick={() => offerInputRef.current?.click()}
              disabled={uploadingKind !== null}
              className="oe-btn-secondary"
            >
              <Upload size={13} />
              {uploadingKind === 'offer' ? 'Uploading…' : 'Upload offer'}
            </button>
            <input
              ref={offerInputRef}
              type="file"
              accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
              className="hidden"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) handleOfferUpload(f);
                e.target.value = '';
              }}
            />
          </div>
        </header>

        {uploadMsg && (
          <div
            className="text-xs px-3 py-2 rounded-lg"
            style={{
              background:
                uploadMsg.tone === 'err'
                  ? 'var(--oe-error-bg)'
                  : 'var(--oe-success-bg)',
              color:
                uploadMsg.tone === 'err' ? 'var(--oe-error)' : 'var(--oe-success)',
            }}
          >
            {uploadMsg.text}
          </div>
        )}

        {/* Filter pills + search */}
        <div className="flex items-center gap-3 flex-wrap">
          <div className="oe-tab-row">
            {(
              [
                ['all', 'All'],
                ['drawing', 'Drawings'],
                ['offer', 'Offers'],
              ] as Array<[FileKind, string]>
            ).map(([k, label]) => (
              <button
                key={k}
                onClick={() => setKind(k)}
                data-active={kind === k}
                className="oe-tab"
              >
                {label}
                <span className="font-numeric text-[10.5px] opacity-60">
                  {counts[k]}
                </span>
              </button>
            ))}
          </div>
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by name or metadata"
            className="oe-input flex-1 min-w-[200px] !rounded-full"
          />
        </div>

        {/* Files table */}
        <div className="oe-card overflow-hidden">
          {loading ? (
            <div className="p-8 text-xs text-content-tertiary italic">Loading…</div>
          ) : rows.length === 0 ? (
            <EmptyFilesState kind={kind} />
          ) : (
            <table className="oe-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th className="w-24">Type</th>
                  <th>Details</th>
                  <th className="w-24">Date</th>
                  <th className="w-32 !text-right">Actions</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={`${r.kind}-${r.id}`}>
                    <td className="font-medium max-w-sm">
                      <span className="truncate block" title={r.name}>
                        {r.href ? (
                          <Link
                            href={r.href}
                            className="text-content-primary hover:text-content-secondary no-underline"
                          >
                            {r.name}
                          </Link>
                        ) : (
                          <span className="text-content-primary">{r.name}</span>
                        )}
                      </span>
                    </td>
                    <td>
                      <span
                        className="oe-badge"
                        data-variant={r.kind === 'drawing' ? 'accent' : 'info'}
                      >
                        {r.badge ?? (r.kind === 'drawing' ? 'DWG' : 'offer')}
                      </span>
                    </td>
                    <td className="text-[12.5px] text-content-tertiary max-w-md">
                      <span className="truncate block">{r.meta}</span>
                    </td>
                    <td className="text-[12px] text-content-tertiary">
                      {new Date(r.created_at).toLocaleDateString('en-GB', {
                        day: 'numeric',
                        month: 'short',
                        year: 'numeric',
                      })}
                    </td>
                    <td>
                      <div className="inline-flex items-center gap-1 justify-end w-full">
                        {r.href && (
                          <Link href={r.href} className="oe-btn-ghost oe-btn-sm">
                            Open <ArrowUpRight size={11} />
                          </Link>
                        )}
                        {r.delete && (
                          <button
                            onClick={async () => {
                              if (!confirm(`Delete "${r.name}"?`)) return;
                              await r.delete!();
                            }}
                            className="oe-btn-ghost oe-btn-sm"
                            style={{ color: 'var(--oe-error)' }}
                            aria-label="Delete"
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

function EmptyFilesState({ kind }: { kind: FileKind }) {
  const lines: Record<FileKind, { hero: string; sub: string }> = {
    all: {
      hero: 'No files yet.',
      sub: 'Drop a DWG to start, or paste a XLSX offer into your library.',
    },
    drawing: {
      hero: 'No drawings yet.',
      sub: 'Upload a DWG or DXF — multi-module detection runs automatically.',
    },
    offer: {
      hero: 'No offers yet.',
      sub: 'XLSX uploads feed your RAG library and pin to drawings 1:1.',
    },
  };
  const l = lines[kind];
  return (
    <div className="px-6 py-16 text-center">
      <div className="inline-flex w-12 h-12 rounded-full items-center justify-center bg-[color:var(--oe-bg-tertiary)] text-content-tertiary">
        {kind === 'offer' ? <Layers size={20} strokeWidth={1.5} /> : <FileText size={20} strokeWidth={1.5} />}
      </div>
      <h3 className="mt-4 oe-display text-[24px] text-content-secondary">
        {l.hero}
      </h3>
      <p className="mt-1 text-[12.5px] text-content-tertiary max-w-sm mx-auto">
        {l.sub}
      </p>
      <div className="mt-5 flex items-center justify-center gap-2">
        {kind !== 'offer' && (
          <Link href="/files?upload=drawing" className="oe-btn-primary oe-btn-sm">
            <Sparkles size={13} /> Upload drawing
          </Link>
        )}
        {kind !== 'drawing' && (
          <Link href="/files?upload=offer" className="oe-btn-secondary oe-btn-sm">
            Upload offer
          </Link>
        )}
      </div>
    </div>
  );
}
