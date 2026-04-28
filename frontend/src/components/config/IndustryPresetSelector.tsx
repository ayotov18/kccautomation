'use client';

import type { IndustryContext } from '@/types/config';
import { INDUSTRY_PRESETS } from '@/types/config';

const accentColors: Record<IndustryContext, string> = {
  automotive: 'border-l-blue-600',
  aerospace: 'border-l-[color:var(--oe-accent)]',
  construction_en1090: 'border-l-[color:var(--oe-accent)]',
};

const selectedBg: Record<IndustryContext, string> = {
  automotive: 'bg-blue-950/30 border-blue-700',
  aerospace: 'bg-[color:var(--oe-accent-soft-bg)] border-[color:var(--oe-accent)]',
  construction_en1090: 'bg-[color:var(--oe-accent-soft-bg)] border-[color:var(--oe-accent)]',
};

interface Props {
  selected: IndustryContext;
  onSelect: (industry: IndustryContext) => void;
}

export function IndustryPresetSelector({ selected, onSelect }: Props) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
      {INDUSTRY_PRESETS.map((preset) => {
        const isSelected = selected === preset.id;
        return (
          <button
            key={preset.id}
            onClick={() => onSelect(preset.id)}
            className={`text-left border-l-4 rounded-lg p-4 transition-all ${accentColors[preset.id]} ${
              isSelected
                ? `${selectedBg[preset.id]} ring-1 ring-inset`
                : 'bg-gray-900 border border-gray-800 hover:bg-gray-800/50'
            }`}
          >
            <div className="font-medium text-sm text-gray-100">{preset.label}</div>
            <div className="text-xs text-gray-500 mt-0.5">{preset.standard}</div>
            <div className="text-xs text-gray-600 mt-2">{preset.description}</div>
            <div className="mt-3 flex gap-3 text-xs text-gray-500">
              <span>KCC &ge; {preset.thresholds.kcc_threshold}</span>
              <span>Important &ge; {preset.thresholds.important_threshold}</span>
            </div>
          </button>
        );
      })}
    </div>
  );
}
