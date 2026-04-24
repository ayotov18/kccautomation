import { create } from 'zustand';
import type {
  Drawing,
  Feature,
  KccResult,
  RenderPacket,
} from '@/types';
import { api } from './api';

// === Auth Store ===

interface AuthState {
  isAuthenticated: boolean;
  userId: string | null;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => void;
  checkAuth: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  userId: null,

  login: async (email, password) => {
    const res = await api.login(email, password);
    set({ isAuthenticated: true, userId: res.user_id });
  },

  register: async (email, password) => {
    const res = await api.register(email, password);
    set({ isAuthenticated: true, userId: res.user_id });
  },

  logout: () => {
    api.clearTokens();
    set({ isAuthenticated: false, userId: null });
  },

  checkAuth: () => {
    set({ isAuthenticated: api.isAuthenticated });
  },
}));

// === Drawings Store ===

interface DrawingsState {
  drawings: Drawing[];
  loading: boolean;
  error: string | null;
  fetchDrawings: () => Promise<void>;
}

export const useDrawingsStore = create<DrawingsState>((set) => ({
  drawings: [],
  loading: false,
  error: null,

  fetchDrawings: async () => {
    set({ loading: true, error: null });
    try {
      const drawings = await api.listDrawings();
      set({ drawings, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch drawings',
        loading: false,
      });
    }
  },
}));

// === Viewer Store ===

interface ViewerState {
  drawingId: string | null;
  renderPacket: RenderPacket | null;
  features: Feature[];
  kccResults: KccResult[];
  selectedFeatureId: string | null;
  visibleLayers: Set<string>;
  showKccOverlay: boolean;
  loading: boolean;
  error: string | null;

  loadDrawing: (drawingId: string) => Promise<void>;
  selectFeature: (featureId: string | null) => void;
  toggleLayer: (layerName: string) => void;
  setAllLayersVisible: (visible: boolean) => void;
  toggleKccOverlay: () => void;
  getKccForFeature: (featureId: string) => KccResult | undefined;
}

export const useViewerStore = create<ViewerState>((set, get) => ({
  drawingId: null,
  renderPacket: null,
  features: [],
  kccResults: [],
  selectedFeatureId: null,
  visibleLayers: new Set<string>(),
  showKccOverlay: true,
  loading: false,
  error: null,

  loadDrawing: async (drawingId: string) => {
    set({ loading: true, error: null, drawingId });
    try {
      const [renderPacket, features, kccResults] = await Promise.all([
        api.getRenderPacket(drawingId),
        api.getFeatures(drawingId),
        api.getKccResults(drawingId),
      ]);
      const visibleLayers = new Set(renderPacket.layers.map((l) => l.name));
      set({
        renderPacket,
        features,
        kccResults,
        visibleLayers,
        loading: false,
        selectedFeatureId: null,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to load drawing',
        loading: false,
      });
    }
  },

  selectFeature: (featureId) => {
    set({ selectedFeatureId: featureId });
  },

  toggleLayer: (layerName) => {
    const { visibleLayers } = get();
    const next = new Set(visibleLayers);
    if (next.has(layerName)) {
      next.delete(layerName);
    } else {
      next.add(layerName);
    }
    set({ visibleLayers: next });
  },

  setAllLayersVisible: (visible) => {
    const { renderPacket } = get();
    if (!renderPacket) return;
    if (visible) {
      set({ visibleLayers: new Set(renderPacket.layers.map((l) => l.name)) });
    } else {
      set({ visibleLayers: new Set() });
    }
  },

  toggleKccOverlay: () => {
    set((state) => ({ showKccOverlay: !state.showKccOverlay }));
  },

  getKccForFeature: (featureId) => {
    return get().kccResults.find((r) => r.feature_id === featureId);
  },
}));
