// === Domain Types ===

export interface Drawing {
  id: string;
  filename: string;
  original_format: 'dxf' | 'dwg';
  units: 'mm' | 'in' | null;
  entity_count: number | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export type JobStatus =
  | 'queued'
  | 'parsing'
  | 'extracting'
  | 'classifying'
  | 'reporting'
  | 'done'
  | 'failed';

export interface Job {
  id: string;
  drawing_id: string;
  status: JobStatus;
  progress: number;
  error_message: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
}

export interface Feature {
  id: string;
  feature_type: string;
  description: string;
  centroid_x: number;
  centroid_y: number;
  properties: Record<string, unknown>;
  dimensions?: FeatureDimension[];
  gdt?: FeatureGdt[];
  datum_refs?: string[];
}

export interface FeatureDimension {
  dim_type: string;
  nominal_value: number;
  tolerance_upper: number | null;
  tolerance_lower: number | null;
  raw_text: string | null;
}

export interface FeatureGdt {
  symbol: string;
  tolerance_value: number;
  material_condition: string | null;
  datum_refs: string[];
}

export type KccClassification = 'kcc' | 'important' | 'standard';

export interface KccFactor {
  name: string;
  points: number;
  reason: string;
}

export interface KccResult {
  feature_id: string;
  classification: KccClassification;
  score: number;
  factors: KccFactor[];
  tolerance_chain?: {
    chain_length: number;
    accumulated_tolerance_wc: number;
    accumulated_tolerance_rss: number;
    critical_path: string[];
  } | null;
}

// === Render Packet Types ===

export interface RenderBounds {
  min_x: number;
  min_y: number;
  max_x: number;
  max_y: number;
}

// Per-entity styling fields (from StyledEntity wrapper)
interface EntityStyle {
  color?: string;
  lineweight?: number;
  linetype?: string;
  entity_id?: number;
}

export interface RenderLine extends EntityStyle {
  type: 'line';
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface RenderCircle extends EntityStyle {
  type: 'circle';
  cx: number;
  cy: number;
  r: number;
}

export interface RenderArc extends EntityStyle {
  type: 'arc';
  cx: number;
  cy: number;
  r: number;
  start: number;
  end: number;
}

export interface RenderPolyline extends EntityStyle {
  type: 'polyline';
  points: [number, number][];
  closed: boolean;
}

export interface RenderText extends EntityStyle {
  type: 'text';
  x: number;
  y: number;
  text: string;
  height: number;
  rotation?: number;
}

export interface RenderEllipse extends EntityStyle {
  type: 'ellipse';
  cx: number;
  cy: number;
  rx: number;
  ry: number;
  rotation: number;
  start: number;
  end: number;
}

export type RenderEntity =
  | RenderLine
  | RenderCircle
  | RenderArc
  | RenderPolyline
  | RenderText
  | RenderEllipse;

export interface RenderLayer {
  name: string;
  color: string;
  entities: RenderEntity[];
}

export interface RenderFeature {
  id: string;
  type: string;
  classification: KccClassification;
  cx: number;
  cy: number;
  highlight_entities: number[];
}

export interface RenderPacket {
  bounds: RenderBounds;
  layers: RenderLayer[];
  features: RenderFeature[];
}

// === Auth Types ===

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user_id: string;
}

export interface UploadResponse {
  drawing_id: string;
  job_id: string;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

// === Price Scraping Types ===

export interface ScrapedPriceItem {
  id: string;
  site: string;
  sek_code: string | null;
  sek_group: string | null;
  item_name: string;
  unit: string | null;
  price_min_eur: number | null;
  price_max_eur: number | null;
  currency: string | null;
  mapping_confidence: number | null;
  extraction_confidence: number | null;
  extraction_strategy: string | null;
  is_manual: boolean;
  is_user_edited: boolean;
  notes: string | null;
  captured_at: string;
}

export interface ScrapeSource {
  id: string;
  site_name: string;
  base_url: string;
  enabled: boolean;
  is_builtin: boolean;
  created_at: string;
}

export interface PriceListInfo {
  id: string;
  name: string;
  item_count: number;
  source?: string;
  is_default?: boolean;
  created_at: string;
}

// === KSS Corrections & DRM Types ===

export interface KssCorrectionItem {
  /** kss_line_items.id — enables deterministic row update (no AI). */
  item_id?: string;
  original_sek_code?: string;
  original_description?: string;
  original_quantity?: number;
  original_unit?: string;
  corrected_sek_code?: string;
  corrected_description?: string;
  corrected_quantity?: number;
  corrected_unit?: string;
  corrected_labor_price?: number;
  corrected_material_price?: number;
  correction_type: string;
  source_layer?: string;
  source_block?: string;
  notes?: string;
}

export interface KssCorrectionRecord {
  id: string;
  original_sek_code: string | null;
  original_description: string | null;
  corrected_sek_code: string | null;
  corrected_description: string | null;
  corrected_quantity: number | null;
  corrected_unit: string | null;
  correction_type: string;
  source_layer: string | null;
  notes: string | null;
  created_at: string;
}

// === AI KSS Research Types ===

export interface AiResearchItem {
  id: string;
  sek_group: string;
  sek_code: string;
  description: string;
  unit: string;
  /** Total unit price (material + labor) without VAT — primary value for Образец 9.1 */
  price_eur: number | null;
  material_price_eur: number | null;
  labor_price_eur: number | null;
  /** Market floor — always strictly less than price_max_eur for a real range */
  price_min_eur: number | null;
  /** Market ceiling */
  price_max_eur: number | null;
  source_url: string;
  notes: string | null;
  confidence: number | null;
  approved: boolean;
  edited: boolean;
}

// === KSS Suggestions ===

export interface KssSuggestion {
  id: string;
  sek_code: string;
  description: string;
  unit: string;
  quantity: number;
  unit_price_eur: number;
  total_eur: number;
  labor_price: number | null;
  material_price: number | null;
  confidence: number;
  reasoning: string | null;
  provenance: string | null;
  /** Added in migration 020. Present on every line item. */
  source_entity_id?: string | null;
  source_layer?: string | null;
  centroid_x?: number | null;
  centroid_y?: number | null;
  extraction_method?: ExtractionMethod | null;
  geometry_confidence?: number | null;
  needs_review?: boolean;
}

export type ExtractionMethod =
  | 'polyline_shoelace'
  | 'block_instance_count'
  | 'linear_polyline'
  | 'text_annotation'
  | 'wall_area_from_centerline'
  | 'wall_volume_from_centerline'
  | 'derived_from_primary'
  | 'ai_inferred'
  | 'assumed_default';

/** Short label + short Bulgarian description, used by the pill in the КСС page. */
export function describeExtractionMethod(m: ExtractionMethod | null | undefined): { label: string; title: string; tone: 'trust' | 'assume' | 'flag' } {
  switch (m) {
    case 'polyline_shoelace':        return { label: '⊞ измерено',   title: 'Площ от затворен polyline (Shoelace)',         tone: 'trust' };
    case 'block_instance_count':     return { label: '⊞ броено',     title: 'Преброени реални INSERT блокове',             tone: 'trust' };
    case 'linear_polyline':          return { label: '⊞ измерено',   title: 'Дължина от polyline',                         tone: 'trust' };
    case 'text_annotation':          return { label: '⊞ от текст',   title: 'Прочетено от анотация в чертежа',             tone: 'trust' };
    case 'wall_area_from_centerline':return { label: '〜 предпол.',   title: 'Дължина × приета височина на стената',        tone: 'assume' };
    case 'wall_volume_from_centerline':return{ label: '〜 предпол.',  title: 'Дължина × приета височина × приета дебелина', tone: 'assume' };
    case 'derived_from_primary':     return { label: '〜 изведено',   title: 'Изведено от друго количество',                tone: 'assume' };
    case 'ai_inferred':              return { label: '⚠︎ AI',         title: 'Производство на AI без геометрия',            tone: 'flag' };
    case 'assumed_default':          return { label: '⚠︎ по умолч.',  title: 'Стойност по умолчание — липсва измерване',    tone: 'flag' };
    default:                          return { label: '—',             title: 'Методът на извличане не е записан',           tone: 'assume' };
  }
}

// === KSS Audit Trail ===

export interface KssAuditTrailEntry {
  id: string;
  pipeline_mode: string;
  total_duration_ms: number;
  total_warnings: number;
  total_errors: number;
  overall_confidence: number | null;
  audit_data: Record<string, unknown>;
  user_summary: UserPhaseSummary[] | null;
}

export interface UserPhaseSummary {
  phase_number: number;
  phase_name: string;
  duration_ms: number;
  summary: string;
  highlights: string[];
}

export interface DrmStats {
  total_artifacts: number;
  auto_generated: number;
  user_corrected: number;
  total_corrections: number;
  avg_confidence: number;
  top_confirmed: {
    input_key: string;
    sek_code: string | null;
    times_confirmed: number;
    confidence: number;
    source: string;
  }[];
}

// ═══════════════════════════════════════════════════════════
// ERP Types — Projects, BOQ, Costs, Assemblies, Schedule, EVM
// ═══════════════════════════════════════════════════════════

export interface Project {
  id: string;
  name: string;
  description?: string;
  region: string;
  classification_standard: string;
  currency: string;
  locale: string;
  status: string;
  owner_id: string;
  project_code?: string;
  phase?: string;
  budget_estimate?: number;
  created_at: string;
}

export interface BOQ {
  id: string;
  project_id: string;
  name: string;
  description?: string;
  status: string;
  is_locked: boolean;
  estimate_type: string;
  positions?: BOQPosition[];
  markups?: BOQMarkup[];
  created_at: string;
}

export interface BOQPosition {
  id: string;
  boq_id: string;
  parent_id?: string;
  ordinal: string;
  description: string;
  unit?: string;
  quantity: string;
  unit_rate: string;
  total: string;
  classification: Record<string, string>;
  source: string;
  confidence?: number;
  validation_status: string;
  sort_order: number;
}

export interface BOQMarkup {
  id: string;
  boq_id: string;
  name: string;
  markup_type: string;
  category: string;
  percentage: string;
  fixed_amount: string;
  apply_to: string;
  sort_order: number;
  is_active: boolean;
}

export interface BOQSnapshot {
  id: string;
  boq_id: string;
  name: string;
  created_at: string;
}

export interface CostItem {
  id: string;
  code: string;
  description: string;
  unit?: string;
  rate: string;
  currency: string;
  source: string;
  region?: string;
  classification: Record<string, string>;
  tags: string[];
  is_active: boolean;
}

export interface Assembly {
  id: string;
  code?: string;
  name: string;
  description?: string;
  unit?: string;
  category?: string;
  total_rate: string;
  currency: string;
  is_template: boolean;
  components?: AssemblyComponent[];
}

export interface AssemblyComponent {
  id: string;
  assembly_id: string;
  cost_item_id?: string;
  description?: string;
  factor: number;
  quantity: number;
  unit?: string;
  unit_cost: number;
  total: number;
}

export interface Schedule {
  id: string;
  project_id: string;
  name: string;
  schedule_type: string;
  start_date?: string;
  end_date?: string;
  status: string;
  activities?: Activity[];
}

export interface Activity {
  id: string;
  schedule_id: string;
  parent_id?: string;
  name: string;
  wbs_code?: string;
  start_date?: string;
  end_date?: string;
  duration_days: number;
  progress_pct: string;
  status: string;
  activity_type: string;
  is_critical: boolean;
  total_float?: number;
}

export interface CostModelSnapshot {
  id: string;
  project_id: string;
  period: string;
  planned_cost: number;
  earned_value: number;
  actual_cost: number;
  spi?: number;
  cpi?: number;
  forecast_eac?: number;
}

export interface EvmMetrics {
  spi: number;
  cpi: number;
  sv: number;
  cv: number;
  eac: number;
  etc_val: number;
  vac: number;
  tcpi: number;
  bac: number;
}

export interface ValidationReport {
  id: string;
  target_type: string;
  target_id: string;
  rule_set: string;
  status: string;
  score: number;
  total_rules: number;
  passed: number;
  warnings: number;
  errors: number;
  results: RuleResult[];
}

export interface RuleResult {
  rule_id: string;
  passed: boolean;
  severity: string;
  message: string;
  element_ref?: string;
  suggestion?: string;
}

export interface TenderPackage {
  id: string;
  project_id: string;
  name: string;
  description?: string;
  status: string;
  due_date?: string;
  bids?: Bid[];
}

export interface Bid {
  id: string;
  package_id: string;
  bidder_name: string;
  bidder_email?: string;
  total_amount?: number;
  status: string;
  submitted_at: string;
}
