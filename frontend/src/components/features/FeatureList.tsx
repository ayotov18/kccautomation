'use client';

import type { Feature, KccResult, KccClassification } from '@/types';

interface FeatureListProps {
  features: Feature[];
  kccResults: KccResult[];
  selectedFeatureId: string | null;
  onSelect: (featureId: string | null) => void;
}

const BORDER_COLORS: Record<KccClassification, string> = {
  kcc: 'border-red-500',
  important: 'border-yellow-500',
  standard: 'border-green-600',
};

const BG_COLORS: Record<KccClassification, string> = {
  kcc: 'bg-red-950/30',
  important: 'bg-yellow-950/30',
  standard: 'bg-green-950/20',
};

const TEXT_COLORS: Record<KccClassification, string> = {
  kcc: 'text-red-400',
  important: 'text-yellow-400',
  standard: 'text-green-500',
};

export function FeatureList({
  features,
  kccResults,
  selectedFeatureId,
  onSelect,
}: FeatureListProps) {
  if (features.length === 0) {
    return (
      <div className="px-4 py-3 text-sm text-gray-600">
        No features extracted
      </div>
    );
  }

  const getClassification = (featureId: string): KccClassification => {
    return kccResults.find((r) => r.feature_id === featureId)?.classification ?? 'standard';
  };

  const getScore = (featureId: string): number | null => {
    return kccResults.find((r) => r.feature_id === featureId)?.score ?? null;
  };

  // Sort: KCC first, then important, then standard
  const sortOrder: Record<KccClassification, number> = { kcc: 0, important: 1, standard: 2 };
  const sorted = [...features].sort((a, b) => {
    const ca = getClassification(a.id);
    const cb = getClassification(b.id);
    return sortOrder[ca] - sortOrder[cb];
  });

  return (
    <div className="flex items-center gap-2 px-4 py-2.5 overflow-x-auto scrollbar-thin">
      {sorted.map((feature) => {
        const classification = getClassification(feature.id);
        const score = getScore(feature.id);
        const isSelected = feature.id === selectedFeatureId;

        return (
          <button
            key={feature.id}
            onClick={() => onSelect(isSelected ? null : feature.id)}
            className={`
              flex-none flex items-center gap-2 px-3 py-1.5 rounded-lg border text-sm transition-all
              ${BORDER_COLORS[classification]}
              ${BG_COLORS[classification]}
              ${isSelected ? 'ring-2 ring-blue-500 ring-offset-1 ring-offset-gray-950' : ''}
              hover:brightness-125
            `}
          >
            <span className={`font-medium ${TEXT_COLORS[classification]}`}>
              {feature.feature_type}
            </span>
            <span className="text-gray-500 text-xs truncate max-w-[120px]">
              {feature.description}
            </span>
            {score !== null && (
              <span className={`text-xs font-bold ${TEXT_COLORS[classification]}`}>
                {score}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
