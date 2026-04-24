import { create } from 'zustand';
import type { AnalysisConfig, IndustryContext, IndustryPreset, KccThresholds } from '@/types/config';
import { DEFAULT_ANALYSIS_CONFIG, INDUSTRY_PRESETS } from '@/types/config';
import { api } from './api';

interface ConfigState {
  analysisConfig: AnalysisConfig;
  userDefaults: KccThresholds | null;
  loading: boolean;

  setIndustry: (industry: IndustryContext) => void;
  setThresholds: (thresholds: KccThresholds) => void;
  applyPreset: (preset: IndustryPreset) => void;
  resetToDefaults: () => void;
  loadUserDefaults: () => Promise<void>;
  saveUserDefaults: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  analysisConfig: { ...DEFAULT_ANALYSIS_CONFIG },
  userDefaults: null,
  loading: false,

  setIndustry: (industry) => {
    const preset = INDUSTRY_PRESETS.find((p) => p.id === industry);
    if (preset) {
      set({
        analysisConfig: {
          industry,
          thresholds: { ...preset.thresholds },
        },
      });
    }
  },

  setThresholds: (thresholds) => {
    set((state) => ({
      analysisConfig: {
        ...state.analysisConfig,
        thresholds,
      },
    }));
  },

  applyPreset: (preset) => {
    set({
      analysisConfig: {
        industry: preset.id,
        thresholds: { ...preset.thresholds },
      },
    });
  },

  resetToDefaults: () => {
    const { userDefaults } = get();
    if (userDefaults) {
      set({
        analysisConfig: {
          industry: 'automotive',
          thresholds: { ...userDefaults },
        },
      });
    } else {
      set({ analysisConfig: { ...DEFAULT_ANALYSIS_CONFIG } });
    }
  },

  loadUserDefaults: async () => {
    set({ loading: true });
    try {
      const thresholds = await api.getThresholds();
      set({ userDefaults: thresholds, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  saveUserDefaults: async () => {
    const { analysisConfig } = get();
    set({ loading: true });
    try {
      const saved = await api.updateThresholds(analysisConfig.thresholds);
      set({ userDefaults: saved, loading: false });
    } catch {
      set({ loading: false });
    }
  },
}));
