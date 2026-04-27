'use client';

import { useEffect, useMemo, useState } from 'react';
import { api } from '@/lib/api';
import type { KssAuditTrailEntry, UserPhaseSummary } from '@/types';

interface Props {
  drawingId: string;
  onClose: () => void;
}

type ViewMode = 'user' | 'dev';

export default function KssAuditTrail({ drawingId, onClose }: Props) {
  const [audits, setAudits] = useState<KssAuditTrailEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [mode, setMode] = useState<ViewMode>('user');
  const [expandedPhases, setExpandedPhases] = useState<Set<number>>(new Set());

  useEffect(() => {
    api.getKssAuditTrail(drawingId)
      .then(data => setAudits(data.audits))
      .catch(err => setError(err.message))
      .finally(() => setLoading(false));
  }, [drawingId]);

  const togglePhase = (phase: number) => {
    setExpandedPhases(prev => {
      const next = new Set(prev);
      if (next.has(phase)) next.delete(phase);
      else next.add(phase);
      return next;
    });
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center">
        <div className="text-white text-lg">Loading audit trail...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center">
        <div className="bg-gray-900 rounded-lg p-6 max-w-md">
          <h3 className="text-red-400 font-bold mb-2">Error Loading Audit Trail</h3>
          <p className="text-gray-300 text-sm">{error}</p>
          <button onClick={onClose} className="mt-4 px-4 py-2 bg-gray-700 rounded text-sm text-white hover:bg-gray-600">
            Close
          </button>
        </div>
      </div>
    );
  }

  const latest = audits[0];
  if (!latest) {
    return (
      <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center">
        <div className="bg-gray-900 rounded-lg p-6 max-w-md">
          <h3 className="text-yellow-400 font-bold mb-2">No Audit Data</h3>
          <p className="text-gray-300 text-sm">No audit trail found for this KSS report. Generate a new KSS to create an audit trail.</p>
          <button onClick={onClose} className="mt-4 px-4 py-2 bg-gray-700 rounded text-sm text-white hover:bg-gray-600">
            Close
          </button>
        </div>
      </div>
    );
  }

  const auditData = latest.audit_data as Record<string, unknown>;
  const userSummary = latest.user_summary as UserPhaseSummary[] | null;

  return (
    <div className="fixed inset-0 bg-black/80 z-50 overflow-y-auto">
      <div className="max-w-5xl mx-auto py-8 px-4">
        {/* Header */}
        <div className="bg-gray-900 rounded-t-lg p-4 flex items-center justify-between sticky top-0 z-10 border-b border-gray-700">
          <div>
            <h2 className="text-xl font-bold text-white">KSS Generation Audit Trail</h2>
            <div className="flex gap-4 mt-1 text-sm text-gray-400">
              <span>Mode: <span className="text-blue-400">{latest.pipeline_mode}</span></span>
              <span>Duration: <span className="text-green-400">{latest.total_duration_ms}ms</span></span>
              <span>Warnings: <span className={latest.total_warnings > 0 ? 'text-yellow-400' : 'text-gray-500'}>{latest.total_warnings}</span></span>
              <span>Errors: <span className={latest.total_errors > 0 ? 'text-red-400' : 'text-gray-500'}>{latest.total_errors}</span></span>
            </div>
          </div>
          <div className="flex items-center gap-3">
            {/* Mode toggle */}
            <div className="flex bg-gray-800 rounded-lg overflow-hidden">
              <button
                onClick={() => setMode('user')}
                className={`px-3 py-1.5 text-sm font-medium transition-colors ${
                  mode === 'user' ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white'
                }`}
              >
                USER
              </button>
              <button
                onClick={() => setMode('dev')}
                className={`px-3 py-1.5 text-sm font-medium transition-colors ${
                  mode === 'dev' ? 'bg-sky-600 text-white' : 'text-gray-400 hover:text-white'
                }`}
              >
                DEV
              </button>
            </div>
            <button onClick={onClose} className="text-gray-400 hover:text-white text-xl">
              &times;
            </button>
          </div>
        </div>

        {/* Timeline */}
        <div className="bg-gray-900 rounded-b-lg">
          {/* Detected structures (modules) — one card per spatial module the
              parser found in the drawing. Hidden when the drawing is single-
              module (or pre-structure-detection legacy data). */}
          <DetectedStructuresPanel data={auditData} drawingId={drawingId} />

          {mode === 'user' && userSummary ? (
            <UserModeView summary={userSummary} />
          ) : (
            <DevModeView data={auditData} expandedPhases={expandedPhases} onToggle={togglePhase} />
          )}

          {/* Warnings/Errors */}
          {(latest.total_warnings > 0 || latest.total_errors > 0) && (
            <div className="border-t border-gray-700 p-4">
              <h3 className="text-sm font-bold text-gray-300 mb-2">Warnings & Errors</h3>
              {((auditData.warnings as Array<{phase: string; message: string}>) || []).map((w, i) => (
                <div key={i} className="text-yellow-400 text-xs mb-1">
                  [{w.phase}] {w.message}
                </div>
              ))}
              {((auditData.errors as Array<{phase: string; message: string}>) || []).map((e, i) => (
                <div key={i} className="text-red-400 text-xs mb-1">
                  [{e.phase}] {e.message}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface StructureAuditRow {
  structure_id?: string;
  structure_index: number;
  label: string;
  bbox_min_x: number;
  bbox_min_y: number;
  bbox_max_x: number;
  bbox_max_y: number;
  dimension_count: number;
  annotation_count: number;
  line_item_count: number;
  subtotal_eur: number;
}

function DetectedStructuresPanel({
  data,
  drawingId,
}: {
  data: Record<string, unknown>;
  drawingId: string;
}) {
  const rows = (data.structures as StructureAuditRow[] | undefined) ?? [];
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [errMsg, setErrMsg] = useState<string | null>(null);

  const selectableIds = useMemo(
    () => rows.map((r) => r.structure_id).filter((s): s is string => !!s),
    [rows],
  );

  if (rows.length === 0) return null;

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const onMerge = async () => {
    if (selected.size < 2) return;
    const ids = [...selected];
    const targetId = ids[0];
    const sourceIds = ids.slice(1);
    setBusy(true);
    setErrMsg(null);
    try {
      await api.mergeStructures(drawingId, sourceIds, targetId);
      window.location.reload();
    } catch (err) {
      setErrMsg(err instanceof Error ? err.message : 'Merge failed');
      setBusy(false);
    }
  };

  const onDelete = async (id: string) => {
    if (!confirm('Delete this module? Its KSS line items will move to the recap.'))
      return;
    setBusy(true);
    setErrMsg(null);
    try {
      await api.deleteStructure(drawingId, id);
      window.location.reload();
    } catch (err) {
      setErrMsg(err instanceof Error ? err.message : 'Delete failed');
      setBusy(false);
    }
  };

  return (
    <div className="border-b border-gray-700 px-4 py-3">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold text-white">
          Detected modules
          <span className="ml-2 text-xs text-gray-400 font-normal">
            {rows.length} spatial cluster{rows.length === 1 ? '' : 's'}
          </span>
        </h3>
        {selectableIds.length > 1 && (
          <div className="flex items-center gap-2 text-xs">
            <span className="text-gray-400">{selected.size} selected</span>
            <button
              onClick={onMerge}
              disabled={selected.size < 2 || busy}
              className="px-2 py-1 rounded border border-blue-500/50 bg-blue-500/10 text-blue-300 disabled:opacity-30 disabled:cursor-not-allowed"
              title="Merge selected modules into the first one"
            >
              Merge {selected.size > 0 ? `(${selected.size})` : ''}
            </button>
            <button
              onClick={() => setSelected(new Set())}
              disabled={selected.size === 0}
              className="px-2 py-1 rounded text-gray-400 hover:text-white disabled:opacity-30"
            >
              Clear
            </button>
          </div>
        )}
      </div>
      {errMsg && (
        <div className="mb-2 text-xs text-red-400 bg-red-900/20 border border-red-500/30 rounded px-2 py-1">
          {errMsg}
        </div>
      )}
      <div
        className="grid gap-2"
        style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))' }}
      >
        {rows.map((s) => {
          const w = Math.max(0, s.bbox_max_x - s.bbox_min_x);
          const h = Math.max(0, s.bbox_max_y - s.bbox_min_y);
          const id = s.structure_id ?? '';
          const isSelected = id !== '' && selected.has(id);
          return (
            <div
              key={s.structure_index}
              className={`rounded border p-2 transition-colors ${
                isSelected ? 'border-blue-400 bg-blue-500/10' : 'border-gray-700 bg-gray-950/40'
              }`}
            >
              <div className="flex items-baseline justify-between">
                <label className="flex items-center gap-2 text-sm text-white font-medium cursor-pointer">
                  {id !== '' && (
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggle(id)}
                      className="cursor-pointer"
                    />
                  )}
                  {s.label}
                </label>
                <div className="text-[10px] text-gray-500">#{s.structure_index + 1}</div>
              </div>
              <div className="mt-1 text-[11px] text-gray-400">
                bbox {w.toFixed(0)} × {h.toFixed(0)}
              </div>
              <div className="mt-1 grid grid-cols-2 gap-x-2 text-[11px] text-gray-300">
                <div>
                  dims: <span className="text-blue-300">{s.dimension_count}</span>
                </div>
                <div>
                  anns: <span className="text-blue-300">{s.annotation_count}</span>
                </div>
                <div>
                  lines: <span className="text-blue-300">{s.line_item_count}</span>
                </div>
                <div className="text-right text-emerald-300 font-mono">
                  {s.subtotal_eur.toFixed(0)}
                </div>
              </div>
              {id !== '' && (
                <div className="mt-1.5 flex justify-end">
                  <button
                    onClick={() => onDelete(id)}
                    disabled={busy}
                    className="text-[10px] text-red-400 hover:text-red-300 disabled:opacity-30"
                    title="Discard this module — line items move to the recap"
                  >
                    Delete
                  </button>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function UserModeView({ summary }: { summary: UserPhaseSummary[] }) {
  return (
    <div className="p-4 space-y-4">
      {summary.map(phase => (
        <div key={phase.phase_number} className="border border-gray-700 rounded-lg p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-white font-medium">
              <span className="text-blue-400 mr-2">Phase {phase.phase_number}</span>
              {phase.phase_name}
            </h3>
            {phase.duration_ms > 0 && (
              <span className="text-xs text-gray-500">{phase.duration_ms}ms</span>
            )}
          </div>
          <p className="text-gray-300 text-sm mb-2">{phase.summary}</p>
          {phase.highlights.length > 0 && (
            <ul className="space-y-1">
              {phase.highlights.map((h, i) => (
                <li key={i} className="text-gray-400 text-xs pl-3 border-l-2 border-gray-700">
                  {h}
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </div>
  );
}

function DevModeView({
  data,
  expandedPhases,
  onToggle,
}: {
  data: Record<string, unknown>;
  expandedPhases: Set<number>;
  onToggle: (phase: number) => void;
}) {
  const phases = [
    { num: 1, key: 'phase1_upload', title: 'Upload & Parse' },
    { num: 2, key: 'phase2_analysis', title: 'Analysis & Features' },
    { num: 3, key: 'phase3_quantities', title: 'Quantity Calculation' },
    { num: 4, key: 'phase4_prices', title: 'Price Research' },
    { num: 5, key: 'phase5_generation', title: 'KSS Generation' },
    { num: 6, key: 'phase6_report', title: 'Final Report' },
  ];

  return (
    <div className="p-4 space-y-2">
      {phases.map(phase => {
        const phaseData = data[phase.key] as Record<string, unknown> | null | undefined;
        if (!phaseData && phase.key === 'phase4_prices') return null; // Optional phase
        const isExpanded = expandedPhases.has(phase.num);

        return (
          <div key={phase.num} className="border border-gray-700 rounded-lg overflow-hidden">
            <button
              onClick={() => onToggle(phase.num)}
              className="w-full flex items-center justify-between p-3 hover:bg-gray-800 transition-colors"
            >
              <div className="flex items-center gap-2">
                <span className="text-sky-300 font-mono text-xs">P{phase.num}</span>
                <span className="text-white text-sm font-medium">{phase.title}</span>
              </div>
              <span className="text-gray-500 text-xs">{isExpanded ? '[-]' : '[+]'}</span>
            </button>
            {isExpanded && phaseData && (
              <div className="border-t border-gray-700 p-3">
                <pre className="text-xs text-gray-300 overflow-x-auto whitespace-pre-wrap font-mono">
                  {JSON.stringify(phaseData, null, 2)}
                </pre>
              </div>
            )}
          </div>
        );
      })}

      {/* Timings */}
      {Array.isArray(data.timings) && (
        <div className="border border-gray-700 rounded-lg p-3">
          <h3 className="text-sky-300 font-mono text-xs mb-2">TIMINGS</h3>
          <div className="flex flex-wrap gap-3">
            {(data.timings as Array<{phase: string; duration_ms: number}>).map((t, i) => (
              <span key={i} className="text-xs text-gray-400">
                {t.phase}: <span className="text-green-400">{t.duration_ms}ms</span>
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
