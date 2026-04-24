'use client';

import { useState } from 'react';
import { api } from '@/lib/api';

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

interface Props {
  drawingId: string | null;
}

export function AnalyzePanel({ drawingId }: Props) {
  const [open, setOpen] = useState(false);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState(0);
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = async () => {
    if (!drawingId) return;
    setRunning(true);
    setDone(false);
    setError(null);
    setProgress(0);

    try {
      const { job_id } = await api.triggerDeepAnalyze(drawingId);

      const poll = setInterval(async () => {
        try {
          const job = await api.getJob(job_id);
          setProgress(job.progress);
          if (job.status === 'done') {
            clearInterval(poll);
            setRunning(false);
            setDone(true);
          } else if (job.status === 'failed') {
            clearInterval(poll);
            setRunning(false);
            setError(job.error_message || 'Analysis failed');
          }
        } catch {
          clearInterval(poll);
          setRunning(false);
          setError('Lost connection');
        }
      }, 1500);
    } catch {
      setRunning(false);
      setError('Failed to start analysis');
    }
  };

  const handleDownload = async () => {
    if (!drawingId) return;
    try {
      const blob = await api.downloadAnalysisJson(drawingId);
      downloadBlob(blob, `deep-analysis-${drawingId}.json`);
    } catch {
      setError('Download failed');
    }
  };

  return (
    <div className="relative z-40">
      <button
        onClick={() => setOpen(!open)}
        className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
          open
            ? 'bg-sky-500 text-gray-900'
            : 'bg-sky-500/90 text-gray-900 hover:bg-sky-400'
        }`}
        title="Deep Analyze — extract all data from DWG"
      >
        Analyze
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-2 w-64 bg-gray-900 border border-gray-700 rounded-lg shadow-xl z-50 p-4 space-y-3">
          <div className="text-xs text-gray-500 uppercase tracking-wider">
            Deep Analysis
          </div>
          <p className="text-xs text-gray-500">
            Extract every detail from the DWG file — layers, styles, dimensions, blocks, all entities — into a comprehensive JSON.
          </p>

          {!running && !done && (
            <button
              onClick={handleAnalyze}
              disabled={!drawingId}
              className="w-full px-3 py-2 bg-sky-500 hover:bg-sky-400 disabled:bg-gray-700 disabled:text-gray-500 rounded-lg text-sm font-medium transition-colors"
            >
              Run Deep Analysis
            </button>
          )}

          {running && (
            <div>
              <div className="flex items-center gap-2 mb-1">
                <div className="w-2 h-2 rounded-full bg-sky-400 animate-pulse" />
                <span className="text-xs text-gray-400">Analyzing...</span>
              </div>
              <div className="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
                <div
                  className="h-full bg-sky-500 rounded-full transition-all duration-500"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}

          {done && (
            <div className="space-y-2">
              <div className="text-xs text-sky-300 flex items-center gap-1">
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Analysis complete
              </div>
              <button
                onClick={handleDownload}
                className="w-full px-3 py-1.5 bg-sky-600 hover:bg-sky-500 rounded text-sm text-white transition-colors"
              >
                Download JSON
              </button>
              <button
                onClick={() => { setDone(false); setProgress(0); }}
                className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
              >
                Run again
              </button>
            </div>
          )}

          {error && <div className="text-xs text-red-400">{error}</div>}
        </div>
      )}
    </div>
  );
}
