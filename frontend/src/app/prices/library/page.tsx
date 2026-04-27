'use client';

import { useEffect, useState, useCallback } from 'react';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';

interface CorpusImport {
  id: string;
  filename: string;
  sheet_count: number;
  row_count: number;
  skipped_count: number;
  imported_at: string;
  drawing_id: string | null;
  drawing_filename: string | null;
}

interface CorpusRow {
  id: string;
  sek_code: string | null;
  description: string;
  unit: string;
  quantity: number | null;
  material_price_eur: number | null;
  labor_price_eur: number | null;
  total_unit_price_eur: number | null;
  currency: string;
  source_sheet: string | null;
  source_row: number | null;
}

interface ConflictInfo {
  summary: string;
  matches: Array<{
    import_id: string;
    filename: string;
    drawing_id: string | null;
    overlapping_rows: number;
    total_rows: number;
    overlap_pct: number;
    imported_at: string;
  }>;
  pendingFile: File;
  pendingDrawingId: string | null;
}

export default function PriceLibraryPage() {
  const [imports, setImports] = useState<CorpusImport[]>([]);
  const [totalRows, setTotalRows] = useState(0);
  const [rows, setRows] = useState<CorpusRow[]>([]);
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [search, setSearch] = useState('');
  const [uploading, setUploading] = useState(false);
  const [uploadMsg, setUploadMsg] = useState<string | null>(null);
  const [loadingRows, setLoadingRows] = useState(false);
  const [linkDrawingId, setLinkDrawingId] = useState<string>('');
  const [conflict, setConflict] = useState<ConflictInfo | null>(null);

  const fetchImports = useCallback(async () => {
    try {
      const data = await api.listCorpusImports();
      setImports(data.imports);
      setTotalRows(data.total_corpus_rows);
    } catch {
      /* swallow */
    }
  }, []);

  const fetchRows = useCallback(async (q: string) => {
    setLoadingRows(true);
    try {
      const data = await api.listCorpus({ q: q || undefined, limit: 100, offset: 0 });
      setRows(data.rows);
    } catch {
      setRows([]);
    }
    setLoadingRows(false);
  }, []);

  useEffect(() => {
    fetchImports();
    fetchRows('');
    api.listDrawings().then(setDrawings).catch(() => setDrawings([]));
  }, [fetchImports, fetchRows]);

  const performUpload = async (
    file: File,
    drawingId: string | null,
    onConflict: 'warn' | 'add' | 'replace' | 'skip',
  ) => {
    setUploading(true);
    setUploadMsg(null);
    try {
      const result = await api.importPriceCorpus(file, { drawingId, onConflict });
      if (result.kind === 'conflict') {
        setConflict({
          summary: result.summary,
          matches: result.matches,
          pendingFile: file,
          pendingDrawingId: drawingId,
        });
        setUploading(false);
        return;
      }
      if (result.deduped) {
        setUploadMsg(`Already imported — ${result.row_count} rows reused${
          result.drawing_id ? ' (link updated)' : ''
        }.`);
      } else {
        const overlapNote =
          result.overlap_warnings && result.overlap_warnings.length > 0
            ? ` ${result.overlap_warnings.length} overlap${result.overlap_warnings.length === 1 ? '' : 's'} detected — kept anyway.`
            : '';
        setUploadMsg(
          `Imported ${result.row_count} rows from ${result.sheet_count} sheet${
            result.sheet_count === 1 ? '' : 's'
          }${result.skipped_count > 0 ? `, ${result.skipped_count} skipped` : ''}.${overlapNote}`,
        );
      }
      await fetchImports();
      await fetchRows(search);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Upload failed';
      setUploadMsg(`Failed: ${msg}`);
    }
    setUploading(false);
    setConflict(null);
  };

  const handleFile = (file: File) => {
    void performUpload(file, linkDrawingId || null, 'warn');
  };

  const handleConflictResolve = (action: 'add' | 'replace' | 'skip') => {
    if (!conflict) return;
    if (action === 'skip') {
      setConflict(null);
      setUploadMsg('Upload cancelled — using the existing import.');
      return;
    }
    void performUpload(conflict.pendingFile, conflict.pendingDrawingId, action);
  };

  const handleDelete = async (importId: string, filename: string) => {
    if (!confirm(`Delete import "${filename}" and all its corpus rows?`)) return;
    try {
      await api.deleteCorpusImport(importId);
      await fetchImports();
      await fetchRows(search);
    } catch {
      /* swallow */
    }
  };

  const handleRelink = async (importId: string, drawingId: string) => {
    try {
      await api.setImportLink(importId, drawingId || null);
      await fetchImports();
    } catch {
      /* swallow */
    }
  };

  const onSearchChange = (q: string) => {
    setSearch(q);
    fetchRows(q);
  };

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
        <div>
          <h1 className="text-[26px] font-semibold tracking-tight text-content-primary">
            My price library
          </h1>
          <p className="mt-1 text-[12.5px] text-content-tertiary">
            Upload past offers (XLSX). Pin each one to its drawing so RAG generation for that
            drawing matches your reference 1:1. Total in library:{' '}
            <span className="font-numeric text-content-secondary">{totalRows}</span> priced
            rows.
          </p>
        </div>

        {/* Upload card */}
        <div className="oe-card p-5 space-y-3">
          <div className="flex items-center justify-between gap-4 flex-wrap">
            <div>
              <div className="text-sm font-medium">Upload an offer (XLSX)</div>
              <div className="text-xs text-content-tertiary mt-1 max-w-md">
                Header row with “Описание / м.ед. / Колич. / Ед. Цена мат / Цена мат / Монтаж
                / Цена монтаж / Общо”. Multi-sheet workbooks import as separate modules.
              </div>
            </div>
            <label
              htmlFor="corpus-upload"
              className="oe-btn-primary cursor-pointer whitespace-nowrap"
            >
              {uploading ? 'Uploading…' : 'Choose file'}
              <input
                id="corpus-upload"
                type="file"
                accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                className="hidden"
                disabled={uploading}
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) handleFile(f);
                  e.target.value = '';
                }}
              />
            </label>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <span className="text-content-tertiary">Link to drawing</span>
            <select
              value={linkDrawingId}
              onChange={(e) => setLinkDrawingId(e.target.value)}
              className="bg-surface-tertiary border border-border-light rounded-full px-3 py-1 text-content-primary outline-none"
            >
              <option value="">— No link (whole-corpus RAG)</option>
              {drawings.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.filename}
                </option>
              ))}
            </select>
          </div>
          {uploadMsg && (
            <div
              className={`text-xs px-3 py-2 rounded-lg ${
                uploadMsg.startsWith('Failed')
                  ? 'bg-red-900/30 text-red-300'
                  : 'bg-emerald-900/30 text-emerald-300'
              }`}
            >
              {uploadMsg}
            </div>
          )}
        </div>

        {/* Imports table */}
        <div className="oe-card p-5">
          <h2 className="text-sm font-medium mb-3">Import history</h2>
          {imports.length === 0 ? (
            <p className="text-xs text-content-tertiary italic">No uploads yet.</p>
          ) : (
            <table className="w-full text-sm">
              <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider">
                <tr className="border-b border-border-light/60">
                  <th className="px-2 py-2">File</th>
                  <th className="px-2 py-2 w-44">Linked drawing</th>
                  <th className="px-2 py-2 w-16 text-right">Sheets</th>
                  <th className="px-2 py-2 w-16 text-right">Rows</th>
                  <th className="px-2 py-2 w-24">Imported</th>
                  <th className="px-2 py-2 w-12"></th>
                </tr>
              </thead>
              <tbody>
                {imports.map((imp) => (
                  <tr key={imp.id} className="border-b border-border-light/30">
                    <td className="px-2 py-2 truncate max-w-xs" title={imp.filename}>
                      {imp.filename}
                    </td>
                    <td className="px-2 py-2">
                      <select
                        value={imp.drawing_id ?? ''}
                        onChange={(e) => handleRelink(imp.id, e.target.value)}
                        className="w-full bg-transparent border border-border-light/40 rounded-full px-2 py-1 text-[12px] text-content-secondary outline-none hover:border-border-light"
                      >
                        <option value="">— None</option>
                        {drawings.map((d) => (
                          <option key={d.id} value={d.id}>
                            {d.filename}
                          </option>
                        ))}
                      </select>
                    </td>
                    <td className="px-2 py-2 text-right font-numeric text-xs">
                      {imp.sheet_count}
                    </td>
                    <td className="px-2 py-2 text-right font-numeric text-xs">
                      {imp.row_count}
                    </td>
                    <td className="px-2 py-2 text-xs text-content-tertiary">
                      {new Date(imp.imported_at).toLocaleDateString('en-GB')}
                    </td>
                    <td className="px-2 py-2 text-right">
                      <button
                        onClick={() => handleDelete(imp.id, imp.filename)}
                        className="text-xs text-red-400 hover:text-red-300"
                        title="Delete this import + all its rows"
                      >
                        ✕
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Corpus row browser */}
        <div className="oe-card p-5">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-medium">Library content</h2>
            <input
              type="text"
              value={search}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Search (KVH, OSB, дограма, …)"
              className="bg-surface-tertiary border border-border-light rounded-full px-3 py-1.5 text-sm w-72 outline-none"
            />
          </div>
          {loadingRows ? (
            <p className="text-xs text-content-tertiary italic">Loading…</p>
          ) : rows.length === 0 ? (
            <p className="text-xs text-content-tertiary italic">
              {search ? 'No matches.' : 'No rows yet.'}
            </p>
          ) : (
            <table className="w-full text-sm">
              <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider">
                <tr className="border-b border-border-light/60">
                  <th className="px-2 py-2">Description</th>
                  <th className="px-2 py-2 w-14">Unit</th>
                  <th className="px-2 py-2 w-24 text-right">Material</th>
                  <th className="px-2 py-2 w-24 text-right">Labour</th>
                  <th className="px-2 py-2 w-24 text-right">Total</th>
                  <th className="px-2 py-2 w-32 text-content-tertiary">Sheet</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr
                    key={r.id}
                    className="border-b border-border-light/30 hover:bg-surface-secondary/30"
                  >
                    <td
                      className="px-2 py-1.5 text-content-secondary truncate max-w-md"
                      title={r.description}
                    >
                      {r.description}
                    </td>
                    <td className="px-2 py-1.5 text-xs">{r.unit}</td>
                    <td className="px-2 py-1.5 text-right font-numeric text-xs">
                      {r.material_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td className="px-2 py-1.5 text-right font-numeric text-xs">
                      {r.labor_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td className="px-2 py-1.5 text-right font-numeric text-xs font-medium">
                      {r.total_unit_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td
                      className="px-2 py-1.5 text-[11px] text-content-tertiary truncate"
                      title={r.source_sheet ?? ''}
                    >
                      {r.source_sheet ?? '—'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Conflict modal */}
      {conflict && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4">
          <div className="oe-card max-w-lg w-full p-5">
            <h3 className="text-base font-semibold text-content-primary">
              Possible duplicate detected
            </h3>
            <p className="text-xs text-content-tertiary mt-2">{conflict.summary}</p>
            <ul className="mt-3 space-y-2 text-xs">
              {conflict.matches.map((m) => (
                <li
                  key={m.import_id}
                  className="rounded-lg border border-border-light/50 px-3 py-2 flex items-center justify-between gap-3"
                >
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-content-primary">{m.filename}</div>
                    <div className="text-[11px] text-content-tertiary">
                      <span className="font-numeric">
                        {m.overlapping_rows}
                      </span>
                      /<span className="font-numeric">{m.total_rows}</span> rows match —
                      <span className="font-numeric"> {m.overlap_pct.toFixed(0)}%</span>{' '}
                      overlap · {new Date(m.imported_at).toLocaleDateString('en-GB')}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-end gap-2 mt-4">
              <button
                onClick={() => handleConflictResolve('skip')}
                className="oe-btn-ghost"
              >
                Skip — keep existing
              </button>
              <button
                onClick={() => handleConflictResolve('replace')}
                className="oe-btn-secondary"
              >
                Replace
              </button>
              <button
                onClick={() => handleConflictResolve('add')}
                className="oe-btn-primary"
              >
                Add anyway
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
