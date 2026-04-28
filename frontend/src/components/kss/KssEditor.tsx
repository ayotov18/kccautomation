'use client';

import { useState, useEffect, useCallback } from 'react';
import { api } from '@/lib/api';
import type { KssCorrectionItem, KssCorrectionRecord } from '@/types';

interface KssItem {
  item_no: number;
  sek_code: string;
  description: string;
  unit: string;
  quantity: number;
  total_price: number;
  // Track edits
  edited?: boolean;
  original_sek_code?: string;
  original_description?: string;
  original_quantity?: number;
  original_unit?: string;
}

interface Props {
  drawingId: string;
}

export function KssEditor({ drawingId }: Props) {
  const [items, setItems] = useState<KssItem[]>([]);
  const [corrections, setCorrections] = useState<KssCorrectionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveResult, setSaveResult] = useState<string | null>(null);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);

  // Load KSS report JSON
  const loadKssData = useCallback(async () => {
    setLoading(true);
    try {
      const blob = await api.downloadReportJson(drawingId);
      const text = await blob.text();
      const report = JSON.parse(text);
      if (report.items) {
        setItems(report.items.map((item: KssItem) => ({
          ...item,
          edited: false,
          original_sek_code: item.sek_code,
          original_description: item.description,
          original_quantity: item.quantity,
          original_unit: item.unit,
        })));
      }
    } catch {
      // Report not available yet — might need to generate first
    }

    // Load existing corrections
    try {
      const corr = await api.listCorrections(drawingId);
      setCorrections(corr);
    } catch {
      // No corrections yet
    }
    setLoading(false);
  }, [drawingId]);

  useEffect(() => {
    loadKssData();
  }, [loadKssData]);

  const handleEdit = (idx: number, field: keyof KssItem, value: string | number) => {
    setItems(prev => prev.map((item, i) => {
      if (i !== idx) return item;
      return { ...item, [field]: value, edited: true };
    }));
  };

  const handleSaveCorrections = async () => {
    const editedItems = items.filter(i => i.edited);
    if (editedItems.length === 0) return;

    setSaving(true);
    setSaveResult(null);

    const correctionItems: KssCorrectionItem[] = editedItems.map(item => ({
      original_sek_code: item.original_sek_code,
      original_description: item.original_description,
      original_quantity: item.original_quantity,
      original_unit: item.original_unit,
      corrected_sek_code: item.sek_code !== item.original_sek_code ? item.sek_code : undefined,
      corrected_description: item.description !== item.original_description ? item.description : undefined,
      corrected_quantity: item.quantity !== item.original_quantity ? item.quantity : undefined,
      corrected_unit: item.unit !== item.original_unit ? item.unit : undefined,
      correction_type: item.sek_code !== item.original_sek_code ? 'sek_code'
        : item.quantity !== item.original_quantity ? 'quantity'
        : item.description !== item.original_description ? 'description'
        : 'unit',
    }));

    try {
      const result = await api.submitCorrections(drawingId, correctionItems);
      setSaveResult(`Saved ${result.corrections_saved} corrections, ${result.drm_artifacts_updated} DRM artifacts updated`);
      // Mark items as no longer edited
      setItems(prev => prev.map(item => ({
        ...item,
        edited: false,
        original_sek_code: item.sek_code,
        original_description: item.description,
        original_quantity: item.quantity,
        original_unit: item.unit,
      })));
      // Reload corrections list
      const corr = await api.listCorrections(drawingId);
      setCorrections(corr);
    } catch {
      setSaveResult('Failed to save corrections');
    }
    setSaving(false);
  };

  const editedCount = items.filter(i => i.edited).length;

  if (loading) {
    return <div className="text-gray-500 text-sm p-4">Loading KSS data...</div>;
  }

  if (items.length === 0) {
    return (
      <div className="text-gray-500 text-sm p-4">
        No KSS data available. Generate a KSS report first.
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-100">KSS Editor</h3>
          <p className="text-xs text-gray-500">
            Click any cell to edit. Corrections feed into the DRM learning system.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {editedCount > 0 && (
            <span className="text-xs text-[color:var(--oe-accent)]">
              {editedCount} unsaved {editedCount === 1 ? 'change' : 'changes'}
            </span>
          )}
          <button
            onClick={handleSaveCorrections}
            disabled={editedCount === 0 || saving}
            className="px-4 py-2 bg-[color:var(--oe-accent)] hover:bg-[color:var(--oe-accent-hot)] disabled:bg-gray-700 disabled:text-gray-500 rounded text-sm font-medium transition-colors"
          >
            {saving ? 'Saving...' : 'Save Corrections'}
          </button>
        </div>
      </div>

      {saveResult && (
        <div className={`text-xs px-3 py-2 rounded ${saveResult.includes('Failed') ? 'bg-red-900/30 text-red-400' : 'bg-[color:var(--oe-accent-soft-bg)] text-[color:var(--oe-accent)]'}`}>
          {saveResult}
        </div>
      )}

      {/* Editable table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-gray-400 border-b border-gray-800">
              <th className="px-2 py-2 w-10">No.</th>
              <th className="px-2 py-2 w-28">СЕК Код</th>
              <th className="px-2 py-2">Описание на СМР</th>
              <th className="px-2 py-2 w-16">Мярка</th>
              <th className="px-2 py-2 w-24">Количество</th>
              <th className="px-2 py-2 w-24">Стойност</th>
              <th className="px-2 py-2 w-8"></th>
            </tr>
          </thead>
          <tbody>
            {items.map((item, idx) => (
              <tr
                key={idx}
                className={`border-b border-gray-800/50 ${item.edited ? 'bg-[color:var(--oe-accent-soft-bg)]' : 'hover:bg-gray-800/30'}`}
              >
                <td className="px-2 py-1.5 text-gray-500">{item.item_no}</td>
                <td className="px-2 py-1.5">
                  {editingIdx === idx ? (
                    <input
                      type="text"
                      value={item.sek_code}
                      onChange={(e) => handleEdit(idx, 'sek_code', e.target.value)}
                      onBlur={() => setEditingIdx(null)}
                      autoFocus
                      className="w-full bg-gray-800 border border-[color:var(--oe-accent)] rounded px-1.5 py-0.5 text-xs font-mono text-[color:var(--oe-accent)] focus:outline-none"
                    />
                  ) : (
                    <span
                      onClick={() => setEditingIdx(idx)}
                      className={`font-mono text-xs cursor-pointer ${item.edited && item.sek_code !== item.original_sek_code ? 'text-[color:var(--oe-accent)]' : 'text-[color:var(--oe-accent)]'}`}
                      title="Click to edit"
                    >
                      {item.sek_code || '-'}
                    </span>
                  )}
                </td>
                <td className="px-2 py-1.5">
                  <EditableCell
                    value={item.description}
                    onChange={(v) => handleEdit(idx, 'description', v)}
                    edited={item.edited && item.description !== item.original_description}
                  />
                </td>
                <td className="px-2 py-1.5">
                  <EditableCell
                    value={item.unit}
                    onChange={(v) => handleEdit(idx, 'unit', v)}
                    edited={item.edited && item.unit !== item.original_unit}
                    className="text-gray-400"
                  />
                </td>
                <td className="px-2 py-1.5">
                  <EditableCell
                    value={String(item.quantity)}
                    onChange={(v) => handleEdit(idx, 'quantity', parseFloat(v) || 0)}
                    edited={item.edited && item.quantity !== item.original_quantity}
                    className="text-right"
                    type="number"
                  />
                </td>
                <td className="px-2 py-1.5 text-right text-gray-400">
                  {item.total_price > 0 ? `${item.total_price.toFixed(2)} €` : '-'}
                </td>
                <td className="px-2 py-1.5">
                  {item.edited && (
                    <span className="w-2 h-2 rounded-full bg-[color:var(--oe-accent)] inline-block" title="Edited" />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Correction history */}
      {corrections.length > 0 && (
        <div className="mt-6">
          <h4 className="text-sm font-medium text-gray-400 mb-2">
            Correction History ({corrections.length})
          </h4>
          <div className="space-y-1">
            {corrections.slice(0, 10).map((c) => (
              <div key={c.id} className="text-xs text-gray-500 flex items-center gap-2">
                <span className="text-[color:var(--oe-accent)]/60">{c.correction_type}</span>
                {c.original_sek_code && (
                  <span><span className="text-red-400/60 line-through">{c.original_sek_code}</span> {'->'} <span className="text-[color:var(--oe-accent)]/60">{c.corrected_sek_code}</span></span>
                )}
                {c.corrected_description && (
                  <span className="truncate max-w-xs">{c.corrected_description}</span>
                )}
                <span className="text-gray-600 ml-auto">{new Date(c.created_at).toLocaleDateString('bg-BG')}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ── Inline editable cell ────────────────────────────────────

function EditableCell({
  value,
  onChange,
  edited = false,
  className = '',
  type = 'text',
}: {
  value: string;
  onChange: (v: string) => void;
  edited?: boolean;
  className?: string;
  type?: string;
}) {
  const [editing, setEditing] = useState(false);

  if (editing) {
    return (
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={() => setEditing(false)}
        onKeyDown={(e) => e.key === 'Enter' && setEditing(false)}
        autoFocus
        className={`w-full bg-gray-800 border border-[color:var(--oe-accent)] rounded px-1.5 py-0.5 text-sm focus:outline-none ${className}`}
      />
    );
  }

  return (
    <span
      onClick={() => setEditing(true)}
      className={`cursor-pointer ${edited ? 'text-[color:var(--oe-accent)]' : ''} ${className}`}
      title="Click to edit"
    >
      {value || '-'}
    </span>
  );
}
