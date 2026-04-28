'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import { useViewerStore } from '@/lib/store';
import { api } from '@/lib/api';
import { MlightcadFrame } from '@/components/viewer/MlightcadFrame';
import { ViewerControls } from '@/components/viewer/ViewerControls';
import { FeatureInspector } from '@/components/features/FeatureInspector';
import { FeatureList } from '@/components/features/FeatureList';
import { KccSummary } from '@/components/features/KccSummary';
import { Breadcrumbs } from '@/components/layout/Breadcrumbs';

export default function DrawingViewerPage() {
  const params = useParams();
  const drawingId = params.id as string;
  const [filename, setFilename] = useState<string | null>(null);
  const {
    loadDrawing,
    renderPacket,
    features,
    kccResults,
    selectedFeatureId,
    selectFeature,
    loading,
    error,
  } = useViewerStore();

  useEffect(() => {
    if (drawingId) {
      loadDrawing(drawingId);
      api.getDrawing(drawingId).then(d => setFilename(d.filename)).catch(() => {});
    }
  }, [drawingId, loadDrawing]);

  useEffect(() => {
    document.title = filename ? `${filename} · View · KCC` : 'View · KCC';
  }, [filename]);

  const selectedFeature = features.find((f) => f.id === selectedFeatureId) ?? null;
  const selectedKcc = kccResults.find((r) => r.feature_id === selectedFeatureId) ?? null;

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      {/* Top bar */}
      <div className="flex-none border-b border-border-light bg-surface-elevated/80 backdrop-blur-sm relative z-20">
        <div className="flex items-center justify-between px-4 py-2 gap-4">
          <div className="min-w-0 flex-1">
            <Breadcrumbs
              items={[
                { label: 'Drawings', href: '/drawings' },
                { label: filename ?? '…', href: `/drawings/${drawingId}` },
                { label: 'View' },
              ]}
            />
          </div>

          <KccSummary features={features} kccResults={kccResults} />

          <ViewerControls />
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Canvas area */}
        <div className="flex-1 relative">
          {error && (
            <div className="absolute inset-0 flex items-center justify-center bg-surface-secondary z-10">
              <div className="text-center">
                <p className="text-red-400 mb-2">Failed to load drawing</p>
                <p className="text-sm text-content-tertiary">{error}</p>
                <button
                  onClick={() => loadDrawing(drawingId)}
                  className="mt-4 px-4 py-2 bg-surface-tertiary hover:bg-gray-700 rounded-lg text-sm transition-colors"
                >
                  Retry
                </button>
              </div>
            </div>
          )}

          {!error && (
            <MlightcadFrame
              drawingId={drawingId}
              renderPacket={renderPacket}
              features={features}
              kccResults={kccResults}
              selectedFeatureId={selectedFeatureId}
              onFeatureClick={selectFeature}
            />
          )}

          {loading && !renderPacket && (
            <div className="absolute top-4 left-4 flex items-center gap-2 bg-surface-elevated/80 backdrop-blur-sm px-3 py-2 rounded-lg z-10 border border-border-light">
              <div className="w-2 h-2 rounded-full bg-[color:var(--oe-accent)] animate-pulse" />
              <span className="text-content-secondary text-xs">Loading KCC overlay…</span>
            </div>
          )}
        </div>

        {/* Right sidebar */}
        <div className="flex-none w-80 border-l border-border-light bg-surface-elevated/50 overflow-y-auto">
          <FeatureInspector feature={selectedFeature} kccResult={selectedKcc} />
        </div>
      </div>

      {/* Bottom feature bar */}
      <div className="flex-none border-t border-border-light bg-surface-elevated/80">
        <FeatureList
          features={features}
          kccResults={kccResults}
          selectedFeatureId={selectedFeatureId}
          onSelect={selectFeature}
        />
      </div>
    </div>
  );
}
