import type {
  AiResearchItem,
  AuthResponse,
  Drawing,
  DrmStats,
  Feature,
  Job,
  KccResult,
  KssAuditTrailEntry,
  KssCorrectionItem,
  KssCorrectionRecord,
  RenderPacket,
  ScrapedPriceItem,
  ScrapeSource,
  UploadResponse,
} from '@/types';
import type { KccThresholds } from '@/types/config';

const API_BASE = '/api/v1';

class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

class ApiClient {
  private accessToken: string | null = null;
  private refreshToken: string | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this.accessToken = localStorage.getItem('kcc_access_token');
      this.refreshToken = localStorage.getItem('kcc_refresh_token');
    }
  }

  private setTokens(access: string, refresh: string) {
    this.accessToken = access;
    this.refreshToken = refresh;
    if (typeof window !== 'undefined') {
      localStorage.setItem('kcc_access_token', access);
      localStorage.setItem('kcc_refresh_token', refresh);
    }
  }

  clearTokens() {
    this.accessToken = null;
    this.refreshToken = null;
    if (typeof window !== 'undefined') {
      localStorage.removeItem('kcc_access_token');
      localStorage.removeItem('kcc_refresh_token');
    }
  }

  get isAuthenticated(): boolean {
    return this.accessToken !== null;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {},
  ): Promise<T> {
    const headers: Record<string, string> = {
      ...(options.headers as Record<string, string>),
    };

    if (this.accessToken) {
      headers['Authorization'] = `Bearer ${this.accessToken}`;
    }

    if (
      !(options.body instanceof FormData) &&
      !headers['Content-Type']
    ) {
      headers['Content-Type'] = 'application/json';
    }

    const response = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    });

    if (response.status === 401 && this.refreshToken) {
      const refreshed = await this.tryRefresh();
      if (refreshed) {
        headers['Authorization'] = `Bearer ${this.accessToken}`;
        const retryResponse = await fetch(`${API_BASE}${path}`, {
          ...options,
          headers,
        });
        if (!retryResponse.ok) {
          await this.handleError(retryResponse);
        }
        return retryResponse.json();
      }
      this.clearTokens();
      throw new ApiError(401, 'AUTH_EXPIRED', 'Session expired. Please log in again.');
    }

    if (!response.ok) {
      await this.handleError(response);
    }

    return response.json();
  }

  private async requestBlob(path: string): Promise<Blob> {
    const headers: Record<string, string> = {};
    if (this.accessToken) {
      headers['Authorization'] = `Bearer ${this.accessToken}`;
    }

    const response = await fetch(`${API_BASE}${path}`, { headers });

    if (!response.ok) {
      await this.handleError(response);
    }

    return response.blob();
  }

  private async handleError(response: Response): Promise<never> {
    let code = 'UNKNOWN_ERROR';
    let message = `Request failed with status ${response.status}`;
    let details: Record<string, unknown> | undefined;

    try {
      const body = await response.json();
      if (body.error) {
        code = body.error.code || code;
        message = body.error.message || message;
        details = body.error.details;
      }
    } catch {
      // Response body is not JSON
    }

    throw new ApiError(response.status, code, message, details);
  }

  private async tryRefresh(): Promise<boolean> {
    try {
      const response = await fetch(`${API_BASE}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: this.refreshToken }),
      });

      if (!response.ok) return false;

      const data: AuthResponse = await response.json();
      this.setTokens(data.access_token, data.refresh_token);
      return true;
    } catch {
      return false;
    }
  }

  // === Auth ===

  async login(email: string, password: string): Promise<AuthResponse> {
    const data = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setTokens(data.access_token, data.refresh_token);
    return data;
  }

  async register(email: string, password: string): Promise<AuthResponse> {
    const data = await this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setTokens(data.access_token, data.refresh_token);
    return data;
  }

  // === Drawings ===

  async uploadDrawing(file: File): Promise<UploadResponse> {
    const formData = new FormData();
    formData.append('file', file);

    return this.request<UploadResponse>('/drawings/upload', {
      method: 'POST',
      body: formData,
    });
  }

  async listDrawings(): Promise<Drawing[]> {
    return this.request<Drawing[]>('/drawings');
  }

  async getDrawing(id: string): Promise<Drawing> {
    return this.request<Drawing>(`/drawings/${id}`);
  }

  async deleteDrawing(id: string): Promise<void> {
    await this.request(`/drawings/${id}`, { method: 'DELETE' });
  }

  // === Jobs ===

  async getJob(id: string): Promise<Job> {
    return this.request<Job>(`/jobs/${id}`);
  }

  // === Reports ===

  async getReportJson(drawingId: string): Promise<Record<string, unknown>> {
    return this.request<Record<string, unknown>>(`/reports/${drawingId}`);
  }

  async downloadReportPdf(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}/pdf`);
  }

  async downloadReportCsv(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}/csv`);
  }

  // === Features ===

  async getFeatures(drawingId: string): Promise<Feature[]> {
    return this.request<Feature[]>(`/features/${drawingId}`);
  }

  async getKccResults(drawingId: string): Promise<KccResult[]> {
    return this.request<KccResult[]>(`/reports/${drawingId}/kcc`);
  }

  // === Render ===

  async getRenderPacket(drawingId: string): Promise<RenderPacket> {
    return this.request<RenderPacket>(`/render/${drawingId}`);
  }

  // === Viewer (iframed mlightcad) ===

  async mintViewerToken(drawingId: string): Promise<{ source_url: string; expires_in: number }> {
    return this.request<{ source_url: string; expires_in: number }>(
      `/drawings/${drawingId}/viewer-token`,
      { method: 'POST' },
    );
  }

  // === Pricing defaults (per-user) ===

  async getPricingDefaults(): Promise<import('@/types/config').PricingDefaults | null> {
    try {
      return await this.request<import('@/types/config').PricingDefaults>(
        '/config/pricing-defaults',
      );
    } catch {
      return null;
    }
  }

  async setPricingDefaults(
    defaults: import('@/types/config').PricingDefaults,
  ): Promise<{ ok: true }> {
    return this.request<{ ok: true }>('/config/pricing-defaults', {
      method: 'PUT',
      body: JSON.stringify(defaults),
    });
  }

  // === KSS ===

  async generateKss(drawingId: string, priceListId?: string): Promise<{ job_id: string }> {
    return this.request<{ job_id: string }>(`/drawings/${drawingId}/generate-kss`, {
      method: 'POST',
      body: JSON.stringify({ price_list_id: priceListId ?? null }),
    });
  }

  async uploadPriceList(file: File): Promise<{ id: string; name: string; item_count: number }> {
    const formData = new FormData();
    formData.append('file', file);
    return this.request<{ id: string; name: string; item_count: number }>('/price-lists/upload', {
      method: 'POST',
      body: formData,
    });
  }

  async listPriceLists(): Promise<{ id: string; name: string; item_count: number; created_at: string }[]> {
    return this.request('/price-lists');
  }

  async downloadKssExcel(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}/kss/excel`);
  }

  async downloadKssPdf(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}/kss/pdf`);
  }

  // === Deep Analyze ===

  async triggerDeepAnalyze(drawingId: string): Promise<{ job_id: string }> {
    return this.request<{ job_id: string }>(`/drawings/${drawingId}/deep-analyze`, {
      method: 'POST',
    });
  }

  async downloadAnalysisJson(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}/analysis`);
  }

  // === Price Scraping ===

  async triggerScrape(sourceIds?: string[]): Promise<{ job_id: string }> {
    return this.request<{ job_id: string }>('/prices/scrape', {
      method: 'POST',
      body: JSON.stringify({ source_ids: sourceIds ?? [] }),
    });
  }

  async listScrapedPrices(params?: { site?: string; category?: string; search?: string; source_type?: string; limit?: number; offset?: number }): Promise<{ items: ScrapedPriceItem[]; total: number }> {
    const qs = new URLSearchParams();
    if (params?.site) qs.set('site', params.site);
    if (params?.category) qs.set('category', params.category);
    if (params?.search) qs.set('search', params.search);
    if (params?.source_type) qs.set('source_type', params.source_type);
    if (params?.limit) qs.set('limit', String(params.limit));
    if (params?.offset) qs.set('offset', String(params.offset));
    const query = qs.toString();
    return this.request(`/prices/scraped${query ? `?${query}` : ''}`);
  }

  async createPriceRow(data: { sek_code?: string; item_name: string; category?: string; unit?: string; price_min_lv?: number; price_max_lv?: number; price_min_eur?: number; price_max_eur?: number; notes?: string }): Promise<{ id: string }> {
    return this.request('/prices/rows', { method: 'POST', body: JSON.stringify(data) });
  }

  async updatePriceRow(id: string, data: { sek_code?: string; item_name?: string; unit?: string; price_min_lv?: number; price_max_lv?: number; price_min_eur?: number; price_max_eur?: number; notes?: string }): Promise<void> {
    await this.request(`/prices/rows/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  }

  async archivePriceRow(id: string): Promise<void> {
    await this.request(`/prices/rows/${id}`, { method: 'DELETE' });
  }

  async listScrapeSources(): Promise<ScrapeSource[]> {
    return this.request<ScrapeSource[]>('/prices/sources');
  }

  async addScrapeSource(siteName: string, baseUrl: string): Promise<ScrapeSource> {
    return this.request<ScrapeSource>('/prices/sources', {
      method: 'POST',
      body: JSON.stringify({ site_name: siteName, base_url: baseUrl }),
    });
  }

  async toggleScrapeSource(id: string, enabled: boolean): Promise<void> {
    await this.request(`/prices/sources/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ enabled }),
    });
  }

  async deleteScrapeSource(id: string): Promise<void> {
    await this.request(`/prices/sources/${id}`, { method: 'DELETE' });
  }

  async setDefaultPriceList(id: string): Promise<void> {
    await this.request(`/price-lists/${id}/default`, { method: 'PUT' });
  }

  // === Config ===

  async getThresholds(): Promise<KccThresholds> {
    return this.request<KccThresholds>('/config/thresholds');
  }

  async updateThresholds(config: KccThresholds): Promise<KccThresholds> {
    return this.request<KccThresholds>('/config/thresholds', {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  // === AI KSS (Perplexity + Opus pipeline) ===

  async triggerAiKssResearch(drawingId: string): Promise<{ job_id: string; session_id: string }> {
    return this.request(`/drawings/${drawingId}/generate-ai-kss`, { method: 'POST' });
  }

  async getAiKssStatus(drawingId: string): Promise<{ session_id: string; status: string; progress: number; model: string | null; error: string | null }> {
    return this.request(`/drawings/${drawingId}/ai-kss/status`);
  }

  async getAiKssResearchItems(drawingId: string): Promise<AiResearchItem[]> {
    return this.request(`/drawings/${drawingId}/ai-kss/items`);
  }

  async updateAiKssItem(drawingId: string, itemId: string, fields: Record<string, unknown>): Promise<void> {
    await this.request(`/drawings/${drawingId}/ai-kss/items/${itemId}`, {
      method: 'PUT',
      body: JSON.stringify(fields),
    });
  }

  // === Drawing Summary (Overview page) ===

  async getDrawingSummary(drawingId: string): Promise<Record<string, unknown>> {
    return this.request(`/drawings/${drawingId}/summary`);
  }

  // === KSS Data (for frontend table) ===

  async getKssData(drawingId: string): Promise<Record<string, unknown>> {
    return this.request(`/reports/${drawingId}/kss/data`);
  }

  async getKssAuditTrail(drawingId: string): Promise<{ audits: KssAuditTrailEntry[] }> {
    return this.request(`/reports/${drawingId}/kss/audit`);
  }

  async acceptSuggestion(drawingId: string, itemId: string, edits?: { edited_sek_code?: string; edited_description?: string; edited_quantity?: number; edited_unit_price?: number }): Promise<{ status: string }> {
    return this.request(`/reports/${drawingId}/kss/suggestions/${itemId}/accept`, {
      method: 'POST',
      body: JSON.stringify(edits || {}),
    });
  }

  async rejectSuggestion(drawingId: string, itemId: string): Promise<{ status: string }> {
    return this.request(`/reports/${drawingId}/kss/suggestions/${itemId}/reject`, { method: 'POST' });
  }

  async addKssItem(drawingId: string, item: { sek_code: string; description: string; unit: string; quantity: number; unit_price_lv: number }): Promise<{ status: string; item_id: string }> {
    return this.request(`/reports/${drawingId}/kss/items`, {
      method: 'POST',
      body: JSON.stringify(item),
    });
  }

  async finalizeKss(drawingId: string): Promise<{ status: string; item_count: number; total_with_vat_bgn: number }> {
    return this.request(`/reports/${drawingId}/kss/finalize`, { method: 'POST' });
  }

  async renameStructure(drawingId: string, structureId: string, label: string): Promise<{ label: string }> {
    return this.request(`/drawings/${drawingId}/structures/${structureId}`, {
      method: 'PUT',
      body: JSON.stringify({ label }),
    });
  }

  async mergeStructures(drawingId: string, sourceIds: string[], targetId: string): Promise<{
    merged_into: string; removed: string[]; new_bbox: number[];
  }> {
    return this.request(`/drawings/${drawingId}/structures/merge`, {
      method: 'POST',
      body: JSON.stringify({ source_ids: sourceIds, target_id: targetId }),
    });
  }

  async deleteStructure(drawingId: string, structureId: string): Promise<{ deleted: boolean }> {
    return this.request(`/drawings/${drawingId}/structures/${structureId}/delete`, {
      method: 'POST',
    });
  }

  // === AI drawing summary (bilingual, redactable) ===

  async getAiSummary(drawingId: string): Promise<{
    summary_en: string | null;
    summary_bg: string | null;
    generated_at: string | null;
    edited_at: string | null;
    model: string | null;
  }> {
    return this.request(`/drawings/${drawingId}/ai-summary`);
  }

  async saveAiSummary(
    drawingId: string,
    payload: { summary_en?: string; summary_bg?: string },
  ): Promise<{ saved: boolean }> {
    return this.request(`/drawings/${drawingId}/ai-summary`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    });
  }

  async regenerateAiSummary(drawingId: string): Promise<{ job_id: string }> {
    return this.request(`/drawings/${drawingId}/ai-summary/regenerate`, {
      method: 'POST',
    });
  }

  // === Price corpus (self-hosted RAG) ===

  async importPriceCorpus(file: File): Promise<{
    import_id: string; filename: string; sheet_count: number;
    row_count: number; skipped_count: number; deduped: boolean;
  }> {
    const form = new FormData();
    form.append('file', file);
    return this.request(`/price-corpus/import`, { method: 'POST', body: form });
  }

  async listCorpusImports(): Promise<{
    imports: Array<{ id: string; filename: string; sheet_count: number; row_count: number; skipped_count: number; imported_at: string }>;
    total_corpus_rows: number;
  }> {
    return this.request(`/price-corpus/imports`);
  }

  async listCorpus(opts?: { q?: string; limit?: number; offset?: number }): Promise<{
    rows: Array<{
      id: string; sek_code: string | null; description: string; unit: string;
      quantity: number | null; material_price_lv: number | null;
      labor_price_lv: number | null; total_unit_price_lv: number | null;
      currency: string; source_sheet: string | null; source_row: number | null;
      import_id: string | null; created_at: string;
    }>;
    total: number; limit: number; offset: number;
  }> {
    const qs = new URLSearchParams();
    if (opts?.q) qs.set('q', opts.q);
    if (opts?.limit !== undefined) qs.set('limit', String(opts.limit));
    if (opts?.offset !== undefined) qs.set('offset', String(opts.offset));
    const q = qs.toString();
    return this.request(`/price-corpus${q ? `?${q}` : ''}`);
  }

  async deleteCorpusImport(importId: string): Promise<{ deleted: boolean }> {
    return this.request(`/price-corpus/imports/${importId}`, { method: 'DELETE' });
  }

  async triggerAiKssGeneration(drawingId: string, mode: 'ai' | 'rag' | 'hybrid' = 'ai'): Promise<{
    job_id: string; session_id: string; mode: string;
  }> {
    return this.request(`/drawings/${drawingId}/ai-kss/generate`, {
      method: 'POST',
      body: JSON.stringify({ mode }),
    });
  }

  // === KSS Corrections & DRM ===

  async submitCorrections(drawingId: string, items: KssCorrectionItem[]): Promise<{ corrections_saved: number; drm_artifacts_updated: number }> {
    return this.request(`/reports/${drawingId}/kss/corrections`, {
      method: 'POST',
      body: JSON.stringify({ items }),
    });
  }

  async listCorrections(drawingId: string): Promise<KssCorrectionRecord[]> {
    return this.request(`/reports/${drawingId}/kss/corrections`);
  }

  async getDrmStats(): Promise<DrmStats> {
    return this.request('/drm/stats');
  }

  // === Report JSON ===

  async downloadReportJson(drawingId: string): Promise<Blob> {
    return this.requestBlob(`/reports/${drawingId}`);
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: Projects
  // ═══════════════════════════════════════════════════════════

  async listProjects(): Promise<Array<{ id: string; name: string; region: string; status: string; created_at: string }>> {
    return this.request('/projects');
  }

  async createProject(data: { name: string; description?: string; region?: string; currency?: string }): Promise<{ id: string }> {
    return this.request('/projects', { method: 'POST', body: JSON.stringify(data) });
  }

  async getProject(id: string): Promise<Record<string, unknown>> {
    return this.request(`/projects/${id}`);
  }

  async updateProject(id: string, data: Record<string, unknown>): Promise<void> {
    return this.request(`/projects/${id}`, { method: 'PUT', body: JSON.stringify(data) });
  }

  async deleteProject(id: string): Promise<void> {
    return this.request(`/projects/${id}`, { method: 'DELETE' });
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: BOQ
  // ═══════════════════════════════════════════════════════════

  async listBoqs(projectId: string): Promise<Array<{ id: string; name: string; status: string }>> {
    return this.request(`/boq?project_id=${projectId}`);
  }

  async createBoq(data: { project_id: string; name: string; description?: string }): Promise<{ id: string }> {
    return this.request('/boq', { method: 'POST', body: JSON.stringify(data) });
  }

  async getBoq(boqId: string): Promise<Record<string, unknown>> {
    return this.request(`/boq/${boqId}`);
  }

  async createPosition(boqId: string, data: { ordinal: string; description: string; unit?: string; quantity?: string; unit_rate?: string }): Promise<{ id: string }> {
    return this.request(`/boq/${boqId}/positions`, { method: 'POST', body: JSON.stringify(data) });
  }

  async updatePosition(positionId: string, data: Record<string, unknown>): Promise<void> {
    return this.request(`/boq/positions/${positionId}`, { method: 'PUT', body: JSON.stringify(data) });
  }

  async deletePosition(positionId: string): Promise<void> {
    return this.request(`/boq/positions/${positionId}`, { method: 'DELETE' });
  }

  async getMarkups(boqId: string): Promise<Array<Record<string, unknown>>> {
    return this.request(`/boq/${boqId}/markups`);
  }

  async createMarkup(boqId: string, data: Record<string, unknown>): Promise<{ id: string }> {
    return this.request(`/boq/${boqId}/markups`, { method: 'POST', body: JSON.stringify(data) });
  }

  async applyDefaultMarkups(boqId: string, region: string): Promise<void> {
    return this.request(`/boq/${boqId}/markups/apply-defaults`, { method: 'POST', body: JSON.stringify({ region }) });
  }

  async computeGrandTotal(boqId: string): Promise<{ direct_cost: number; markups: Array<Record<string, unknown>>; grand_total: number }> {
    return this.request(`/boq/${boqId}/grand-total`);
  }

  async listSnapshots(boqId: string): Promise<Array<{ id: string; name: string; created_at: string }>> {
    return this.request(`/boq/${boqId}/snapshots`);
  }

  async createSnapshot(boqId: string, name: string): Promise<{ id: string }> {
    return this.request(`/boq/${boqId}/snapshots`, { method: 'POST', body: JSON.stringify({ name }) });
  }

  async restoreSnapshot(boqId: string, snapshotId: string): Promise<void> {
    return this.request(`/boq/${boqId}/snapshots/${snapshotId}/restore`, { method: 'POST' });
  }

  async validateBoq(boqId: string): Promise<Record<string, unknown>> {
    return this.request(`/boq/${boqId}/validate`, { method: 'POST' });
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: Cost Database
  // ═══════════════════════════════════════════════════════════

  async searchCosts(query: string, region?: string, limit?: number): Promise<Array<{ id: string; code: string; description: string; unit: string; rate: string; region: string }>> {
    const params = new URLSearchParams({ q: query });
    if (region) params.set('region', region);
    if (limit) params.set('limit', String(limit));
    return this.request(`/costs/search?${params}`);
  }

  async importCosts(file: File): Promise<{ imported: number }> {
    const form = new FormData();
    form.append('file', file);
    return this.request('/costs/import', { method: 'POST', body: form });
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: Assemblies
  // ═══════════════════════════════════════════════════════════

  async listAssemblies(projectId?: string): Promise<Array<Record<string, unknown>>> {
    const url = projectId ? `/assemblies?project_id=${projectId}` : '/assemblies';
    return this.request(url);
  }

  async createAssembly(data: Record<string, unknown>): Promise<{ id: string }> {
    return this.request('/assemblies', { method: 'POST', body: JSON.stringify(data) });
  }

  async getAssembly(id: string): Promise<Record<string, unknown>> {
    return this.request(`/assemblies/${id}`);
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: Schedule
  // ═══════════════════════════════════════════════════════════

  async getSchedule(id: string): Promise<Record<string, unknown>> {
    return this.request(`/schedule/${id}`);
  }

  async calculateCpm(scheduleId: string): Promise<Record<string, unknown>> {
    return this.request(`/schedule/${scheduleId}/cpm`, { method: 'POST' });
  }

  // ═══════════════════════════════════════════════════════════
  // ERP: Cost Model (EVM)
  // ═══════════════════════════════════════════════════════════

  async getEvm(projectId: string): Promise<Record<string, unknown>> {
    return this.request(`/costmodel/${projectId}/evm`);
  }

  async listEvmSnapshots(projectId: string): Promise<Array<Record<string, unknown>>> {
    return this.request(`/costmodel/${projectId}/snapshots`);
  }

  // ═══════════════════════════════════════════════════════════
  // Quantity Norms (УСН / consumption norms) — mirrors /prices
  // ═══════════════════════════════════════════════════════════

  async listQuantityNorms(params?: {
    sek_group?: string;
    search?: string;
    source?: string;
    only_mine?: boolean;
    limit?: number;
    offset?: number;
  }): Promise<{ items: QuantityNorm[]; total: number }> {
    const qs = new URLSearchParams();
    if (params?.sek_group) qs.set('sek_group', params.sek_group);
    if (params?.search) qs.set('search', params.search);
    if (params?.source) qs.set('source', params.source);
    if (params?.only_mine) qs.set('only_mine', 'true');
    if (params?.limit != null) qs.set('limit', String(params.limit));
    if (params?.offset != null) qs.set('offset', String(params.offset));
    const query = qs.toString();
    return this.request(`/quantities/norms${query ? `?${query}` : ''}`);
  }

  async createQuantityNorm(norm: Omit<QuantityNorm, 'id' | 'user_id' | 'created_at' | 'updated_at'>): Promise<{ id: string }> {
    return this.request('/quantities/norms', { method: 'POST', body: JSON.stringify(norm) });
  }

  async updateQuantityNorm(id: string, norm: Omit<QuantityNorm, 'id' | 'user_id' | 'created_at' | 'updated_at'>): Promise<void> {
    await this.request(`/quantities/norms/${id}`, { method: 'PUT', body: JSON.stringify(norm) });
  }

  async deleteQuantityNorm(id: string): Promise<void> {
    await this.request(`/quantities/norms/${id}`, { method: 'DELETE' });
  }

  async bulkImportQuantityNorms(norms: QuantityNormInput[], onDuplicate: 'skip' | 'replace' = 'skip'): Promise<{ created: number; updated: number }> {
    return this.request('/quantities/bulk-import', {
      method: 'POST',
      body: JSON.stringify({ norms, on_duplicate: onDuplicate }),
    });
  }

  async listProjectDistributions(): Promise<ProjectDistribution[]> {
    return this.request('/quantities/distributions');
  }

  async upsertProjectDistribution(dist: Omit<ProjectDistribution, 'id'>): Promise<void> {
    await this.request('/quantities/distributions', { method: 'POST', body: JSON.stringify(dist) });
  }

  async deleteProjectDistribution(id: string): Promise<void> {
    await this.request(`/quantities/distributions/${id}`, { method: 'DELETE' });
  }

  async listQuantitySources(): Promise<QuantitySource[]> {
    return this.request('/quantities/sources');
  }

  async createQuantitySource(src: { site_name: string; base_url: string; description?: string; parser_template: string }): Promise<void> {
    await this.request('/quantities/sources', { method: 'POST', body: JSON.stringify(src) });
  }

  async deleteQuantitySource(id: string): Promise<void> {
    await this.request(`/quantities/sources/${id}`, { method: 'DELETE' });
  }

  async listQuantityRuns(): Promise<QuantityRun[]> {
    return this.request('/quantities/runs');
  }

  async triggerQuantityScrape(sourceIds: string[] = []): Promise<{ job_id: string }> {
    return this.request('/quantities/scrape', {
      method: 'POST',
      body: JSON.stringify({ source_ids: sourceIds }),
    });
  }
}

// ── Quantity types (kept local — no separate types/ file needed yet) ──
export interface QuantityMaterial {
  name: string;
  qty: number;
  unit: string;
}

export interface QuantityNorm {
  id?: string;
  sek_code: string;
  description_bg: string;
  work_unit: string;
  labor_qualified_h: number;
  labor_helper_h: number;
  labor_trade?: string | null;
  materials: QuantityMaterial[] | Record<string, unknown>;
  machinery: QuantityMaterial[] | Record<string, unknown>;
  source: string;
  source_url?: string | null;
  confidence: number;
  user_id?: string | null;
  created_at?: string;
  updated_at?: string;
}

export type QuantityNormInput = Omit<QuantityNorm, 'id' | 'user_id' | 'created_at' | 'updated_at'>;

export interface ProjectDistribution {
  id?: string;
  building_type: string;
  metric_key: string;
  metric_label_bg: string;
  unit: string;
  min_value?: number | null;
  max_value?: number | null;
  median_value: number;
  sample_size: number;
  source?: string | null;
  notes?: string | null;
}

export interface QuantitySource {
  id: string;
  site_name: string;
  base_url: string;
  description?: string | null;
  parser_template: string;
  is_builtin: boolean;
  enabled: boolean;
  last_run_at?: string | null;
  last_success?: boolean | null;
  last_norms_count?: number | null;
}

export interface QuantityRun {
  id: string;
  status: string;
  started_at: string;
  completed_at?: string | null;
  total_sources: number;
  successful_sources: number;
  failed_sources: number;
  norms_created: number;
  norms_updated: number;
  elapsed_ms?: number | null;
  notes?: Record<string, unknown> | null;
}

export const api = new ApiClient();
export { ApiError };
