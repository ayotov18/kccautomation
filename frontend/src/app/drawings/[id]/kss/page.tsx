'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import KssAuditTrail from '@/components/kss/KssAuditTrail';
import { KssSectionAccordion } from '@/components/kss/KssSectionAccordion';
import { SuggestionsReviewer } from '@/components/kss/SuggestionsReviewer';
import { Breadcrumbs } from '@/components/layout/Breadcrumbs';
import { Skeleton, SkeletonCard, SkeletonRow } from '@/components/ui/Skeleton';
import { WidgetCarousel, type WidgetSlide } from '@/components/ui/WidgetCarousel';
import { api } from '@/lib/api';
import { DEFAULT_PRICING, type PricingDefaults } from '@/types/config';
import type { KssCorrectionItem, KssSuggestion } from '@/types';

interface KssItem {
  /** kss_line_items.id — required for deterministic save. */
  id?: string;
  item_no: number;
  sek_code: string;
  description: string;
  unit: string;
  quantity: number;
  labor_price: number;
  material_price: number;
  mechanization_price: number;
  overhead_price: number;
  total_price: number;
  // Edit tracking — snapshot of values at load time
  edited?: boolean;
  original_sek_code?: string;
  original_description?: string;
  original_quantity?: number;
  original_unit?: string;
  original_labor_price?: number;
  original_material_price?: number;
}

interface KssSection {
  number: string;
  title_bg: string;
  sek_group: string;
  items: KssItem[];
  section_total_bgn: number;
}

export default function KssReportPage() {
  const params = useParams();
  const router = useRouter();
  const drawingId = params.id as string;

  const [report, setReport] = useState<Record<string, unknown> | null>(null);
  const [sections, setSections] = useState<KssSection[]>([]);
  const [loading, setLoading] = useState(true);
  /** True during post-save refetch — swaps table/summary with skeletons. */
  const [refreshing, setRefreshing] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const [showAudit, setShowAudit] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [suggestions, setSuggestions] = useState<KssSuggestion[]>([]);
  const [addingToSection, setAddingToSection] = useState<string | null>(null);
  const [filename, setFilename] = useState<string | null>(null);
  /** User's configured overhead/VAT defaults — drives the totals panel. */
  const [pricing, setPricing] = useState<PricingDefaults>(DEFAULT_PRICING);

  useEffect(() => {
    api.getPricingDefaults().then(p => { if (p) setPricing(p); }).catch(() => {});
  }, []);
  /**
   * Staged decisions from the AI Suggestions modal. Accept / Reject actions
   * don't hit the backend until the user clicks "Save Corrections". Source of
   * truth for "unsaved changes" along with cell edits.
   */
  const [pendingAccepts, setPendingAccepts] = useState<Set<string>>(new Set());
  const [pendingRejects, setPendingRejects] = useState<Set<string>>(new Set());

  /** Open/closed state per SEK group, persisted per-drawing. */
  const [openGroups, setOpenGroups] = useState<Set<string>>(new Set());
  const sectionStorageKey = `kcc_kss_open_sections:${drawingId}`;

  useEffect(() => {
    try {
      const raw = window.localStorage.getItem(sectionStorageKey);
      if (raw) setOpenGroups(new Set(JSON.parse(raw) as string[]));
    } catch { /* */ }
  }, [sectionStorageKey]);

  const persistOpenGroups = (groups: Set<string>) => {
    try {
      window.localStorage.setItem(sectionStorageKey, JSON.stringify([...groups]));
    } catch { /* */ }
  };

  const toggleGroup = (group: string) => {
    setOpenGroups(prev => {
      const next = new Set(prev);
      if (next.has(group)) next.delete(group);
      else next.add(group);
      persistOpenGroups(next);
      return next;
    });
  };

  const expandAll = () => {
    const next = new Set(sections.map(s => s.sek_group));
    persistOpenGroups(next);
    setOpenGroups(next);
  };
  const collapseAll = () => {
    persistOpenGroups(new Set());
    setOpenGroups(new Set());
  };

  useEffect(() => {
    api.getDrawing(drawingId).then(d => setFilename(d.filename)).catch(() => {});
  }, [drawingId]);

  useEffect(() => {
    document.title = filename ? `${filename} · КСС · KCC` : 'КСС · KCC';
  }, [filename]);

  const fetchKss = useCallback(async () => {
    try {
      const data = await api.getKssData(drawingId);
      setReport(data);
      const reportData = data.report as Record<string, unknown> | undefined;
      if (reportData?.sections) {
        const secs = (reportData.sections as KssSection[]).map(s => ({
          ...s,
          items: s.items.map(item => ({
            ...item,
            edited: false,
            original_sek_code: item.sek_code,
            original_description: item.description,
            original_quantity: item.quantity,
            original_unit: item.unit,
            original_labor_price: item.labor_price,
            original_material_price: item.material_price,
          })),
        }));
        setSections(secs);
      }
      // Load suggestions (low-confidence AI items)
      if (data.suggestions && Array.isArray(data.suggestions)) {
        setSuggestions(data.suggestions as KssSuggestion[]);
      }

      // Default-open the first section if nothing persisted.
      setOpenGroups(prev => {
        if (prev.size > 0) return prev;
        const first = (reportData?.sections as KssSection[] | undefined)?.[0]?.sek_group;
        if (!first) return prev;
        const next = new Set<string>([first]);
        persistOpenGroups(next);
        return next;
      });
    } catch { /* KSS not generated yet */ }
    setLoading(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [drawingId]);

  useEffect(() => { fetchKss(); }, [fetchKss]);

  const handleEdit = (sectionIdx: number, itemIdx: number, field: string, value: string | number) => {
    setSections(prev => prev.map((sec, si) => {
      if (si !== sectionIdx) return sec;
      return {
        ...sec,
        items: sec.items.map((item, ii) => {
          if (ii !== itemIdx) return item;
          return { ...item, [field]: value, edited: true };
        }),
      };
    }));
  };

  const stageAccept = (id: string) => {
    setPendingAccepts(prev => { const n = new Set(prev); n.add(id); return n; });
    setPendingRejects(prev => { const n = new Set(prev); n.delete(id); return n; });
  };
  const stageReject = (id: string) => {
    setPendingRejects(prev => { const n = new Set(prev); n.add(id); return n; });
    setPendingAccepts(prev => { const n = new Set(prev); n.delete(id); return n; });
  };
  const undoStage = (id: string) => {
    setPendingAccepts(prev => { const n = new Set(prev); n.delete(id); return n; });
    setPendingRejects(prev => { const n = new Set(prev); n.delete(id); return n; });
  };

  const handleSave = async () => {
    const editedItems: KssCorrectionItem[] = [];
    for (const sec of sections) {
      for (const item of sec.items) {
        if (!item.edited) continue;
        const sekChanged = item.sek_code !== item.original_sek_code;
        const descChanged = item.description !== item.original_description;
        const qtyChanged = item.quantity !== item.original_quantity;
        const unitChanged = item.unit !== item.original_unit;
        const labChanged = item.labor_price !== item.original_labor_price;
        const matChanged = item.material_price !== item.original_material_price;

        editedItems.push({
          item_id: item.id,
          original_sek_code: item.original_sek_code,
          original_description: item.original_description,
          original_quantity: item.original_quantity,
          original_unit: item.original_unit,
          corrected_sek_code: sekChanged ? item.sek_code : undefined,
          corrected_description: descChanged ? item.description : undefined,
          corrected_quantity: qtyChanged ? item.quantity : undefined,
          corrected_unit: unitChanged ? item.unit : undefined,
          corrected_labor_price: labChanged ? item.labor_price : undefined,
          corrected_material_price: matChanged ? item.material_price : undefined,
          correction_type: sekChanged ? 'sek_code'
            : qtyChanged ? 'quantity'
            : descChanged ? 'description'
            : (labChanged || matChanged) ? 'price'
            : 'unit',
        });
      }
    }

    const accepts = [...pendingAccepts];
    const rejects = [...pendingRejects];

    if (editedItems.length === 0 && accepts.length === 0 && rejects.length === 0) return;
    setSaving(true);
    try {
      // Flush suggestions first — each is a deterministic UPDATE on an
      // existing kss_line_items row, no AI involvement. These run as the
      // original AI values were stored; we only flip the status flag.
      for (const id of accepts) {
        try { await api.acceptSuggestion(drawingId, id); }
        catch (err) { console.warn('accept failed', id, err); }
      }
      for (const id of rejects) {
        try { await api.rejectSuggestion(drawingId, id); }
        catch (err) { console.warn('reject failed', id, err); }
      }

      // Then flush cell-edit corrections.
      let corrCount = 0;
      if (editedItems.length > 0) {
        const result = await api.submitCorrections(drawingId, editedItems);
        corrCount = result.corrections_saved;
      }

      const parts: string[] = [];
      if (corrCount > 0) parts.push(`${corrCount} edit${corrCount === 1 ? '' : 's'}`);
      if (accepts.length > 0) parts.push(`${accepts.length} accepted`);
      if (rejects.length > 0) parts.push(`${rejects.length} rejected`);
      setSaveMsg(`Saved ${parts.join(', ')} — report updated`);

      setPendingAccepts(new Set());
      setPendingRejects(new Set());

      // Refetch with visible skeletons so the user sees the transition.
      // Also small floor to keep the skeleton on-screen long enough to read
      // in case the refresh returns in <100 ms.
      setRefreshing(true);
      await Promise.all([
        fetchKss(),
        new Promise(res => setTimeout(res, 400)),
      ]);
      setRefreshing(false);
    } catch {
      setSaveMsg('Failed to save corrections');
    }
    setSaving(false);
  };

  const editCount = sections.reduce((acc, sec) => acc + sec.items.filter(i => i.edited).length, 0);
  const totalUnsaved = editCount + pendingAccepts.size + pendingRejects.size;

  const handleDownloadExcel = async () => {
    try {
      const blob = await api.downloadKssExcel(drawingId);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url; a.download = `kss-${drawingId}.xlsx`; a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setSaveMsg(e instanceof Error ? e.message : 'Excel download failed');
    }
  };

  const handleDownloadAnalysis = async () => {
    try {
      const blob = await api.downloadAnalysisJson(drawingId);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url; a.download = `deep-analysis-${drawingId}.json`; a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setSaveMsg(e instanceof Error ? e.message : 'Deep analysis download failed');
    }
  };

  if (loading) {
    return (
      <div className="oe-fade-in">
        <div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
          {/* Header skeleton */}
          <div className="flex items-center justify-between">
            <div className="min-w-0 space-y-3">
              <Skeleton className="h-3 w-56" />
              <Skeleton className="h-7 w-96" />
              <Skeleton className="h-3 w-48" />
            </div>
            <div className="flex items-center gap-3">
              <Skeleton className="h-9 w-36" />
              <Skeleton className="h-9 w-20" />
              <Skeleton className="h-9 w-28" />
            </div>
          </div>
          {/* Summary tiles skeleton */}
          <div className="grid grid-cols-4 gap-4">
            <SkeletonCard /> <SkeletonCard /> <SkeletonCard /> <SkeletonCard />
          </div>
          {/* Table skeleton */}
          <div className="oe-card overflow-hidden">
            <table className="w-full text-sm">
              <tbody>
                {Array.from({ length: 10 }).map((_, i) => (
                  <SkeletonRow key={i} cols={[30, '100%', 40, 50, 70, 70, 80]} />
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    );
  }

  if (!report || sections.length === 0) {
    return (
      <div className="oe-fade-in">
<div className="max-w-6xl mx-auto px-6 py-12 text-center">
          <p className="text-content-tertiary mb-4">KSS report not generated yet.</p>
          <button onClick={() => router.push(`/drawings/${drawingId}`)} className="px-4 py-2 bg-surface-tertiary rounded text-sm">
            Back to Drawing
          </button>
        </div>
      </div>
    );
  }

  const subtotal = report.subtotal_lv as number ?? 0;

  return (
    <div className="oe-fade-in">
<div className="max-w-6xl mx-auto px-6 py-8 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="min-w-0">
            <Breadcrumbs
              items={[
                { label: 'Drawings', href: '/drawings' },
                { label: filename ?? '…', href: `/drawings/${drawingId}` },
                { label: 'КСС' },
              ]}
            />
            <h1 className="text-2xl font-bold mt-2">
              Количествено-Стойностна Сметка
              {(report.ai_enhanced as boolean) && (
                <span className="ml-2 px-2 py-1 bg-sky-900/40 text-sky-300 rounded text-xs font-medium align-middle">AI-enhanced</span>
              )}
            </h1>
            <p className="text-sm text-content-tertiary mt-1">Generated: {new Date(report.generated_at as string).toLocaleString('bg-BG')}</p>
          </div>
          <div className="flex items-center gap-3">
            {totalUnsaved > 0 && (
              <span className="text-xs text-sky-300">
                {totalUnsaved} unsaved{' '}
                {pendingAccepts.size > 0 || pendingRejects.size > 0
                  ? `(${editCount} edit${editCount === 1 ? '' : 's'}${
                      pendingAccepts.size ? `, ${pendingAccepts.size} accepted` : ''
                    }${pendingRejects.size ? `, ${pendingRejects.size} rejected` : ''})`
                  : ''}
              </span>
            )}
            {suggestions.length > 0 && (
              <button onClick={() => setShowSuggestions(true)} className="oe-btn-secondary">
                AI Предложения
                <span className="text-[10px] leading-none px-1.5 py-0.5 rounded-full bg-sky-500/20 text-sky-300 font-semibold">
                  {suggestions.length}
                </span>
              </button>
            )}
            <button
              onClick={handleSave}
              disabled={totalUnsaved === 0 || saving}
              className="oe-btn-primary"
            >
              {saving && (
                <svg
                  className="w-3.5 h-3.5 animate-spin"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
                  <circle cx="12" cy="12" r="9" strokeOpacity="0.25" />
                  <path d="M21 12a9 9 0 0 1-9 9" />
                </svg>
              )}
              {saving ? 'Saving…' : 'Save Corrections'}
            </button>
            <button onClick={handleDownloadExcel} className="oe-btn-secondary">Excel</button>
            <button onClick={handleDownloadAnalysis} className="oe-btn-ghost">Deep Analysis</button>
            <button onClick={() => setShowAudit(true)} className="oe-btn-ghost">Audit Trail</button>
          </div>
        </div>

        {saveMsg && (
          <div className={`text-xs px-3 py-2 rounded ${saveMsg.includes('Failed') ? 'bg-red-900/30 text-red-400' : 'bg-emerald-900/30 text-emerald-400'}`}>
            {saveMsg}
          </div>
        )}

        {/* AI-heavy banner: when >50% of rows are ai_inferred, tell the user
            to review every row before submission. */}
        {(() => {
          const suggCount = suggestions.length;
          const itemCount = (report?.item_count as number) ?? 0;
          if (itemCount === 0 || suggCount === 0) return null;
          const ratio = suggCount / itemCount;
          if (ratio < 0.5) return null;
          return (
            <div className="px-4 py-3 rounded-2xl border border-amber-400/30 bg-amber-500/10 text-amber-100 text-sm flex items-start gap-3">
              <span className="text-lg leading-none">⚠︎</span>
              <div className="flex-1">
                <div className="font-semibold">
                  Този КСС е генериран предимно от AI без геометрични данни.
                </div>
                <div className="text-xs text-amber-200/90 mt-0.5">
                  {suggCount} от {itemCount} позиции изискват ръчен преглед (увереност &lt; 0.7).
                  Моля, отворете „Предложения“ и прегледайте всяка, преди подаване в обществена поръчка.
                </div>
              </div>
              <button
                onClick={() => setShowSuggestions(true)}
                className="oe-btn-secondary oe-btn-sm"
              >
                Преглед
              </button>
            </div>
          );
        })()}

        {/* Summary Bar */}
        {refreshing ? (
          <div className="grid grid-cols-4 gap-4">
            <SkeletonCard /> <SkeletonCard /> <SkeletonCard /> <SkeletonCard />
          </div>
        ) : (
        <div className="grid grid-cols-4 gap-4">
          <div className="oe-card p-4 text-center">
            <div className="text-2xl font-bold">{report.item_count as number}</div>
            <div className="text-xs text-content-tertiary">Позиции</div>
          </div>
          <div className="oe-card p-4 text-center">
            <div className="text-2xl font-bold text-emerald-400">{subtotal.toFixed(2)}</div>
            <div className="text-xs text-content-tertiary">Общо СМР (€)</div>
          </div>
          <div className="oe-card p-4 text-center">
            <div className="text-2xl font-bold text-content-secondary">{(subtotal * 0.58).toFixed(2)}</div>
            <div className="text-xs text-content-tertiary">Надбавки (58%)</div>
          </div>
          <div className="oe-card p-4 text-center">
            <div className="text-2xl font-bold text-sky-300">{(subtotal * 1.58 * 1.20).toFixed(2)}</div>
            <div className="text-xs text-content-tertiary">Общо с ДДС (€)</div>
          </div>
        </div>
        )}

        {/* Accordion controls */}
        {!refreshing && sections.length > 0 && (
          <div className="flex items-center justify-between text-xs">
            <div className="text-content-tertiary">
              {sections.length} секции · {sections.reduce((s, x) => s + x.items.length, 0)} позиции
            </div>
            <div className="flex gap-1">
              <button onClick={expandAll} className="oe-btn-ghost oe-btn-sm">Разгъни всички</button>
              <button onClick={collapseAll} className="oe-btn-ghost oe-btn-sm">Сгъни всички</button>
            </div>
          </div>
        )}

        {/* Paginated widget carousel: swipe between sections list and totals */}
        {refreshing ? (
          <div className="oe-card p-4 space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-11 w-full rounded-md" />
            ))}
            <div className="text-center text-xs text-content-tertiary py-3">
              Applying your corrections to the report…
            </div>
          </div>
        ) : (
          <WidgetCarousel
            storageKey={`kcc_kss_widget:${drawingId}`}
            slides={[
              {
                id: 'sections',
                label: `Позиции · ${sections.length} секции`,
                content: (
                  <ScrollableSections
                    sections={sections}
                    openGroups={openGroups}
                    addingToSection={addingToSection}
                    onToggleGroup={toggleGroup}
                    onToggleAdd={(group) =>
                      setAddingToSection(addingToSection === group ? null : group)
                    }
                    onEdit={handleEdit}
                    onItemAdded={() => { setAddingToSection(null); fetchKss(); }}
                    drawingId={drawingId}
                  />
                ),
              },
              {
                id: 'totals',
                label: 'Обща стойност',
                content: (() => {
                  // Canonical cost ladder — prefer the persisted values so the
                  // UI matches the audit trail byte-for-byte. Fall back to
                  // on-the-fly computation only if the older columns are null.
                  const ladder = (report?.cost_ladder as {
                    smr_subtotal?: number | null;
                    contingency?: number | null;
                    delivery_storage?: number | null;
                    profit?: number | null;
                    pre_vat_total?: number | null;
                    vat?: number | null;
                    final_total?: number | null;
                  } | undefined) ?? {};
                  const smr = ladder.smr_subtotal ?? subtotal;
                  const contingency = ladder.contingency ?? (subtotal * (pricing.contingency_pct / 100));
                  const delivery = ladder.delivery_storage ?? (subtotal * (pricing.dr_materials_pct / 100));
                  const profit = ladder.profit ?? (subtotal * (pricing.profit_pct / 100));
                  const beforeVat = ladder.pre_vat_total ?? (smr + contingency + delivery + profit);
                  const vat = ladder.vat ?? (beforeVat * (pricing.vat_rate_pct / 100));
                  const grand = ladder.final_total ?? (beforeVat + vat);
                  return (
                    <div className="px-5 py-4 space-y-1 text-sm">
                      <TotalRow label="ОБЩО СМР" value={smr} emphasis />
                      <TotalRow label={`Непредвидени разходи ${pricing.contingency_pct}%`} value={contingency} dim />
                      <TotalRow label={`Доставно-складови разходи ${pricing.dr_materials_pct}%`} value={delivery} dim />
                      <TotalRow label={`Печалба ${pricing.profit_pct}%`} value={profit} dim />
                      <div className="h-px bg-border-light my-2" />
                      <TotalRow label="ОБЩО ЗА ОБЕКТА" value={beforeVat} emphasis />
                      <TotalRow label={`ДДС ${pricing.vat_rate_pct}%`} value={vat} dim />
                      <TotalRow label="ОБЩО С ДДС" value={grand} grand />
                    </div>
                  );
                })(),
              },
            ] satisfies WidgetSlide[]}
          />
        )}
      </div>

      {/* AI Suggestions Reviewer (single-card stepper) */}
      {showSuggestions && suggestions.length > 0 && (
        <SuggestionsReviewer
          suggestions={suggestions}
          pendingAccepts={pendingAccepts}
          pendingRejects={pendingRejects}
          onAccept={stageAccept}
          onReject={stageReject}
          onUndo={undoStage}
          onCommit={handleSave}
          onClose={() => setShowSuggestions(false)}
        />
      )}

      {/* Audit Trail Overlay */}
      {showAudit && (
        <KssAuditTrail drawingId={drawingId} onClose={() => setShowAudit(false)} />
      )}
    </div>
  );
}

/**
 * Fixed-height scroll area around the accordion list. Height is set in rem so
 * it shows ~5–6 collapsed rows regardless of font scaling. When a section
 * opens, its content scrolls into view automatically.
 */
function ScrollableSections({
  sections,
  openGroups,
  addingToSection,
  onToggleGroup,
  onToggleAdd,
  onEdit,
  onItemAdded,
  drawingId,
}: {
  sections: KssSection[];
  openGroups: Set<string>;
  addingToSection: string | null;
  onToggleGroup: (group: string) => void;
  onToggleAdd: (group: string) => void;
  onEdit: (si: number, ii: number, field: string, value: string | number) => void;
  onItemAdded: () => void;
  drawingId: string;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const rowRefs = useRef<Record<string, HTMLDivElement | null>>({});

  // Scroll a newly-opened section into view — keep its header visible.
  useEffect(() => {
    const lastOpened = [...openGroups].pop();
    if (!lastOpened) return;
    const row = rowRefs.current[lastOpened];
    const container = containerRef.current;
    if (!row || !container) return;
    // Give the accordion animation a tick to start measuring
    setTimeout(() => {
      const rowTop = row.offsetTop;
      container.scrollTo({ top: rowTop - 4, behavior: 'smooth' });
    }, 40);
  }, [openGroups]);

  return (
    <div
      ref={containerRef}
      className="overflow-y-auto kss-scroll-area"
      // ~5 rows @ 40px + padding. Stays compact regardless of viewport.
      style={{ maxHeight: '240px' }}
    >
      {sections.map((section, si) => (
        <div
          key={section.sek_group}
          ref={(el) => { rowRefs.current[section.sek_group] = el; }}
        >
          <KssSectionAccordion
            number={section.number}
            title={section.title_bg}
            itemCount={section.items.length}
            total={section.section_total_bgn}
            isOpen={openGroups.has(section.sek_group)}
            onToggle={() => onToggleGroup(section.sek_group)}
          >
            <SectionItemsTable
              section={section}
              sectionIdx={si}
              onEdit={onEdit}
              drawingId={drawingId}
              isAdding={addingToSection === section.sek_group}
              onToggleAdd={() => onToggleAdd(section.sek_group)}
              onItemAdded={onItemAdded}
            />
          </KssSectionAccordion>
        </div>
      ))}
    </div>
  );
}

/** Items table — rendered inside the accordion panel for each section. */
function SectionItemsTable({ section, sectionIdx, onEdit, drawingId, isAdding, onToggleAdd, onItemAdded }: {
  section: KssSection; sectionIdx: number;
  onEdit: (si: number, ii: number, field: string, value: string | number) => void;
  drawingId: string; isAdding: boolean; onToggleAdd: () => void; onItemAdded: () => void;
}) {
  const [newItem, setNewItem] = useState({ sek_code: '', description: '', unit: 'М2', quantity: 0, unit_price_lv: 0 });
  const [addingSaving, setAddingSaving] = useState(false);

  const handleAddSave = async () => {
    if (!newItem.description || newItem.quantity <= 0) return;
    setAddingSaving(true);
    try {
      await api.addKssItem(drawingId, { ...newItem, sek_code: newItem.sek_code || `${section.sek_group}.999` });
      setNewItem({ sek_code: '', description: '', unit: 'М2', quantity: 0, unit_price_lv: 0 });
      onItemAdded();
    } catch { /* */ }
    setAddingSaving(false);
  };

  if (section.items.length === 0 && !isAdding) {
    return <p className="text-xs text-content-tertiary italic py-2">Няма позиции.</p>;
  }

  return (
    <div className="overflow-hidden rounded-lg border border-border-light/60">
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-content-tertiary border-b border-border-light bg-surface-secondary/40 text-[10px] uppercase tracking-wider">
            <th className="px-3 py-2 w-14">№</th>
            <th className="px-3 py-2">Наименование</th>
            <th className="px-3 py-2 w-14">М-ка</th>
            <th className="px-3 py-2 w-20 text-right">К-во</th>
            <th className="px-3 py-2 w-24 text-right">Материали</th>
            <th className="px-3 py-2 w-24 text-right">Труд</th>
            <th className="px-3 py-2 w-28 text-right">Общо</th>
          </tr>
        </thead>
        <tbody>
          {section.items.map((item, ii) => (
            <tr key={ii} className={`border-b border-border-light/30 last:border-b-0 hover:bg-surface-secondary/30 ${item.edited ? 'bg-sky-900/10' : ''}`}>
              <td className="px-3 py-1.5 text-content-tertiary text-xs font-mono">{section.number}.{item.item_no}</td>
              <td className="px-3 py-1.5">
                <EditableCell value={item.description} onChange={v => onEdit(sectionIdx, ii, 'description', v)} edited={!!item.edited && item.description !== item.original_description} />
              </td>
              <td className="px-3 py-1.5 text-content-secondary text-xs">{item.unit}</td>
              <td className="px-3 py-1.5 text-right">
                <EditableCell value={String(item.quantity)} onChange={v => onEdit(sectionIdx, ii, 'quantity', parseFloat(v) || 0)} edited={!!item.edited && item.quantity !== item.original_quantity} type="number" className="text-right font-mono" />
              </td>
              <td className="px-3 py-1.5 text-right text-content-secondary text-xs font-mono">
                {item.material_price > 0 ? (item.material_price * item.quantity).toFixed(2) : '—'}
              </td>
              <td className="px-3 py-1.5 text-right text-content-secondary text-xs font-mono">
                {item.labor_price > 0 ? (item.labor_price * item.quantity).toFixed(2) : '—'}
              </td>
              <td className="px-3 py-1.5 text-right font-medium font-mono">
                {item.total_price > 0 ? item.total_price.toFixed(2) : '—'}
              </td>
            </tr>
          ))}
          {isAdding && (
            <tr className="border-b border-sky-500/30 bg-sky-900/10">
              <td className="px-3 py-1.5 text-sky-300 text-xs">+</td>
              <td className="px-3 py-1.5">
                <input value={newItem.description} onChange={e => setNewItem(p => ({ ...p, description: e.target.value }))}
                  placeholder="Доставка и монтаж на..." className="w-full bg-transparent border-b border-sky-500/40 text-sm outline-none px-1 py-0.5" />
              </td>
              <td className="px-3 py-1.5">
                <select value={newItem.unit} onChange={e => setNewItem(p => ({ ...p, unit: e.target.value }))}
                  className="bg-surface-tertiary text-xs rounded px-1 py-0.5 border border-sky-500/30">
                  <option>м2</option><option>м3</option><option>м</option><option>м.л.</option>
                  <option>бр</option><option>кг</option><option>тон</option><option>компл</option>
                </select>
              </td>
              <td className="px-3 py-1.5">
                <input type="number" value={newItem.quantity || ''} onChange={e => setNewItem(p => ({ ...p, quantity: parseFloat(e.target.value) || 0 }))}
                  placeholder="0" className="w-full bg-transparent border-b border-sky-500/40 text-sm text-right outline-none px-1 py-0.5 font-mono" />
              </td>
              <td className="px-3 py-1.5"></td>
              <td className="px-3 py-1.5"></td>
              <td className="px-3 py-1.5 text-right">
                <button onClick={handleAddSave} disabled={addingSaving || !newItem.description}
                  className="text-xs px-2 py-1 bg-sky-500/90 hover:bg-sky-400 disabled:bg-gray-700 disabled:text-gray-500 text-gray-900 rounded">
                  {addingSaving ? '…' : 'Добави'}
                </button>
              </td>
            </tr>
          )}
        </tbody>
        <tfoot>
          <tr className="bg-surface-secondary/30 border-t border-border-light">
            <td colSpan={4} className="px-3 py-1.5 text-right text-content-tertiary text-xs italic">
              Общо за секция:
            </td>
            <td className="px-3 py-1.5 text-right text-content-tertiary text-xs font-mono">
              {section.items.reduce((s, i) => s + i.material_price * i.quantity, 0).toFixed(2)}
            </td>
            <td className="px-3 py-1.5 text-right text-content-tertiary text-xs font-mono">
              {section.items.reduce((s, i) => s + i.labor_price * i.quantity, 0).toFixed(2)}
            </td>
            <td className="px-3 py-1.5 text-right font-medium text-content-primary font-mono">
              {section.section_total_bgn.toFixed(2)}
            </td>
          </tr>
        </tfoot>
      </table>
      {/* Add-row action — lives inside the expanded panel, out of the header */}
      <div className="mt-2 flex justify-end">
        <button
          onClick={onToggleAdd}
          className="oe-btn-ghost oe-btn-sm"
          title="Добави нова позиция"
        >
          {isAdding ? '✕ Откажи' : '+ Нова позиция'}
        </button>
      </div>
    </div>
  );
}

/** Row for the bottom totals panel. */
function TotalRow({
  label,
  value,
  dim = false,
  emphasis = false,
  grand = false,
}: {
  label: string;
  value: number;
  dim?: boolean;
  emphasis?: boolean;
  grand?: boolean;
}) {
  return (
    <div
      className={`flex items-center justify-between ${
        grand
          ? 'text-base font-bold pt-1 pb-0.5'
          : emphasis
          ? 'font-semibold text-sm py-1'
          : dim
          ? 'text-xs text-content-tertiary py-0.5'
          : 'text-sm py-1'
      }`}
    >
      <span>{label}</span>
      <span
        className={`font-mono ${
          grand ? 'text-sky-300' : emphasis ? 'text-content-primary' : ''
        }`}
      >
        {Number.isFinite(value) ? value.toFixed(2) : '—'} €
      </span>
    </div>
  );
}

/* Old multi-card SuggestionsWidget removed — replaced by SuggestionsReviewer. */

function EditableCell({ value, onChange, edited = false, className = '', type = 'text' }: {
  value: string; onChange: (v: string) => void; edited?: boolean; className?: string; type?: string;
}) {
  const [editing, setEditing] = useState(false);

  if (editing) {
    return (
      <input
        type={type}
        value={value}
        onChange={e => onChange(e.target.value)}
        onBlur={() => setEditing(false)}
        onKeyDown={e => e.key === 'Enter' && setEditing(false)}
        autoFocus
        className={`w-full bg-surface-tertiary border border-sky-500 rounded px-1.5 py-0.5 text-sm focus:outline-none ${className}`}
      />
    );
  }

  return (
    <span
      onClick={() => setEditing(true)}
      className={`cursor-pointer hover:text-sky-200 ${edited ? 'text-sky-200' : ''} ${className}`}
      title="Click to edit"
    >
      {value || '-'}
    </span>
  );
}
