'use client';

/**
 * Self-hosted RAG price library — the single place to view and edit your prices.
 *
 * Layout (top → bottom):
 *   1. Header summary + Upload + Add row CTAs
 *   2. Imports list (file ↔ drawing pinning, delete an offer)
 *   3. Filter row (search, scope-by-import, page size)
 *   4. Always-visible price table (inline edit, delete, pagination)
 *   5. Conflict modal (duplicate XLSX detection)
 *   6. Add-row modal
 */

import { useEffect, useMemo, useState, useCallback } from 'react';
import { Plus, Upload, RefreshCw, X, Pencil, Check, Sparkles } from 'lucide-react';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';
import { Select } from '@/components/ui/Select';

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
  import_id: string | null;
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

const PAGE_SIZE = 50;

export function PriceLibrarySection() {
  const [imports, setImports] = useState<CorpusImport[]>([]);
  const [totalCorpusRows, setTotalCorpusRows] = useState(0);
  const [rows, setRows] = useState<CorpusRow[]>([]);
  const [totalRowsForFilter, setTotalRowsForFilter] = useState(0);
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [search, setSearch] = useState('');
  const [scopeImportId, setScopeImportId] = useState<string>(''); // '' = all
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [uploadMsg, setUploadMsg] = useState<{ tone: 'ok' | 'err'; text: string } | null>(null);
  const [linkDrawingId, setLinkDrawingId] = useState<string>('');
  const [conflict, setConflict] = useState<ConflictInfo | null>(null);
  const [addOpen, setAddOpen] = useState(false);

  /**
   * Explicit edit mode: cells are read-only by default. The user has to
   * click "Edit prices" to unlock them; on "Done" we surface a confirm
   * dialog asking whether to regenerate KCC for any linked drawings.
   */
  const [editMode, setEditMode] = useState(false);
  /** Tracks how many edits the user committed during this edit session.
   *  When > 0 and the user clicks Done, we offer to regenerate. */
  const [editsThisSession, setEditsThisSession] = useState(0);
  /** When non-null, render the regenerate-confirm dialog. */
  const [regenPrompt, setRegenPrompt] = useState<{
    drawings: Array<{ id: string; filename: string }>;
  } | null>(null);
  const [regenerating, setRegenerating] = useState(false);

  const fetchImports = useCallback(async () => {
    try {
      const data = await api.listCorpusImports();
      setImports(data.imports);
      setTotalCorpusRows(data.total_corpus_rows);
    } catch {
      /* swallow */
    }
  }, []);

  const fetchRows = useCallback(
    async (q: string, importId: string, pageIdx: number) => {
      setLoading(true);
      setError(null);
      try {
        const data = await api.listCorpus({
          q: q || undefined,
          import_id: importId || undefined,
          limit: PAGE_SIZE,
          offset: pageIdx * PAGE_SIZE,
        });
        setRows(data.rows);
        setTotalRowsForFilter(data.total);
      } catch (e) {
        setRows([]);
        setTotalRowsForFilter(0);
        setError(e instanceof Error ? e.message : 'Failed to load prices');
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  useEffect(() => {
    fetchImports();
    api.listDrawings().then(setDrawings).catch(() => setDrawings([]));
  }, [fetchImports]);

  // Debounced refetch on filter change.
  useEffect(() => {
    const t = setTimeout(() => fetchRows(search, scopeImportId, page), 200);
    return () => clearTimeout(t);
  }, [search, scopeImportId, page, fetchRows]);

  // Reset to page 0 when filters change.
  useEffect(() => {
    setPage(0);
  }, [search, scopeImportId]);

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
        setUploadMsg({
          tone: 'ok',
          text: `Already imported — ${result.row_count} rows reused${
            result.drawing_id ? ' (link updated)' : ''
          }.`,
        });
      } else {
        const overlapNote =
          result.overlap_warnings && result.overlap_warnings.length > 0
            ? ` ${result.overlap_warnings.length} overlap${
                result.overlap_warnings.length === 1 ? '' : 's'
              } detected — kept anyway.`
            : '';
        setUploadMsg({
          tone: 'ok',
          text: `Imported ${result.row_count} rows from ${result.sheet_count} sheet${
            result.sheet_count === 1 ? '' : 's'
          }${result.skipped_count > 0 ? `, ${result.skipped_count} skipped` : ''}.${overlapNote}`,
        });
      }
      await fetchImports();
      await fetchRows(search, scopeImportId, page);
    } catch (err) {
      setUploadMsg({
        tone: 'err',
        text: `Failed: ${err instanceof Error ? err.message : 'Upload failed'}`,
      });
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
      setUploadMsg({ tone: 'ok', text: 'Upload cancelled — using the existing import.' });
      return;
    }
    void performUpload(conflict.pendingFile, conflict.pendingDrawingId, action);
  };

  const handleDeleteImport = async (importId: string, filename: string) => {
    if (!confirm(`Delete import "${filename}" and all its corpus rows?`)) return;
    await api.deleteCorpusImport(importId).catch(() => {});
    await fetchImports();
    await fetchRows(search, scopeImportId, page);
  };

  const handleRelink = async (importId: string, drawingId: string) => {
    await api.setImportLink(importId, drawingId || null).catch(() => {});
    await fetchImports();
  };

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
                  ? (field === 'material_price_eur' ? Number(value) : r.material_price_eur ?? 0) +
                    (field === 'labor_price_eur' ? Number(value) : r.labor_price_eur ?? 0)
                  : r.total_unit_price_eur,
            }
          : r,
      ),
    );
    try {
      await api.updateCorpusRow(rowId, { [field]: value });
      setEditsThisSession((n) => n + 1);
    } catch {
      // Refetch to reset state if the server rejected.
      await fetchRows(search, scopeImportId, page);
    }
  };

  /**
   * "Done" closes edit mode. If the user committed at least one edit and
   * any imports are pinned to drawings, prompt to regenerate the KCC for
   * those drawings — otherwise the report is now out of sync with the
   * library prices.
   */
  const handleDoneEditing = () => {
    if (editsThisSession === 0) {
      setEditMode(false);
      return;
    }
    const linked = imports
      .filter((imp) => imp.drawing_id !== null)
      .map((imp) => ({ id: imp.drawing_id!, filename: imp.drawing_filename ?? '—' }));
    // Dedupe drawings (multiple offers can share one drawing in theory).
    const seen = new Set<string>();
    const unique = linked.filter((d) => (seen.has(d.id) ? false : (seen.add(d.id), true)));
    if (unique.length === 0) {
      // Edits made but nothing pinned → just close, no prompt.
      setEditMode(false);
      setEditsThisSession(0);
      return;
    }
    setRegenPrompt({ drawings: unique });
  };

  const handleRegenerate = async () => {
    if (!regenPrompt) return;
    setRegenerating(true);
    try {
      // Trigger price-research → KCC build for each linked drawing in
      // RAG mode (the offer we just edited is the source of truth).
      await Promise.all(
        regenPrompt.drawings.map((d) =>
          api.triggerAiKssGeneration(d.id, 'rag').catch(() => null),
        ),
      );
      setUploadMsg({
        tone: 'ok',
        text: `KCC regeneration started for ${regenPrompt.drawings.length} drawing${
          regenPrompt.drawings.length === 1 ? '' : 's'
        }.`,
      });
    } catch {
      setUploadMsg({ tone: 'err', text: 'Failed to trigger regeneration.' });
    }
    setRegenerating(false);
    setRegenPrompt(null);
    setEditMode(false);
    setEditsThisSession(0);
  };

  const handleDeleteRow = async (rowId: string) => {
    if (!confirm('Delete this row?')) return;
    try {
      await api.deleteCorpusRow(rowId);
      setRows((prev) => prev.filter((r) => r.id !== rowId));
      setTotalRowsForFilter((t) => Math.max(0, t - 1));
      await fetchImports();
    } catch {
      /* swallow */
    }
  };

  const handleAddRow = async (input: {
    description: string;
    unit: string;
    quantity: string;
    material_price_eur: string;
    labor_price_eur: string;
    sek_code: string;
    import_id: string;
  }) => {
    try {
      await api.createCorpusRow({
        description: input.description.trim(),
        unit: input.unit.trim(),
        quantity: input.quantity ? parseFloat(input.quantity) : undefined,
        material_price_eur: input.material_price_eur
          ? parseFloat(input.material_price_eur)
          : undefined,
        labor_price_eur: input.labor_price_eur
          ? parseFloat(input.labor_price_eur)
          : undefined,
        sek_code: input.sek_code.trim() || undefined,
        import_id: input.import_id || undefined,
      });
      setAddOpen(false);
      await fetchImports();
      await fetchRows(search, scopeImportId, 0);
      setPage(0);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to add row');
    }
  };

  const importMap = useMemo(() => {
    const m = new Map<string, CorpusImport>();
    for (const imp of imports) m.set(imp.id, imp);
    return m;
  }, [imports]);

  const totalPages = Math.max(1, Math.ceil(totalRowsForFilter / PAGE_SIZE));
  const fromIdx = totalRowsForFilter === 0 ? 0 : page * PAGE_SIZE + 1;
  const toIdx = Math.min(totalRowsForFilter, (page + 1) * PAGE_SIZE);

  return (
    <section className="oe-card p-5 space-y-5">
      {/* Header summary */}
      <div className="flex items-start justify-between gap-4 flex-wrap">
        <div>
          <h2 className="text-base font-medium text-content-primary">
            My price library{' '}
            <span className="oe-badge ml-2" data-variant="accent">your data</span>
          </h2>
          <p className="mt-1 text-[12.5px] text-content-tertiary">
            <span className="font-numeric text-content-secondary">{totalCorpusRows}</span> priced
            rows from <span className="font-numeric">{imports.length}</span>{' '}
            {imports.length === 1 ? 'offer' : 'offers'} you uploaded. Click any cell to edit,
            link offers to drawings for 1:1 RAG.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {editMode ? (
            <button
              onClick={handleDoneEditing}
              className="oe-btn-primary oe-btn-sm"
              title="Exit edit mode"
            >
              <Check size={13} /> Done editing
            </button>
          ) : (
            <button
              onClick={() => {
                setEditMode(true);
                setEditsThisSession(0);
              }}
              className="oe-btn-secondary oe-btn-sm"
              title="Unlock cells for inline editing"
            >
              <Pencil size={13} /> Edit prices
            </button>
          )}
          <button
            onClick={() => setAddOpen(true)}
            className="oe-btn-secondary oe-btn-sm"
          >
            <Plus size={13} /> Add row
          </button>
          <label className="oe-btn-primary oe-btn-sm cursor-pointer">
            <Upload size={13} /> {uploading ? 'Uploading…' : 'Upload XLSX'}
            <input
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
      </div>

      {/* Upload-target drawing picker */}
      <div className="flex items-center gap-3 flex-wrap text-[11.5px] text-content-tertiary">
        <span>Next upload links to:</span>
        <Select
          size="sm"
          ariaLabel="Link uploaded offer to drawing"
          value={linkDrawingId}
          onChange={setLinkDrawingId}
          options={[
            { value: '', label: '— No link (whole-corpus RAG)' },
            ...drawings.map((d) => ({ value: d.id, label: d.filename })),
          ]}
        />
      </div>

      {uploadMsg && (
        <div
          className="text-xs px-3 py-2 rounded-lg"
          style={{
            background:
              uploadMsg.tone === 'err' ? 'var(--oe-error-bg)' : 'var(--oe-success-bg)',
            color: uploadMsg.tone === 'err' ? 'var(--oe-error)' : 'var(--oe-success)',
          }}
        >
          {uploadMsg.text}
        </div>
      )}

      {/* Imports list */}
      {imports.length > 0 && (
        <div className="rounded-lg border border-border-light/60 overflow-hidden">
          <table className="oe-table">
            <thead>
              <tr>
                <th>File</th>
                <th className="w-44">Linked drawing</th>
                <th className="w-16 !text-right">Sheets</th>
                <th className="w-16 !text-right">Rows</th>
                <th className="w-24">Imported</th>
                <th className="w-12"></th>
              </tr>
            </thead>
            <tbody>
              {imports.map((imp) => (
                <tr key={imp.id}>
                  <td className="truncate max-w-xs" title={imp.filename}>
                    {imp.filename}
                  </td>
                  <td>
                    <Select
                      size="sm"
                      ariaLabel="Linked drawing"
                      value={imp.drawing_id ?? ''}
                      onChange={(v) => handleRelink(imp.id, v)}
                      options={[
                        { value: '', label: '— None' },
                        ...drawings.map((d) => ({ value: d.id, label: d.filename })),
                      ]}
                    />
                  </td>
                  <td className="oe-num">{imp.sheet_count}</td>
                  <td className="oe-num">{imp.row_count}</td>
                  <td className="text-xs text-content-tertiary">
                    {new Date(imp.imported_at).toLocaleDateString('en-GB')}
                  </td>
                  <td className="!text-right">
                    <button
                      onClick={() => handleDeleteImport(imp.id, imp.filename)}
                      className="text-xs"
                      style={{ color: 'var(--oe-error)' }}
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

      {/* Filter row */}
      <div className="flex items-center gap-3 flex-wrap">
        <input
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search (description, e.g. KVH, OSB, дограма…)"
          className="oe-input flex-1 min-w-[220px] !rounded-full"
        />
        <Select
          size="sm"
          ariaLabel="Scope to offer"
          value={scopeImportId}
          onChange={setScopeImportId}
          options={[
            { value: '', label: 'All offers' },
            ...imports.map((imp) => ({ value: imp.id, label: imp.filename })),
          ]}
        />
        <button
          onClick={() => fetchRows(search, scopeImportId, page)}
          className="oe-btn-ghost oe-btn-sm"
          title="Reload"
          aria-label="Reload"
        >
          <RefreshCw size={13} />
        </button>
      </div>

      {/* Browser table */}
      <div className="rounded-lg border border-border-light/60 overflow-x-auto">
        <table className="oe-table">
          <thead>
            <tr>
              <th>Description</th>
              <th className="w-16">Unit</th>
              <th className="w-20 !text-right" title="Quantity from the XLSX (column D)">Qty</th>
              <th className="w-24 !text-right" title="Unit material price (XLSX column E)">Material € / unit</th>
              <th className="w-24 !text-right" title="Unit labour price (XLSX column G)">Labour € / unit</th>
              <th className="w-24 !text-right" title="Material + labour, per unit">Unit total €</th>
              <th
                className="w-24 !text-right"
                title="Quantity × unit total — matches the XLSX «Общо» column"
              >
                Row total €
              </th>
              <th className="w-24">Sheet</th>
              <th className="w-8"></th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              Array.from({ length: 6 }).map((_, i) => (
                <tr key={`sk-${i}`} className="kcc-skeleton-row">
                  <td><div className="oe-skeleton h-3 w-3/4" /></td>
                  <td><div className="oe-skeleton h-3 w-8" /></td>
                  <td className="!text-right"><div className="oe-skeleton h-3 w-12 ml-auto" /></td>
                  <td className="!text-right"><div className="oe-skeleton h-3 w-14 ml-auto" /></td>
                  <td className="!text-right"><div className="oe-skeleton h-3 w-14 ml-auto" /></td>
                  <td className="!text-right"><div className="oe-skeleton h-3 w-16 ml-auto" /></td>
                  <td className="!text-right"><div className="oe-skeleton h-3 w-16 ml-auto" /></td>
                  <td><div className="oe-skeleton h-3 w-20" /></td>
                  <td></td>
                </tr>
              ))
            ) : error ? (
              <tr>
                <td
                  colSpan={9}
                  className="text-center !py-6 text-xs"
                  style={{ color: 'var(--oe-error)' }}
                >
                  {error}
                </td>
              </tr>
            ) : rows.length === 0 ? (
              <tr>
                <td colSpan={9} className="text-center !py-10 text-content-tertiary">
                  <div className="oe-display text-[18px] text-content-secondary">
                    {search || scopeImportId ? 'No matches.' : 'No rows yet.'}
                  </div>
                  <p className="text-xs mt-1">
                    {search || scopeImportId
                      ? 'Try broadening the search or switching the offer filter.'
                      : 'Upload an XLSX offer or add a manual row to get started.'}
                  </p>
                </td>
              </tr>
            ) : (
              (() => {
                /* Group rows by sheet so the user sees clear ТАСОС /
                   Дани67 / Топос35 sections matching their workbook tabs. */
                const elements: React.ReactNode[] = [];
                let lastKey: string | null = null;
                for (const r of rows) {
                  const importedFile = r.import_id
                    ? importMap.get(r.import_id)?.filename ?? '—'
                    : 'manual';
                  const sheet = r.source_sheet ?? '';
                  const groupKey = `${r.import_id ?? 'manual'}::${sheet}`;
                  if (groupKey !== lastKey) {
                    lastKey = groupKey;
                    elements.push(
                      <tr key={`grp-${groupKey}`} className="kcc-group-header">
                        <td colSpan={9}>
                          <span className="oe-eyebrow">
                            {sheet ? `Sheet ${sheet}` : 'Manual rows'}
                          </span>
                          <span className="ml-3 text-[11px] text-content-tertiary">
                            {importedFile}
                          </span>
                        </td>
                      </tr>,
                    );
                  }
                  const rowTotal =
                    r.quantity != null && r.total_unit_price_eur != null
                      ? r.quantity * r.total_unit_price_eur
                      : null;
                  elements.push(
                    <tr key={r.id}>
                      <td className="text-content-secondary max-w-md">
                        <EditableCell
                          editable={editMode}
                          value={r.description}
                          onCommit={(v) => handleEdit(r.id, 'description', v)}
                        />
                      </td>
                      <td className="text-xs">
                        <EditableCell
                          editable={editMode}
                          value={r.unit}
                          onCommit={(v) => handleEdit(r.id, 'unit', v)}
                        />
                      </td>
                      <td className="oe-num text-xs">
                        <EditableNumber
                          editable={editMode}
                          value={r.quantity}
                          decimals={3}
                          onCommit={(v) => handleEdit(r.id, 'quantity', v)}
                        />
                      </td>
                      <td className="oe-num text-xs">
                        <EditableNumber
                          editable={editMode}
                          value={r.material_price_eur}
                          onCommit={(v) => handleEdit(r.id, 'material_price_eur', v)}
                        />
                      </td>
                      <td className="oe-num text-xs">
                        <EditableNumber
                          editable={editMode}
                          value={r.labor_price_eur}
                          onCommit={(v) => handleEdit(r.id, 'labor_price_eur', v)}
                        />
                      </td>
                      <td className="oe-num text-xs font-medium text-content-primary">
                        {r.total_unit_price_eur?.toFixed(2) ?? '—'}
                      </td>
                      <td className="oe-num text-xs font-medium text-content-primary">
                        {rowTotal != null ? rowTotal.toFixed(2) : '—'}
                      </td>
                      <td
                        className="text-[11px] text-content-tertiary"
                        title={`Row ${r.source_row ?? '–'} · ${importedFile}`}
                      >
                        {sheet ? (
                          <span className="oe-badge" data-variant="info">
                            {sheet}
                          </span>
                        ) : (
                          <span className="oe-badge">manual</span>
                        )}
                      </td>
                      <td className="!text-right">
                        {editMode && (
                          <button
                            onClick={() => handleDeleteRow(r.id)}
                            className="text-[11px]"
                            style={{ color: 'var(--oe-error)' }}
                            title="Delete row"
                          >
                            ✕
                          </button>
                        )}
                      </td>
                    </tr>,
                  );
                }
                return elements;
              })()
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalRowsForFilter > 0 && (
        <div className="flex items-center justify-between text-[11.5px] text-content-tertiary">
          <span>
            <span className="font-numeric">{fromIdx}</span>–
            <span className="font-numeric">{toIdx}</span> of{' '}
            <span className="font-numeric">{totalRowsForFilter}</span>
          </span>
          <div className="flex items-center gap-1">
            <button
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={page === 0}
              className="oe-btn-ghost oe-btn-sm"
            >
              ←
            </button>
            <span className="font-numeric px-2">
              {page + 1} / {totalPages}
            </span>
            <button
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={page >= totalPages - 1}
              className="oe-btn-ghost oe-btn-sm"
            >
              →
            </button>
          </div>
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
                      <span className="font-numeric"> {m.overlap_pct.toFixed(0)}%</span> overlap ·{' '}
                      {new Date(m.imported_at).toLocaleDateString('en-GB')}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-end gap-2 mt-4">
              <button onClick={() => handleConflictResolve('skip')} className="oe-btn-ghost oe-btn-sm">
                Skip — keep existing
              </button>
              <button
                onClick={() => handleConflictResolve('replace')}
                className="oe-btn-secondary oe-btn-sm"
              >
                Replace
              </button>
              <button onClick={() => handleConflictResolve('add')} className="oe-btn-primary oe-btn-sm">
                Add anyway
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Regenerate-after-edit prompt */}
      {regenPrompt && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4">
          <div className="oe-card max-w-md w-full p-5 space-y-4">
            <div>
              <h3 className="text-base font-medium text-content-primary">
                Regenerate KCC?
              </h3>
              <p className="text-[12.5px] text-content-tertiary mt-1">
                You changed{' '}
                <span className="font-numeric">{editsThisSession}</span>{' '}
                {editsThisSession === 1 ? 'cell' : 'cells'}. The previously
                generated report{regenPrompt.drawings.length > 1 ? 's' : ''}{' '}
                still reflects the old prices. Regenerate now to bring{' '}
                {regenPrompt.drawings.length === 1 ? 'it' : 'them'} back in sync?
              </p>
            </div>
            <ul className="text-xs space-y-1.5 max-h-40 overflow-y-auto rounded-lg border border-border-light/60 p-2">
              {regenPrompt.drawings.map((d) => (
                <li key={d.id} className="flex items-center gap-2 truncate">
                  <span
                    className="w-1.5 h-1.5 rounded-full flex-none"
                    style={{ background: 'var(--oe-accent)' }}
                  />
                  <span className="truncate text-content-secondary">{d.filename}</span>
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-end gap-2">
              <button
                onClick={() => {
                  setRegenPrompt(null);
                  setEditMode(false);
                  setEditsThisSession(0);
                }}
                disabled={regenerating}
                className="oe-btn-ghost oe-btn-sm"
              >
                Skip — keep old report
              </button>
              <button
                onClick={handleRegenerate}
                disabled={regenerating}
                className="oe-btn-primary oe-btn-sm"
              >
                <Sparkles size={13} />
                {regenerating ? 'Starting…' : 'Regenerate KCC'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Add row modal */}
      {addOpen && (
        <AddRowModal
          imports={imports}
          defaultImportId={scopeImportId}
          onCancel={() => setAddOpen(false)}
          onSubmit={handleAddRow}
        />
      )}
    </section>
  );
}

// ──────────────────────────────────────────────────────────────────────
// Inline-editable text cell. Click to edit, Enter / blur to commit, Esc to
// cancel. When `editable` is false the cell renders read-only — no click
// affordance, no hover background, no cursor-text.
function EditableCell({
  value,
  onCommit,
  editable,
}: {
  value: string;
  onCommit: (v: string) => void;
  editable: boolean;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  if (!editable) {
    return (
      <span className="block w-full">
        {value || <span className="text-content-tertiary italic">empty</span>}
      </span>
    );
  }

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
      className="bg-surface-tertiary border border-border-light rounded px-1.5 py-0.5 text-sm w-full outline-none focus:border-content-tertiary"
    />
  );
}

// Inline-editable numeric cell. Empty string commits as 0.
// `decimals` (default 2) controls how many decimal places we render. We
// also trim trailing zeros, so 90.000 → "90" but 1.674 → "1.674".
function EditableNumber({
  value,
  onCommit,
  editable,
  decimals = 2,
}: {
  value: number | null;
  onCommit: (v: number) => void;
  editable: boolean;
  decimals?: number;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value === null ? '' : String(value));

  useEffect(() => {
    setDraft(value === null ? '' : String(value));
  }, [value]);

  const formatted = (() => {
    if (value === null) return null;
    const fixed = value.toFixed(decimals);
    // Trim trailing zeros after decimal point but keep at least 2 digits
    // when there's any fractional part (so 1640 stays "1640" but 2.184
    // stays "2.184" and 1.5 stays "1.50").
    if (!fixed.includes('.')) return fixed;
    const trimmed = fixed.replace(/0+$/, '').replace(/\.$/, '');
    return trimmed.includes('.')
      ? trimmed
      : trimmed; // integer — keep no trailing decimals
  })();

  if (!editable) {
    return (
      <span className="block w-full text-right font-numeric">
        {formatted === null ? <span className="text-content-tertiary">—</span> : formatted}
      </span>
    );
  }

  if (!editing) {
    return (
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="text-right w-full hover:bg-surface-secondary/40 rounded px-1 -mx-1 py-0.5 cursor-text font-numeric"
        title="Click to edit"
      >
        {formatted === null ? <span className="text-content-tertiary">—</span> : formatted}
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

// ──────────────────────────────────────────────────────────────────────
function AddRowModal({
  imports,
  defaultImportId,
  onCancel,
  onSubmit,
}: {
  imports: CorpusImport[];
  defaultImportId: string;
  onCancel: () => void;
  onSubmit: (input: {
    description: string;
    unit: string;
    quantity: string;
    material_price_eur: string;
    labor_price_eur: string;
    sek_code: string;
    import_id: string;
  }) => void;
}) {
  const [form, setForm] = useState({
    description: '',
    unit: 'М2',
    quantity: '',
    material_price_eur: '',
    labor_price_eur: '',
    sek_code: '',
    import_id: defaultImportId,
  });

  const valid = form.description.trim().length > 0 && form.unit.trim().length > 0;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4">
      <div className="oe-card max-w-lg w-full p-5 space-y-4">
        <div className="flex items-start justify-between">
          <div>
            <h3 className="text-base font-medium text-content-primary">Add a manual row</h3>
            <p className="text-[11.5px] text-content-tertiary mt-1">
              Adds straight to your corpus — RAG can pull from it like any imported row.
            </p>
          </div>
          <button onClick={onCancel} className="oe-btn-ghost oe-btn-sm oe-btn-icon" aria-label="Close">
            <X size={14} />
          </button>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Field label="Description *">
            <input
              type="text"
              value={form.description}
              onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              placeholder="e.g. KVH 8x10 — стенна конструкция"
              className="oe-input"
            />
          </Field>
          <Field label="Unit *">
            <Select
              ariaLabel="Unit"
              value={form.unit}
              onChange={(v) => setForm((f) => ({ ...f, unit: v }))}
              options={['М2', 'М3', 'М', 'БР', 'КГ', 'Т', 'Л', 'КОМПЛ.'].map((u) => ({
                value: u,
                label: u,
              }))}
            />
          </Field>
          <Field label="Qty">
            <input
              type="number"
              step="0.01"
              value={form.quantity}
              onChange={(e) => setForm((f) => ({ ...f, quantity: e.target.value }))}
              className="oe-input font-numeric"
            />
          </Field>
          <Field label="СЕК код">
            <input
              type="text"
              value={form.sek_code}
              onChange={(e) => setForm((f) => ({ ...f, sek_code: e.target.value }))}
              placeholder="СЕК05.002"
              className="oe-input"
            />
          </Field>
          <Field label="Material €">
            <input
              type="number"
              step="0.01"
              value={form.material_price_eur}
              onChange={(e) =>
                setForm((f) => ({ ...f, material_price_eur: e.target.value }))
              }
              className="oe-input font-numeric"
            />
          </Field>
          <Field label="Labour €">
            <input
              type="number"
              step="0.01"
              value={form.labor_price_eur}
              onChange={(e) => setForm((f) => ({ ...f, labor_price_eur: e.target.value }))}
              className="oe-input font-numeric"
            />
          </Field>
          <div className="col-span-2">
            <Field label="Pin to offer (optional)">
              <Select
                ariaLabel="Pin to offer"
                value={form.import_id}
                onChange={(v) => setForm((f) => ({ ...f, import_id: v }))}
                options={[
                  { value: '', label: '— Manual (not pinned)' },
                  ...imports.map((imp) => ({ value: imp.id, label: imp.filename })),
                ]}
              />
            </Field>
          </div>
        </div>

        <div className="flex items-center justify-end gap-2 pt-2">
          <button onClick={onCancel} className="oe-btn-ghost oe-btn-sm">
            Cancel
          </button>
          <button
            disabled={!valid}
            onClick={() => onSubmit(form)}
            className="oe-btn-primary oe-btn-sm"
          >
            Add row
          </button>
        </div>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="block text-[11px] text-content-tertiary mb-1.5">{label}</span>
      {children}
    </label>
  );
}
