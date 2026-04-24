'use client';

import type { KccThresholds } from '@/types/config';

interface Props {
  thresholds: KccThresholds;
  onChange: (thresholds: KccThresholds) => void;
}

export function ThresholdEditor({ thresholds, onChange }: Props) {
  const max = 15;
  const importantPct = (thresholds.important_threshold / max) * 100;
  const kccPct = (thresholds.kcc_threshold / max) * 100;

  const updateField = (field: 'kcc_threshold' | 'important_threshold', value: number) => {
    const clamped = Math.max(1, Math.min(max, value));
    const next = { ...thresholds, [field]: clamped };

    // Ensure kcc > important
    if (field === 'important_threshold' && clamped >= next.kcc_threshold) {
      next.kcc_threshold = clamped + 1;
    }
    if (field === 'kcc_threshold' && clamped <= next.important_threshold) {
      next.important_threshold = clamped - 1;
    }

    onChange(next);
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-6">
        <div className="flex-1">
          <label className="block text-xs text-gray-500 uppercase tracking-wider mb-1.5">
            Important threshold
          </label>
          <div className="flex items-center gap-2">
            <button
              onClick={() => updateField('important_threshold', thresholds.important_threshold - 1)}
              className="w-8 h-8 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 flex items-center justify-center text-sm transition-colors"
            >
              -
            </button>
            <input
              type="number"
              min={1}
              max={max}
              value={thresholds.important_threshold}
              onChange={(e) => updateField('important_threshold', parseInt(e.target.value) || 1)}
              className="w-16 bg-gray-900 border border-gray-700 rounded-lg px-3 py-1.5 text-center text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
            />
            <button
              onClick={() => updateField('important_threshold', thresholds.important_threshold + 1)}
              className="w-8 h-8 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 flex items-center justify-center text-sm transition-colors"
            >
              +
            </button>
          </div>
        </div>

        <div className="flex-1">
          <label className="block text-xs text-gray-500 uppercase tracking-wider mb-1.5">
            KCC threshold
          </label>
          <div className="flex items-center gap-2">
            <button
              onClick={() => updateField('kcc_threshold', thresholds.kcc_threshold - 1)}
              className="w-8 h-8 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 flex items-center justify-center text-sm transition-colors"
            >
              -
            </button>
            <input
              type="number"
              min={1}
              max={max}
              value={thresholds.kcc_threshold}
              onChange={(e) => updateField('kcc_threshold', parseInt(e.target.value) || 1)}
              className="w-16 bg-gray-900 border border-gray-700 rounded-lg px-3 py-1.5 text-center text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
            />
            <button
              onClick={() => updateField('kcc_threshold', thresholds.kcc_threshold + 1)}
              className="w-8 h-8 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 flex items-center justify-center text-sm transition-colors"
            >
              +
            </button>
          </div>
        </div>
      </div>

      {/* Visual range bar */}
      <div>
        <div className="text-xs text-gray-600 mb-1">Classification ranges</div>
        <div className="h-3 rounded-full overflow-hidden flex">
          <div
            className="bg-green-700/60 transition-all"
            style={{ width: `${importantPct}%` }}
            title={`Standard: 0 - ${thresholds.important_threshold - 1}`}
          />
          <div
            className="bg-yellow-600/60 transition-all"
            style={{ width: `${kccPct - importantPct}%` }}
            title={`Important: ${thresholds.important_threshold} - ${thresholds.kcc_threshold - 1}`}
          />
          <div
            className="bg-red-600/60 transition-all"
            style={{ width: `${100 - kccPct}%` }}
            title={`KCC: ${thresholds.kcc_threshold}+`}
          />
        </div>
        <div className="flex justify-between text-xs text-gray-600 mt-1">
          <span>Standard</span>
          <span>Important</span>
          <span>KCC</span>
        </div>
      </div>
    </div>
  );
}
