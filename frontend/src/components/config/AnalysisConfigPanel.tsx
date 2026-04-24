'use client';

import { useState } from 'react';
import { useConfigStore } from '@/lib/configStore';
import { ThresholdEditor } from './ThresholdEditor';

export function AnalysisConfigPanel() {
  const { analysisConfig, setThresholds } = useConfigStore();
  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <div className="space-y-5">
      <div>
        <button
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="flex items-center gap-2 text-sm text-gray-500 hover:text-gray-300 transition-colors"
        >
          <svg
            className={`w-3 h-3 transition-transform ${showAdvanced ? 'rotate-90' : ''}`}
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
              clipRule="evenodd"
            />
          </svg>
          Advanced Configuration
        </button>

        {showAdvanced && (
          <div className="mt-3 p-4 bg-gray-900/50 border border-gray-800 rounded-lg">
            <ThresholdEditor
              thresholds={analysisConfig.thresholds}
              onChange={setThresholds}
            />
          </div>
        )}
      </div>
    </div>
  );
}
