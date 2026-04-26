import { z } from 'zod';
import { BRIDGE_VERSION } from './protocol';

const v = z.literal(BRIDGE_VERSION);

const kccFeatureLite = z.object({
  id: z.string(),
  cx: z.number(),
  cy: z.number(),
  classification: z.enum(['kcc', 'important', 'standard']),
  label: z.string().optional(),
});

const layerInfo = z.object({
  name: z.string(),
  color: z.string(),
  visible: z.boolean(),
  entityCount: z.number().int().nonnegative(),
});

const bounds = z.object({
  minX: z.number(),
  minY: z.number(),
  maxX: z.number(),
  maxY: z.number(),
});

export const hostMessageSchema = z.discriminatedUnion('type', [
  z.object({ v, type: z.literal('load'), drawingId: z.string(), sourceUrl: z.string().url() }),
  z.object({ v, type: z.literal('setKccFeatures'), features: z.array(kccFeatureLite) }),
  z.object({ v, type: z.literal('setSelectedFeature'), featureId: z.string().nullable() }),
  z.object({ v, type: z.literal('setKccOverlayVisible'), visible: z.boolean() }),
  z.object({ v, type: z.literal('setLayerVisible'), layer: z.string(), visible: z.boolean() }),
  z.object({ v, type: z.literal('zoom'), action: z.enum(['in', 'out', 'fit']) }),
  z.object({ v, type: z.literal('theme'), mode: z.enum(['dark', 'light']) }),
]);

export const frameMessageSchema = z.discriminatedUnion('type', [
  z.object({ v, type: z.literal('ready') }),
  z.object({ v, type: z.literal('loadProgress'), pct: z.number().min(0).max(100) }),
  z.object({ v, type: z.literal('loaded'), bounds, layers: z.array(layerInfo) }),
  z.object({ v, type: z.literal('featureClicked'), featureId: z.string() }),
  z.object({ v, type: z.literal('entityClicked'), entityId: z.number().int() }),
  z.object({ v, type: z.literal('error'), code: z.string(), message: z.string() }),
]);

export const bridgeMessageSchema = z.union([hostMessageSchema, frameMessageSchema]);
