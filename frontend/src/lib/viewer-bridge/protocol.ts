/**
 * postMessage protocol — kept in sync with `packages/viewer-bridge-types/src/protocol.ts`.
 *
 * This file is a local copy because the Next.js build does not pull from the
 * monorepo packages directory automatically (frontend is on npm, packages/ is
 * plain TS). Keep both in sync by hand; version bump required on every change.
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

export type HostMessage =
  | { v: typeof BRIDGE_VERSION; type: 'load'; drawingId: string; sourceUrl: string }
  | { v: typeof BRIDGE_VERSION; type: 'setKccFeatures'; features: KccFeatureLite[] }
  | { v: typeof BRIDGE_VERSION; type: 'setSelectedFeature'; featureId: string | null }
  | { v: typeof BRIDGE_VERSION; type: 'setKccOverlayVisible'; visible: boolean }
  | { v: typeof BRIDGE_VERSION; type: 'setLayerVisible'; layer: string; visible: boolean }
  | { v: typeof BRIDGE_VERSION; type: 'zoom'; action: 'in' | 'out' | 'fit' }
  | { v: typeof BRIDGE_VERSION; type: 'theme'; mode: 'dark' | 'light' };

export type FrameMessage =
  | { v: typeof BRIDGE_VERSION; type: 'ready' }
  | { v: typeof BRIDGE_VERSION; type: 'loadProgress'; pct: number }
  | { v: typeof BRIDGE_VERSION; type: 'loaded'; bounds: Bounds; layers: LayerInfo[] }
  | { v: typeof BRIDGE_VERSION; type: 'featureClicked'; featureId: string }
  | { v: typeof BRIDGE_VERSION; type: 'entityClicked'; entityId: number }
  | { v: typeof BRIDGE_VERSION; type: 'error'; code: string; message: string };
