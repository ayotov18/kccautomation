'use client';

import type { Feature, KccResult } from '@/types';

interface KccSummaryProps {
  features: Feature[];
  kccResults: KccResult[];
}

export function KccSummary({ features, kccResults }: KccSummaryProps) {
  const total = features.length;
  const kccCount = kccResults.filter((r) => r.classification === 'kcc').length;
  const importantCount = kccResults.filter((r) => r.classification === 'important').length;
  const standardCount = kccResults.filter((r) => r.classification === 'standard').length;

  if (total === 0) return null;

  return (
    <div className="flex items-center gap-4 text-xs">
      <div className="text-gray-500">
        <span className="font-mono font-bold text-gray-300">{total}</span> features
      </div>
      <div className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-red-500" />
        <span className="text-red-400 font-bold">{kccCount}</span>
        <span className="text-gray-600">KCC</span>
      </div>
      <div className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-yellow-500" />
        <span className="text-yellow-400 font-bold">{importantCount}</span>
        <span className="text-gray-600">IMP</span>
      </div>
      <div className="flex items-center gap-1">
        <span className="w-2 h-2 rounded-full bg-green-500" />
        <span className="text-green-400 font-bold">{standardCount}</span>
        <span className="text-gray-600">STD</span>
      </div>
    </div>
  );
}
