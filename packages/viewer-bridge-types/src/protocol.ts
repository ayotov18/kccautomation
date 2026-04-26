/**
 * postMessage protocol between the Next.js host (parent) and the iframed
 * mlightcad cad-viewer app. This is the ONLY coupling between the two apps.
 *
 * Versioning: bump BRIDGE_VERSION on any breaking change. Both sides must
 * match; parent rejects messages with a mismatched version.
 */
export const BRIDGE_VERSION = 1 as const;

export type KccClassification = 'kcc' | 'important' | 'standard';

export interface KccFeatureLite {
  id: string;
  cx: number;
  cy: number;
  classification: KccClassification;
  label?: string;
}

export interface LayerInfo {
  name: string;
  color: string;
  visible: boolean;
  entityCount: number;
}

export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

export type ThemeMode = 'dark' | 'light';

export type ZoomAction = 'in' | 'out' | 'fit';

// ── Parent → Iframe ─────────────────────────────────────────

export type HostMessage =
  | { v: typeof BRIDGE_VERSION; type: 'load'; drawingId: string; sourceUrl: string }
  | { v: typeof BRIDGE_VERSION; type: 'setKccFeatures'; features: KccFeatureLite[] }
  | { v: typeof BRIDGE_VERSION; type: 'setSelectedFeature'; featureId: string | null }
  | { v: typeof BRIDGE_VERSION; type: 'setKccOverlayVisible'; visible: boolean }
  | { v: typeof BRIDGE_VERSION; type: 'setLayerVisible'; layer: string; visible: boolean }
  | { v: typeof BRIDGE_VERSION; type: 'zoom'; action: ZoomAction }
  | { v: typeof BRIDGE_VERSION; type: 'theme'; mode: ThemeMode };

// ── Iframe → Parent ─────────────────────────────────────────

export type FrameMessage =
  | { v: typeof BRIDGE_VERSION; type: 'ready' }
  | { v: typeof BRIDGE_VERSION; type: 'loadProgress'; pct: number }
  | { v: typeof BRIDGE_VERSION; type: 'loaded'; bounds: Bounds; layers: LayerInfo[] }
  | { v: typeof BRIDGE_VERSION; type: 'featureClicked'; featureId: string }
  | { v: typeof BRIDGE_VERSION; type: 'entityClicked'; entityId: number }
  | { v: typeof BRIDGE_VERSION; type: 'error'; code: string; message: string };

export type BridgeMessage = HostMessage | FrameMessage;
