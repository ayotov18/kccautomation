// === Industry & Configuration Types ===

export type IndustryContext = 'automotive' | 'aerospace' | 'construction_en1090';

export interface KccThresholds {
  kcc_threshold: number;
  important_threshold: number;
  tolerance_typical: Record<string, number>;
}

export interface AnalysisConfig {
  industry: IndustryContext;
  thresholds: KccThresholds;
}

export interface IndustryPreset {
  id: IndustryContext;
  label: string;
  standard: string;
  description: string;
  thresholds: KccThresholds;
}

// Default tolerance values by feature type (mm)
const DEFAULT_TOLERANCE_TYPICAL: Record<string, number> = {
  hole_small: 0.1,
  hole_medium: 0.15,
  hole_large: 0.2,
  slot: 0.15,
  linear: 0.2,
  angular: 0.5,
};

export const INDUSTRY_PRESETS: IndustryPreset[] = [
  {
    id: 'automotive',
    label: 'Automotive',
    standard: 'PPAP / IATF 16949',
    description: 'Standard automotive production part approval thresholds',
    thresholds: {
      kcc_threshold: 8,
      important_threshold: 5,
      tolerance_typical: { ...DEFAULT_TOLERANCE_TYPICAL },
    },
  },
  {
    id: 'aerospace',
    label: 'Aerospace',
    standard: 'FAIR / AS9100',
    description: 'Tighter thresholds for aerospace first article inspection',
    thresholds: {
      kcc_threshold: 6,
      important_threshold: 4,
      tolerance_typical: {
        ...DEFAULT_TOLERANCE_TYPICAL,
        hole_small: 0.05,
        hole_medium: 0.08,
        hole_large: 0.1,
        slot: 0.08,
        linear: 0.1,
        angular: 0.25,
      },
    },
  },
  {
    id: 'construction_en1090',
    label: 'Construction',
    standard: 'EN 1090-2',
    description: 'Steel fabrication tolerance verification per execution class',
    thresholds: {
      kcc_threshold: 10,
      important_threshold: 6,
      tolerance_typical: {
        ...DEFAULT_TOLERANCE_TYPICAL,
        hole_small: 0.25,
        hole_medium: 0.5,
        hole_large: 1.0,
        slot: 0.5,
        linear: 1.0,
        angular: 1.0,
      },
    },
  },
];

const CONSTRUCTION_PRESET = INDUSTRY_PRESETS.find(
  (p) => p.id === 'construction_en1090',
)!;

export const DEFAULT_ANALYSIS_CONFIG: AnalysisConfig = {
  industry: 'construction_en1090',
  thresholds: CONSTRUCTION_PRESET.thresholds,
};

// === Pricing defaults (Bulgarian КСС) ===
//
// Values derived from BG industry research (СЕК, УСН, 2026 market).
// Every number is user-configurable on /settings/pricing and persists per-user.

export type PricingCurrency = 'EUR' | 'EUR';

export interface LaborRateBand {
  low: number; // EUR/hour
  high: number; // EUR/hour
}

export interface PricingDefaults {
  currency: PricingCurrency;
  vat_rate_pct: number;
  // ДР — допълнителни разходи
  dr_labor_pct: number;
  dr_light_machinery_pct: number;
  dr_heavy_machinery_pct: number;
  dr_materials_pct: number;
  // Надбавки
  contingency_pct: number;
  profit_pct: number;
  transport_slab_eur: number;
  // Ставки на труда — EUR/час
  labor_rates: {
    mason: LaborRateBand;        // зидар
    formwork: LaborRateBand;     // кофражист
    rebar: LaborRateBand;        // армировач
    painter: LaborRateBand;      // бояджия
    electrician: LaborRateBand;  // електротехник
    plumber: LaborRateBand;      // водопроводчик
    welder: LaborRateBand;       // заварчик
    helper: LaborRateBand;       // помощник
  };
  active_preset: PricingPresetId | null;
}

export type PricingPresetId =
  | 'public_tender'
  | 'private_client'
  | 'emergency_works'
  | 'eu_funded';

export interface PricingPreset {
  id: PricingPresetId;
  label: string;
  description: string;
  overrides: Partial<
    Omit<PricingDefaults, 'labor_rates' | 'currency' | 'active_preset'>
  >;
}

export const DEFAULT_PRICING: PricingDefaults = {
  currency: 'EUR',
  vat_rate_pct: 20,
  dr_labor_pct: 110,
  dr_light_machinery_pct: 100,
  dr_heavy_machinery_pct: 30,
  dr_materials_pct: 12,
  contingency_pct: 10,
  profit_pct: 10,
  transport_slab_eur: 800,
  labor_rates: {
    mason: { low: 9, high: 14 },
    formwork: { low: 9, high: 14 },
    rebar: { low: 9, high: 15 },
    painter: { low: 8, high: 13 },
    electrician: { low: 10, high: 18 },
    plumber: { low: 10, high: 18 },
    welder: { low: 11, high: 20 },
    helper: { low: 4, high: 7 },
  },
  active_preset: 'public_tender',
};

export const PRICING_PRESETS: PricingPreset[] = [
  {
    id: 'public_tender',
    label: 'Обществена поръчка',
    description: 'ЗОП-съвместимо: печалба 10%, непредвидени 10% (законов таван)',
    overrides: {
      dr_labor_pct: 110,
      dr_materials_pct: 12,
      contingency_pct: 10,
      profit_pct: 10,
    },
  },
  {
    id: 'private_client',
    label: 'Частен клиент',
    description: 'По-висока печалба, по-нисък непредвиден буфер',
    overrides: {
      dr_labor_pct: 100,
      dr_materials_pct: 15,
      contingency_pct: 5,
      profit_pct: 18,
    },
  },
  {
    id: 'emergency_works',
    label: 'Спешни / нощни работи',
    description: 'Повишени ставки за нощен труд и ускорени срокове',
    overrides: {
      dr_labor_pct: 160,
      dr_materials_pct: 18,
      contingency_pct: 15,
      profit_pct: 15,
    },
  },
  {
    id: 'eu_funded',
    label: 'ЕС финансирани (ОПОС)',
    description: 'Строга документация, по-ниска печалба, одитируем',
    overrides: {
      dr_labor_pct: 100,
      dr_materials_pct: 10,
      contingency_pct: 10,
      profit_pct: 8,
    },
  },
];
