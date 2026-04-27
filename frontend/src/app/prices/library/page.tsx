'use client';

import { useEffect, useState, useCallback } from 'react';
import { api } from '@/lib/api';

interface CorpusImport {
  id: string;
  filename: string;
  sheet_count: number;
  row_count: number;
  skipped_count: number;
  imported_at: string;
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

export default function PriceLibraryPage() {
  const [imports, setImports] = useState<CorpusImport[]>([]);
  const [totalRows, setTotalRows] = useState(0);
  const [rows, setRows] = useState<CorpusRow[]>([]);
  const [search, setSearch] = useState('');
  const [uploading, setUploading] = useState(false);
  const [uploadMsg, setUploadMsg] = useState<string | null>(null);
  const [loadingRows, setLoadingRows] = useState(false);

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
  }, [fetchImports, fetchRows]);

  const handleUpload = async (file: File) => {
    setUploading(true);
    setUploadMsg(null);
    try {
      const result = await api.importPriceCorpus(file);
      if (result.deduped) {
        setUploadMsg(`This file was already imported (${result.row_count} rows reused).`);
      } else {
        setUploadMsg(
          `Imported ${result.row_count} priced rows from ${result.sheet_count} sheet${
            result.sheet_count === 1 ? '' : 's'
          }${result.skipped_count > 0 ? ` (${result.skipped_count} non-priced rows skipped)` : ''}.`,
        );
      }
      await fetchImports();
      await fetchRows(search);
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Upload failed';
      setUploadMsg(`Failed: ${msg}`);
    }
    setUploading(false);
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

  const onSearchChange = (q: string) => {
    setSearch(q);
    fetchRows(q);
  };

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
        <div>
          <h1 className="text-2xl font-bold">Моята ценова библиотека</h1>
          <p className="text-sm text-content-tertiary mt-1">
            Качете предишни оферти (XLSX) — системата извлича цените и при
            генериране на нов КСС може да ги използва (RAG режим) вместо
            всеки път да търси наново онлайн. Текущо в библиотеката:{' '}
            <span className="font-mono text-content-secondary">{totalRows}</span> позиции.
          </p>
        </div>

        {/* Drop / pick */}
        <div className="oe-card p-4">
          <label
            htmlFor="corpus-upload"
            className="flex items-center justify-between gap-4 cursor-pointer"
          >
            <div>
              <div className="text-sm font-medium">Качване на оферта (XLSX)</div>
              <div className="text-xs text-content-tertiary mt-1">
                Поддържан формат: КСС-оферта с колони „Описание / м.ед. / Колич. /
                Ед. Цена мат / Цена мат / Монтаж / Цена монтаж / Общо“. Множество
                листове се внасят като отделни модули.
              </div>
            </div>
            <input
              id="corpus-upload"
              type="file"
              accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
              className="hidden"
              disabled={uploading}
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) handleUpload(f);
                e.target.value = '';
              }}
            />
            <span className="oe-btn-primary cursor-pointer whitespace-nowrap">
              {uploading ? 'Качване…' : 'Избери файл'}
            </span>
          </label>
          {uploadMsg && (
            <div
              className={`mt-3 text-xs px-3 py-2 rounded ${
                uploadMsg.startsWith('Failed')
                  ? 'bg-red-900/30 text-red-300'
                  : 'bg-emerald-900/30 text-emerald-300'
              }`}
            >
              {uploadMsg}
            </div>
          )}
        </div>

        {/* Imports list */}
        <div className="oe-card p-4">
          <h2 className="text-sm font-semibold mb-3">История на качванията</h2>
          {imports.length === 0 ? (
            <p className="text-xs text-content-tertiary italic">
              Все още няма качени файлове.
            </p>
          ) : (
            <table className="w-full text-sm">
              <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider">
                <tr className="border-b border-border-light">
                  <th className="px-2 py-2">Файл</th>
                  <th className="px-2 py-2 w-20 text-right">Листове</th>
                  <th className="px-2 py-2 w-20 text-right">Позиции</th>
                  <th className="px-2 py-2 w-32">Качено</th>
                  <th className="px-2 py-2 w-16"></th>
                </tr>
              </thead>
              <tbody>
                {imports.map((imp) => (
                  <tr key={imp.id} className="border-b border-border-light/30">
                    <td className="px-2 py-2 truncate max-w-xs">{imp.filename}</td>
                    <td className="px-2 py-2 text-right font-mono text-xs">
                      {imp.sheet_count}
                    </td>
                    <td className="px-2 py-2 text-right font-mono text-xs">
                      {imp.row_count}
                    </td>
                    <td className="px-2 py-2 text-xs text-content-tertiary">
                      {new Date(imp.imported_at).toLocaleDateString('bg-BG')}
                    </td>
                    <td className="px-2 py-2 text-right">
                      <button
                        onClick={() => handleDelete(imp.id, imp.filename)}
                        className="text-xs text-red-400 hover:text-red-300"
                        title="Премахване на този файл и всичките му позиции"
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
        <div className="oe-card p-4">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-semibold">Съдържание</h2>
            <input
              type="text"
              value={search}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Търсене (напр. KVH, OSB, дограма)…"
              className="bg-surface-tertiary border border-border-light rounded px-3 py-1.5 text-sm w-72"
            />
          </div>
          {loadingRows ? (
            <p className="text-xs text-content-tertiary italic">Зареждане…</p>
          ) : rows.length === 0 ? (
            <p className="text-xs text-content-tertiary italic">
              {search ? 'Няма съвпадения.' : 'Няма позиции.'}
            </p>
          ) : (
            <table className="w-full text-sm">
              <thead className="text-left text-content-tertiary text-[10px] uppercase tracking-wider">
                <tr className="border-b border-border-light">
                  <th className="px-2 py-2">Описание</th>
                  <th className="px-2 py-2 w-14">М-ка</th>
                  <th className="px-2 py-2 w-24 text-right">Материали</th>
                  <th className="px-2 py-2 w-24 text-right">Труд</th>
                  <th className="px-2 py-2 w-24 text-right">Общо</th>
                  <th className="px-2 py-2 w-32 text-content-tertiary">Лист</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={r.id} className="border-b border-border-light/30 hover:bg-surface-secondary/30">
                    <td className="px-2 py-1.5 text-content-secondary truncate max-w-md" title={r.description}>
                      {r.description}
                    </td>
                    <td className="px-2 py-1.5 text-xs">{r.unit}</td>
                    <td className="px-2 py-1.5 text-right font-mono text-xs">
                      {r.material_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td className="px-2 py-1.5 text-right font-mono text-xs">
                      {r.labor_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td className="px-2 py-1.5 text-right font-mono text-xs font-medium">
                      {r.total_unit_price_eur?.toFixed(2) ?? '—'}
                    </td>
                    <td className="px-2 py-1.5 text-[11px] text-content-tertiary truncate" title={r.source_sheet ?? ''}>
                      {r.source_sheet ?? '—'}
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
