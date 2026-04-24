'use client';

import { useViewerStore } from '@/lib/store';
import { ReportDownloadPanel } from '@/components/reports/ReportDownloadPanel';
import { KssPanel } from '@/components/kss/KssPanel';
import { AnalyzePanel } from '@/components/analyze/AnalyzePanel';

export function ViewerControls() {
  const { showKccOverlay, toggleKccOverlay, drawingId } = useViewerStore();

  return (
    <div className="flex items-center gap-2">
      {/* KCC overlay toggle — drives hotpoint visibility in the iframe */}
      <button
        onClick={toggleKccOverlay}
        className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
          showKccOverlay
            ? 'bg-red-600/20 text-red-400 border border-red-800'
            : 'bg-gray-800 text-gray-500'
        }`}
        title="Toggle KCC overlay"
      >
        KCC
      </button>

      {/* KSS */}
      <KssPanel drawingId={drawingId} />

      {/* Deep Analyze */}
      <AnalyzePanel drawingId={drawingId} />

      {/* Reports */}
      <div className="w-px h-5 bg-gray-800" />
      <ReportDownloadPanel drawingId={drawingId} />
    </div>
  );
}
