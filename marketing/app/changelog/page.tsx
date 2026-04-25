import type { Metadata } from 'next';
import { Nav } from '@/components/nav';
import { Footer } from '@/components/footer';
import { ScrollProgress } from '@/components/scroll-progress';
import { SubPageShell } from '@/components/sub-page-shell';
import { Eyebrow } from '@/components/eyebrow';
import { TextEffect } from '@/components/ui/text-effect';
import { LiquidGlass } from '@/components/ui/liquid-glass';
import { SpotlightCard } from '@/components/ui/spotlight-card';
import { ProgressiveSeam, AccentGleam } from '@/components/ui/edge-bleed';

export const metadata: Metadata = {
  title: 'Changelog — KCC Automation',
  description:
    'Twenty-one migrations, four phases, every entry shipped because someone got tired of the workaround.',
};

const PHASES = [
  {
    key: 'phase-1',
    label: 'Foundation',
    range: 'mig 001 – 005',
    summary: 'Drawings, jobs, KSS skeleton, file-hash dedupe, nullable joins so partial drawings can still be priced.',
    entries: [
      { v: '001', h: 'Initial schema', b: 'drawings, jobs, kss_line_items, kss_corrections.' },
      { v: '002', h: 'File hash dedupe', b: 'SHA-256 on every upload. Same file twice is one row.' },
      { v: '003', h: 'KSS bootstrap', b: 'Bulgarian СЕК group taxonomy, ОБРАЗЕЦ 9.1 column shape.' },
      { v: '004', h: 'Scrape prices', b: 'First version of scraped_price_rows for supplier pricing.' },
      { v: '005', h: 'Nullable job → drawing', b: 'Allow KSS jobs that operate on partial parses.' },
    ],
  },
  {
    key: 'phase-2',
    label: 'Price intelligence',
    range: 'mig 006 – 010',
    summary: 'Scrape pipeline v2, line-item price columns, DRM rules, KSS reports table.',
    entries: [
      { v: '006', h: 'Scrape pipeline v2', b: 'Worker schedules, partial results, BrightData proxy contract.' },
      { v: '007', h: 'Price LV columns', b: 'Labour, material, mechanisation, overhead split persisted, not computed.' },
      { v: '008', h: 'DRM (Drawing Rule Mapping)', b: 'Per-user rules to override layer→СЕК group decisions.' },
      { v: '009', h: 'Price CRUD', b: 'Editable prices in the UI, with audit on every change.' },
      { v: '010', h: 'KSS reports', b: 'Versioned KSS snapshots; old versions remain queryable.' },
    ],
  },
  {
    key: 'phase-3',
    label: 'Auditability',
    range: 'mig 011 – 015',
    summary: 'AI dual-mode KSS, audit trail, suggestions, ERP foundation, draft status.',
    entries: [
      { v: '011', h: 'AI KSS dual-mode', b: 'Standard pipeline + AI pipeline (research → review → generate).' },
      { v: '012', h: 'KSS audit trail', b: 'kss_audit_trail records every fired rule, source, and Opus reasoning.' },
      { v: '013', h: 'Suggestions', b: 'Low-confidence rows surface to a review widget instead of rolling silently.' },
      { v: '014', h: 'ERP foundation', b: 'erp-core types: money, units, codes. Used by future BOQ + costs work.' },
      { v: '015', h: 'KSS draft status', b: 'Drafts vs finalized — only finalized exports go out.' },
    ],
  },
  {
    key: 'phase-4',
    label: 'Refinement',
    range: 'mig 016 – 021',
    summary: 'Pricing defaults + EUR, phase-4 audit duals, quantity norms, scraper runtime, extraction traceability, the explicit totals ladder.',
    entries: [
      { v: '016', h: 'Pricing defaults + EUR', b: 'Per-org defaults, EUR support, anchor bands per СЕК.' },
      { v: '017', h: 'Phase-4 audit dual code', b: 'Dual-coding of audit entries against a versioned schema.' },
      { v: '018', h: 'Quantity norms', b: 'Per-unit consumption norms (e.g. m³ concrete per m² wall).' },
      { v: '019', h: 'Quantity scraper runtime', b: 'Norms can be researched live when missing.' },
      { v: '020', h: 'Extraction traceability', b: 'Every quantity carries an extraction_method and confidence.' },
      { v: '021', h: 'Explicit totals ladder', b: 'Subtotals at every aggregation level. No more "where did this number come from".' },
    ],
  },
];

export default function ChangelogPage() {
  return (
    <>
      <ScrollProgress />
      <Nav />
      <SubPageShell
        eyebrow="Velocity"
        title="Twenty-one migrations. Every one of them solved a real problem."
        sub="Four phases since the project began: foundation, price intelligence, auditability, refinement. Each entry shipped because somebody on the team got tired of the workaround."
        hero="/assets/gen/page-changelog-hero.png"
      >
        <section className="relative py-32 md:py-40 overflow-hidden">
          <ProgressiveSeam direction="top" height={160} />
          <AccentGleam position={{ left: '20%', bottom: '0%' }} size={900} opacity={0.1} />
          <div className="mx-auto max-w-7xl px-6">
            <div className="grid lg:grid-cols-[1fr_2fr] gap-12">
              {/* Phase nav */}
              <div className="lg:sticky lg:top-28 lg:self-start space-y-3">
                {PHASES.map((p, i) => (
                  <SpotlightCard key={p.key} className="liquid-glass rounded-2xl p-6">
                    <div className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)] mb-2">
                      Phase {String(i + 1).padStart(2, '0')} · {p.range}
                    </div>
                    <h3 className="text-[length:var(--text-lg)] font-semibold tracking-[-0.015em]">
                      {p.label}
                    </h3>
                    <p className="mt-3 text-[13px] leading-[1.6] text-[var(--color-fg-secondary)]">
                      {p.summary}
                    </p>
                  </SpotlightCard>
                ))}
              </div>

              {/* Entries */}
              <div className="space-y-12">
                {PHASES.map((p) => (
                  <div key={p.key}>
                    <Eyebrow className="mb-4 block">{p.label}</Eyebrow>
                    <LiquidGlass intensity="soft" className="rounded-2xl overflow-hidden">
                      <ul className="divide-y divide-white/5">
                        {p.entries.map((e) => (
                          <li key={e.v} className="grid grid-cols-[60px_1fr] gap-6 px-7 py-5 hover:bg-white/[0.02] transition-colors">
                            <span className="font-[family-name:var(--font-mono)] text-[13px] text-[var(--color-amber)] tabular-nums">
                              {e.v}
                            </span>
                            <div>
                              <h4 className="text-[14px] font-medium tracking-[-0.005em]">{e.h}</h4>
                              <p className="mt-1.5 text-[13px] leading-[1.6] text-[var(--color-fg-secondary)]">
                                {e.b}
                              </p>
                            </div>
                          </li>
                        ))}
                      </ul>
                    </LiquidGlass>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>
      </SubPageShell>
      <Footer />
    </>
  );
}
