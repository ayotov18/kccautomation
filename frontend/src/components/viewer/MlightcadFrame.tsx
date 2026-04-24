'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import { api } from '@/lib/api';
import { useViewerStore } from '@/lib/store';
import { ViewerBridgeHost } from '@/lib/viewer-bridge/host';
import { BRIDGE_VERSION } from '@/lib/viewer-bridge/protocol';
import type { Feature, KccResult, RenderPacket } from '@/types';

/**
 * `/cad-viewer/` is path-mounted on the same origin as Next.js in production.
 * During local dev, the Vite dev server runs on :3002 and we cross origins —
 * `NEXT_PUBLIC_CAD_VIEWER_ORIGIN` overrides the default.
 */
const CAD_VIEWER_ORIGIN =
  process.env.NEXT_PUBLIC_CAD_VIEWER_ORIGIN ??
  (typeof window !== 'undefined' ? window.location.origin : '');

const CAD_VIEWER_PATH =
  process.env.NEXT_PUBLIC_CAD_VIEWER_PATH ?? '/cad-viewer/';

interface Props {
  drawingId: string;
  renderPacket: RenderPacket | null;
  features: Feature[];
  kccResults: KccResult[];
  selectedFeatureId: string | null;
  onFeatureClick: (id: string | null) => void;
}

export function MlightcadFrame({
  drawingId,
  features,
  kccResults,
  selectedFeatureId,
  onFeatureClick,
}: Props) {
  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const bridgeRef = useRef<ViewerBridgeHost | null>(null);
  const [frameError, setFrameError] = useState<string | null>(null);
  const { showKccOverlay } = useViewerStore();

  const iframeSrc = useMemo(() => `${CAD_VIEWER_ORIGIN}${CAD_VIEWER_PATH}`, []);
  const iframeOrigin = useMemo(() => new URL(iframeSrc).origin, [iframeSrc]);

  // Establish bridge once, tear down on unmount.
  useEffect(() => {
    const bridge = new ViewerBridgeHost({
      iframeOrigin,
      getWindow: () => iframeRef.current?.contentWindow ?? null,
    });
    bridgeRef.current = bridge;

    const off = bridge.on(async msg => {
      switch (msg.type) {
        case 'ready':
          // Iframe is mounted. Mint a short-lived token and kick off loading.
          try {
            const { source_url } = await api.mintViewerToken(drawingId);
            const absoluteSrc = source_url.startsWith('http')
              ? source_url
              : `${window.location.origin}${source_url}`;
            bridge.send({
              v: BRIDGE_VERSION,
              type: 'load',
              drawingId,
              sourceUrl: absoluteSrc,
            });
          } catch (err) {
            setFrameError(
              err instanceof Error ? err.message : 'Failed to start viewer session',
            );
          }
          break;
        case 'featureClicked':
          onFeatureClick(msg.featureId);
          break;
        case 'error':
          setFrameError(`${msg.code}: ${msg.message}`);
          break;
        default:
          // loadProgress / loaded / entityClicked — not wired into parent yet
          break;
      }
    });

    return () => {
      off();
      bridge.dispose();
      bridgeRef.current = null;
    };
  }, [drawingId, iframeOrigin, onFeatureClick]);

  // Push KCC features downstream whenever they change.
  useEffect(() => {
    const bridge = bridgeRef.current;
    if (!bridge) return;
    bridge.send({
      v: BRIDGE_VERSION,
      type: 'setKccFeatures',
      features: features.map(f => {
        const kcc = kccResults.find(r => r.feature_id === f.id);
        const classification = kcc?.classification ?? 'standard';
        return {
          id: f.id,
          cx: f.centroid_x ?? 0,
          cy: f.centroid_y ?? 0,
          classification:
            classification === 'kcc' || classification === 'important'
              ? classification
              : 'standard',
        };
      }),
    });
  }, [features, kccResults]);

  useEffect(() => {
    bridgeRef.current?.send({
      v: BRIDGE_VERSION,
      type: 'setSelectedFeature',
      featureId: selectedFeatureId,
    });
  }, [selectedFeatureId]);

  useEffect(() => {
    bridgeRef.current?.send({
      v: BRIDGE_VERSION,
      type: 'setKccOverlayVisible',
      visible: showKccOverlay,
    });
  }, [showKccOverlay]);

  return (
    <div className="absolute inset-0">
      <iframe
        ref={iframeRef}
        src={iframeSrc}
        className="w-full h-full border-0 block"
        title="KCC CAD Viewer"
        // `allow-same-origin` enables localStorage/IndexedDB inside the iframe
        // and lets us tighten the origin check — it does NOT weaken our
        // postMessage security since we still pin the origin on both sides.
        sandbox="allow-scripts allow-same-origin"
        referrerPolicy="no-referrer"
      />
      {frameError && (
        <div className="absolute top-4 left-4 right-4 bg-red-900/60 border border-red-700 rounded-lg px-4 py-2 text-sm text-red-100 z-10">
          CAD viewer error: {frameError}
        </div>
      )}
    </div>
  );
}
