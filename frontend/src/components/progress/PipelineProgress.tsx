'use client';

import type { JobStatus } from '@/types';

const stages: { key: JobStatus; label: string }[] = [
  { key: 'parsing', label: 'Parse DXF' },
  { key: 'extracting', label: 'Extract Features' },
  { key: 'classifying', label: 'Classify KCCs' },
  { key: 'reporting', label: 'Generate Reports' },
];

const stageOrder: Record<string, number> = {
  queued: -1,
  parsing: 0,
  extracting: 1,
  classifying: 2,
  reporting: 3,
  done: 4,
  failed: -2,
};

function getStepState(
  stageIndex: number,
  currentStatus: JobStatus,
): 'pending' | 'active' | 'done' | 'failed' {
  const currentOrder = stageOrder[currentStatus] ?? -1;

  if (currentStatus === 'failed') {
    // The failed stage is the one matching currentOrder; stages before it are done
    // We don't know exactly which stage failed, so mark all pending
    // Actually we can infer: last non-done stage
    return stageIndex <= currentOrder ? 'done' : stageIndex === currentOrder + 1 ? 'failed' : 'pending';
  }

  if (currentStatus === 'done') return 'done';
  if (stageIndex < currentOrder) return 'done';
  if (stageIndex === currentOrder) return 'active';
  return 'pending';
}

interface Props {
  status: JobStatus;
  progress: number;
}

export function PipelineProgress({ status, progress }: Props) {
  return (
    <div className="space-y-4">
      {/* Steps */}
      <div className="flex items-center justify-between">
        {stages.map((stage, i) => {
          const state = getStepState(i, status);
          return (
            <div key={stage.key} className="flex items-center flex-1">
              {/* Step indicator */}
              <div className="flex flex-col items-center">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium transition-all ${
                    state === 'done'
                      ? 'bg-green-600 text-white'
                      : state === 'active'
                        ? 'bg-blue-600 text-white animate-pulse'
                        : state === 'failed'
                          ? 'bg-red-600 text-white'
                          : 'bg-gray-800 text-gray-600'
                  }`}
                >
                  {state === 'done' ? (
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                  ) : state === 'failed' ? (
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  ) : (
                    i + 1
                  )}
                </div>
                <span
                  className={`mt-2 text-xs whitespace-nowrap ${
                    state === 'active'
                      ? 'text-blue-400'
                      : state === 'done'
                        ? 'text-green-500'
                        : state === 'failed'
                          ? 'text-red-400'
                          : 'text-gray-600'
                  }`}
                >
                  {stage.label}
                </span>
              </div>

              {/* Connector line */}
              {i < stages.length - 1 && (
                <div className="flex-1 mx-3 mt-[-1.25rem]">
                  <div
                    className={`h-0.5 rounded transition-colors ${
                      getStepState(i, status) === 'done' ? 'bg-green-700' : 'bg-gray-800'
                    }`}
                  />
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Progress bar */}
      <div className="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ease-out ${
            status === 'failed' ? 'bg-red-600' : status === 'done' ? 'bg-green-600' : 'bg-blue-600'
          }`}
          style={{ width: `${progress}%` }}
        />
      </div>
      <div className="text-xs text-gray-600 text-right">{progress}%</div>
    </div>
  );
}
