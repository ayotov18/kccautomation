'use client';

/**
 * Self-hosted RAG price library.
 *
 * Lives inside the unified `/prices` page. Three responsibilities:
 *   1. Upload XLSX offers (user's prior KSS reports) into the corpus.
 *   2. Pin each upload to a specific drawing so RAG retrieves 1:1 against
 *      that offer when the drawing's KSS is generated.
 *   3. Surface duplicate-detection feedback (file-hash silent skip, content
 *      overlap >= 50% → conflict modal with three options).
 */

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

export function PriceLibrarySection() {
  const [imports, setImports] = useState<CorpusImport[]>([]);
  const [totalRows, setTotalRows] = useState(0);
  const [rows, setRows] = useState<CorpusRow[]>([]);
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [search, setSearch] = useState('');
  const [showBrowser, setShowBrowser] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [uploadMsg, setUploadMsg] = useState<string | null>(null);
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
    try {
      const data = await api.listCorpus({ q: q || undefined, limit: 100, offset: 0 });
      setRows(data.rows);
    } catch {
      setRows([]);
    }
  }, []);

  useEffect(() => {
    fetchImports();
    api.listDrawings().then(setDrawings).catch(() => setDrawings([]));
  }, [fetchImports]);

  useEffect(() => {
    if (showBrowser) fetchRows(search);
  }, [showBrowser, search, fetchRows]);

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
        setUploadMsg(
          `Already imported — ${result.row_count} rows reused${
            result.drawing_id ? ' (link updated)' : ''
          }.`,
        );
      } else {
        const overlapNote =
          result.overlap_warnings && result.overlap_warnings.length > 0
            ? ` ${result.overlap_warnings.length} overlap${
                result.overlap_warnings.length === 1 ? '' : 's'
              } detected — kept anyway.`
            : '';
        setUploadMsg(
          `Imported ${result.row_count} rows from ${result.sheet_count} sheet${
            result.sheet_count === 1 ? '' : 's'
          }${result.skipped_count > 0 ? `, ${result.skipped_count} skipped` : ''}.${overlapNote}`,
        );
      }
      await fetchImports();
      if (showBrowser) await fetchRows(search);
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
      if (showBrowser) await fetchRows(search);
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

  // Inline edit handlers (debounced auto-save per cell on blur).
  const handleEdit = async (
    rowId: string,
    field:
      | 'description'
      | 'unit'
      | 'quantity'
      | 'material_price_eur'
      | 'labor_price_eur'
      | 'sek_code',
    value: string | number,
  ) => {
    setRows((prev) =>
      prev.map((r) =>
        r.id === rowId
          ? {
              ...r,
              [field]: value,
              total_unit_price_eur:
                field === 'material_price_eur' || field === 'labor_price_eur'
                  ? (field === 'material_price_eur'
                      ? Number(value)
                      : r.material_price_eur ?? 0) +
                    (field === 'labor_price_eur'
                      ? Number(value)
                      : r.labor_price_eur ?? 0)
                  : r.total_unit_price_eur,
            }
          : r,
      ),
    );
    try {
      await api.updateCorpusRow(rowId, { [field]: value });
      await fetchImports(); // row counts may not change but keep summary fresh
    } catch {
      /* swallow — UI already optimistic */
    }
  };

  const handleDeleteRow = async (rowId: string) => {
    if (!confirm('Delete this row?')) return;
    try {
      await api.deleteCorpusRow(rowId);
      setRows((prev) => prev.filter((r) => r.id !== rowId));
      await fetchImports();
    } catch {
      /* swallow */
    }
  };

  return (
    <section className="oe-card p-5 space-y-4">
      <div className="flex items-baseline justify-between gap-4 flex-wrap">
        <div>
          <h2 className="text-base font-medium text-content-primary">My price library</h2>
          <p className="mt-1 text-[12.5px] text-content-tertiary">
            <span className="font-numeric text-content-secondary">{totalRows}</span> priced
            rows from <span className="font-numeric">{imports.length}</span> uploaded
            {imports.length === 1 ? ' offer' : ' offers'}. Link each XLSX to a drawing
            for 1:1 RAG generation. Click any cell in the browser to edit.
          </p>
        </div>
        <button
          onClick={() => setShowBrowser((v) => !v)}
          className="oe-btn-ghost oe-btn-sm"
        >
          {showBrowser ? 'Hide' : 'Browse & edit'} prices
        </button>
      </div>

      {/* Upload row */}
      <div className="flex items-center gap-3 flex-wrap">
        <label htmlFor="corpus-upload" className="oe-btn-primary cursor-pointer">
          {uploading ? 'Uploading…' : 'Upload XLSX offer'}
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
        <span className="text-xs text-content-tertiary">Link to drawing</span>
        <select
          value={linkDrawingId}
          onChange={(e) => setLinkDrawingId(e.target.value)}
          className="bg-surface-tertiary border border-border-light rounded-full px-3 py-1.5 text-xs text-content-primary outline-none"
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

      {/* Imports list */}
      {imports.length > 0 && (
        <div className="rounded-lg border border-border-light/50 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider bg-surface-secondary/40">
              <tr>
                <th className="px-3 py-2">File</th>
                <th className="px-3 py-2 w-44">Linked drawing</th>
                <th className="px-3 py-2 w-16 text-right">Sheets</th>
                <th className="px-3 py-2 w-16 text-right">Rows</th>
                <th className="px-3 py-2 w-24">Imported</th>
                <th className="px-3 py-2 w-12"></th>
              </tr>
            </thead>
            <tbody>
              {imports.map((imp) => (
                <tr key={imp.id} className="border-t border-border-light/30">
                  <td className="px-3 py-2 truncate max-w-xs" title={imp.filename}>
                    {imp.filename}
                  </td>
                  <td className="px-3 py-2">
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
                  <td className="px-3 py-2 text-right font-numeric text-xs">
                    {imp.sheet_count}
                  </td>
                  <td className="px-3 py-2 text-right font-numeric text-xs">
                    {imp.row_count}
                  </td>
                  <td className="px-3 py-2 text-xs text-content-tertiary">
                    {new Date(imp.imported_at).toLocaleDateString('en-GB')}
                  </td>
                  <td className="px-3 py-2 text-right">
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
        </div>
      )}

      {/* Inline browser, opt-in */}
      {showBrowser && (
        <div className="space-y-3">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search corpus (KVH, OSB, дограма…)"
            className="bg-surface-tertiary border border-border-light rounded-full px-3 py-1.5 text-sm w-full md:w-72 outline-none"
          />
          {rows.length === 0 ? (
            <p className="text-xs text-content-tertiary italic">
              {search ? 'No matches.' : 'No rows.'}
            </p>
          ) : (
            <div className="rounded-lg border border-border-light/50 overflow-x-auto">
              <table className="w-full text-sm">
                <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider bg-surface-secondary/40">
                  <tr>
                    <th className="px-3 py-2">Description</th>
                    <th className="px-3 py-2 w-14">Unit</th>
                    <th className="px-3 py-2 w-20 text-right">Qty</th>
                    <th className="px-3 py-2 w-24 text-right">Material €</th>
                    <th className="px-3 py-2 w-24 text-right">Labour €</th>
                    <th className="px-3 py-2 w-24 text-right">Total €</th>
                    <th className="px-3 py-2 w-24 text-content-tertiary">Sheet</th>
                    <th className="px-3 py-2 w-8"></th>
                  </tr>
                </thead>
                <tbody>
                  {rows.map((r) => (
                    <tr
                      key={r.id}
                      className="border-t border-border-light/30 hover:bg-surface-secondary/30"
                    >
                      <td className="px-3 py-1.5 text-content-secondary max-w-md">
                        <EditableCell
                          value={r.description}
                          onCommit={(v) => handleEdit(r.id, 'description', v)}
                        />
                      </td>
                      <td className="px-3 py-1.5 text-xs">
                        <EditableCell
                          value={r.unit}
                          onCommit={(v) => handleEdit(r.id, 'unit', v)}
                          width={48}
                        />
                      </td>
                      <td className="px-3 py-1.5 text-right font-numeric text-xs">
                        <EditableNumber
                          value={r.quantity}
                          onCommit={(v) => handleEdit(r.id, 'quantity', v)}
                        />
                      </td>
                      <td className="px-3 py-1.5 text-right font-numeric text-xs">
                        <EditableNumber
                          value={r.material_price_eur}
                          onCommit={(v) => handleEdit(r.id, 'material_price_eur', v)}
                        />
                      </td>
                      <td className="px-3 py-1.5 text-right font-numeric text-xs">
                        <EditableNumber
                          value={r.labor_price_eur}
                          onCommit={(v) => handleEdit(r.id, 'labor_price_eur', v)}
                        />
                      </td>
                      <td className="px-3 py-1.5 text-right font-numeric text-xs font-medium text-content-primary">
                        {r.total_unit_price_eur?.toFixed(2) ?? '—'}
                      </td>
                      <td
                        className="px-3 py-1.5 text-[11px] text-content-tertiary truncate"
                        title={r.source_sheet ?? ''}
                      >
                        {r.source_sheet ?? '—'}
                      </td>
                      <td className="px-3 py-1.5 text-right">
                        <button
                          onClick={() => handleDeleteRow(r.id)}
                          className="text-[11px] text-red-400 hover:text-red-300"
                          title="Delete row"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

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
                      <span className="font-numeric">{m.overlapping_rows}</span>/
                      <span className="font-numeric">{m.total_rows}</span> rows match —
                      <span className="font-numeric"> {m.overlap_pct.toFixed(0)}%</span>{' '}
                      overlap · {new Date(m.imported_at).toLocaleDateString('en-GB')}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-end gap-2 mt-4">
              <button onClick={() => handleConflictResolve('skip')} className="oe-btn-ghost">
                Skip — keep existing
              </button>
              <button
                onClick={() => handleConflictResolve('replace')}
                className="oe-btn-secondary"
              >
                Replace
              </button>
              <button onClick={() => handleConflictResolve('add')} className="oe-btn-primary">
                Add anyway
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

// Inline-editable text cell. Click to edit, Enter / blur to commit, Esc to cancel.
function EditableCell({
  value,
  onCommit,
  width,
}: {
  value: string;
  onCommit: (v: string) => void;
  width?: number;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  if (!editing) {
    return (
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="text-left w-full hover:bg-surface-secondary/40 rounded px-1 -mx-1 py-0.5 cursor-text"
        title="Click to edit"
      >
        {value || <span className="text-content-tertiary italic">empty</span>}
      </button>
    );
  }

  return (
    <input
      autoFocus
      type="text"
      value={draft}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => {
        setEditing(false);
        if (draft !== value) onCommit(draft);
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          e.currentTarget.blur();
        } else if (e.key === 'Escape') {
          setDraft(value);
          setEditing(false);
        }
      }}
      style={width ? { width } : undefined}
      className="bg-surface-tertiary border border-border-light rounded px-1.5 py-0.5 text-sm w-full outline-none focus:border-content-tertiary"
    />
  );
}

// Inline-editable numeric cell. Empty string commits as 0.
function EditableNumber({
  value,
  onCommit,
}: {
  value: number | null;
  onCommit: (v: number) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value === null ? '' : String(value));

  useEffect(() => {
    setDraft(value === null ? '' : String(value));
  }, [value]);

  if (!editing) {
    return (
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="text-right w-full hover:bg-surface-secondary/40 rounded px-1 -mx-1 py-0.5 cursor-text font-numeric"
        title="Click to edit"
      >
        {value === null ? <span className="text-content-tertiary">—</span> : value.toFixed(2)}
      </button>
    );
  }

  return (
    <input
      autoFocus
      type="number"
      step="0.01"
      value={draft}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => {
        setEditing(false);
        const parsed = parseFloat(draft);
        const next = Number.isFinite(parsed) ? parsed : 0;
        if (next !== value) onCommit(next);
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter') {
          e.currentTarget.blur();
        } else if (e.key === 'Escape') {
          setDraft(value === null ? '' : String(value));
          setEditing(false);
        }
      }}
      className="bg-surface-tertiary border border-border-light rounded px-1.5 py-0.5 text-sm w-20 text-right outline-none focus:border-content-tertiary font-numeric"
    />
  );
}
