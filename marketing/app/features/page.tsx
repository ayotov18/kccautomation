import type { Metadata } from 'next';
import { Nav } from '@/components/nav';
import { Footer } from '@/components/footer';
import { ScrollProgress } from '@/components/scroll-progress';
import { SubPageShell } from '@/components/sub-page-shell';
import { Eyebrow } from '@/components/eyebrow';
import { TextEffect } from '@/components/ui/text-effect';
import { SpotlightCard } from '@/components/ui/spotlight-card';
import { LiquidGlass } from '@/components/ui/liquid-glass';
import { ProgressiveSeam, AccentGleam } from '@/components/ui/edge-bleed';
import {
  ScanLine,
  Boxes,
  Compass,
  Hash,
  Network,
  ShieldCheck,
  Coins,
  FileBadge,
  FileSpreadsheet,
  FileCode2,
  Layers,
  Search,
} from 'lucide-react';

export const metadata: Metadata = {
  title: 'Features — KCC Automation',
  description:
    'Every layer parsed, every quantity scored, every price cited. The full feature surface of KCC, grouped by parsing, pricing, exporting and auditing.',
};

const PARSING = [
  { icon: ScanLine, h: 'DXF, DWG, PDF', b: 'kcc-dxf parses DXF natively, DWG via ODA File Converter, and PDFs through layout extraction. Same downstream model regardless of source.' },
  { icon: Layers, h: 'Layer + entity model', b: 'Every layer, block, dimension, annotation kept in a structured AnalysisResult — not a flattened bag of polylines.' },
  { icon: Compass, h: 'Spatial index', b: 'RTree-backed lookups. Dimension-to-geometry links resolved before quantity extraction.' },
  { icon: Hash, h: 'Feature extraction', b: 'Holes, slots, pockets, welds, bolts. Stored on each feature with its own confidence score.' },
  { icon: Network, h: 'Drawing-type detection', b: 'Architectural vs steel-fabrication auto-detected from layer heuristics — different pipelines downstream.' },
  { icon: Boxes, h: 'Per-user S3 storage', b: 'Originals at uploads/{drawing_id}/original.{ext}; analysis snapshots at analysis/{drawing_id}/canonical.json.' },
];

const QUANTITIES = [
  { h: 'Polyline area (Shoelace)', b: 'Closed polylines on layer-mapped surfaces compute area directly. Confidence 0.9.' },
  { h: 'Block count', b: 'Block references mapped to count-units (e.g. doors, windows). Confidence 0.85.' },
  { h: 'Linear length', b: 'Open polylines on linear-unit layers (m, m²/m). Confidence 0.8.' },
  { h: 'Dimension annotation', b: 'When the drawing is dimensioned, dim values feed quantities directly.' },
  { h: 'Manual override (DRM)', b: 'Drawing Rule Mappings let you override layer→СЕК group and quantity method per drawing-type.' },
  { h: 'AI fallback (Opus 4.6)', b: 'When all of the above are below 0.6, Opus reads the drawing and proposes a quantity. Always flagged for review.' },
];

const PRICING = [
  { h: 'BrightData supplier scrape', b: 'Rotating residential proxy hits Bulgarian supplier sites. Outputs deduped by normalized key, stored in scraped_price_rows.' },
  { h: 'Perplexity sonar-pro', b: 'Live market research with citations. Confidence + reasoning persisted alongside the price.' },
  { h: 'User-uploaded CSV', b: 'Bring your own price list. Same schema, same priority rules.' },
  { h: 'Sanity anchor bands', b: 'Per-СЕК bands ("masonry 25cm: 18–33 €/M²"). Out-of-band prices flagged.' },
];

const EXPORTS = [
  { icon: FileSpreadsheet, h: 'Excel — ОБРАЗЕЦ 9.1', b: 'rust_xlsxwriter generates the Bulgarian standard form, ready to hand a client.' },
  { icon: FileBadge, h: 'PDF', b: 'Internal kcc-report crate. Tabular layout, branded header, audit trail at the back.' },
  { icon: FileCode2, h: 'CSV', b: 'For pivot tables, ERP imports, anywhere else.' },
];

const AUDIT = [
  { icon: ShieldCheck, h: 'Confidence on every row', b: '0.0–1.0 score, computed from how the quantity was derived. Below 0.6 always surfaces for review.' },
  { icon: Search, h: 'Full audit trail', b: 'kss_audit_trail logs the rule that fired, the price source, and the Opus reasoning.' },
  { icon: Coins, h: 'Corrections compound', b: 'Edits in the UI feed back into kss_corrections; future drawings inherit them.' },
];

export default function FeaturesPage() {
  return (
    <>
      <ScrollProgress />
      <Nav />
      <SubPageShell
        eyebrow="The full surface"
        title="Every layer parsed, every quantity scored, every price cited."
        sub="Six parsing capabilities, six extraction methods, three pricing sources, three export formats — and a confidence number on every single row."
        hero="/assets/gen/page-features-hero.png"
        variant="features"
      >
        {/* Parsing */}
        <section className="relative py-32 md:py-40 overflow-hidden">
          <ProgressiveSeam direction="top" height={140} />
          <AccentGleam position={{ left: '20%', top: '0%' }} size={800} opacity={0.1} />

          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">01 · Parse</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Reads the drawing the way an estimator does, only faster.
              </TextEffect>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {PARSING.map((it) => (
                <SpotlightCard key={it.h} className="liquid-glass border-shine rounded-2xl p-7 min-h-[200px]">
                  <div className="mb-4 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                    <it.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                  </div>
                  <h3 className="text-[length:var(--text-base)] font-semibold tracking-[-0.015em]">
                    {it.h}
                  </h3>
                  <p className="mt-3 text-[13px] leading-[1.65] text-[var(--color-fg-secondary)]">{it.b}</p>
                </SpotlightCard>
              ))}
            </div>
          </div>
        </section>

        {/* Quantity extraction */}
        <section className="relative py-28 md:py-36 overflow-hidden border-t border-white/5">
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">02 · Quantities</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Six extraction methods, ranked by confidence.
              </TextEffect>
            </div>
            <LiquidGlass intensity="soft" className="rounded-2xl overflow-hidden">
              <ul className="divide-y divide-white/5">
                {QUANTITIES.map((it, i) => (
                  <li key={it.h} className="grid grid-cols-1 md:grid-cols-[260px_1fr] px-7 py-6">
                    <span className="font-[family-name:var(--font-mono)] text-[12px] uppercase tracking-[0.1em] flex items-center gap-2">
                      <span className="inline-block h-1 w-1 rounded-full bg-[var(--color-amber)]/60" />
                      <span className="text-[var(--color-fg-quaternary)] mr-2">
                        {String(i + 1).padStart(2, '0')}
                      </span>
                      {it.h}
                    </span>
                    <span className="mt-1 md:mt-0 text-[13.5px] leading-[1.65] text-[var(--color-fg-secondary)]">
                      {it.b}
                    </span>
                  </li>
                ))}
              </ul>
            </LiquidGlass>
          </div>
        </section>

        {/* Pricing */}
        <section className="relative py-28 md:py-36 overflow-hidden border-t border-white/5">
          <AccentGleam position={{ right: '10%', top: '20%' }} size={700} opacity={0.12} />
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">03 · Price</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Live Bulgarian prices, every row cited.
              </TextEffect>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {PRICING.map((it) => (
                <SpotlightCard key={it.h} className="liquid-glass border-shine rounded-2xl p-7 min-h-[180px]">
                  <h3 className="text-[length:var(--text-base)] font-semibold tracking-[-0.015em]">
                    {it.h}
                  </h3>
                  <p className="mt-3 text-[13px] leading-[1.65] text-[var(--color-fg-secondary)]">{it.b}</p>
                </SpotlightCard>
              ))}
            </div>
          </div>
        </section>

        {/* Exports + Audit */}
        <section className="relative py-28 md:py-36 overflow-hidden border-t border-white/5">
          <div className="mx-auto max-w-7xl px-6 grid md:grid-cols-2 gap-10">
            <div>
              <Eyebrow className="mb-4 block">04 · Export</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-2xl)] leading-[1.04] tracking-[-0.02em]"
                stagger={0.04}
                triggerOnView
              >
                Three formats. One endpoint each. No "please wait while we build your file."
              </TextEffect>
              <div className="mt-8 space-y-4">
                {EXPORTS.map((it) => (
                  <SpotlightCard key={it.h} className="liquid-glass border-shine border-shine-cool rounded-xl p-6">
                    <div className="flex items-start gap-4">
                      <div className="shrink-0 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                        <it.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                      </div>
                      <div>
                        <h3 className="text-[length:var(--text-base)] font-semibold tracking-[-0.015em]">
                          {it.h}
                        </h3>
                        <p className="mt-2 text-[13px] leading-[1.65] text-[var(--color-fg-secondary)]">
                          {it.b}
                        </p>
                      </div>
                    </div>
                  </SpotlightCard>
                ))}
              </div>
            </div>
            <div>
              <Eyebrow className="mb-4 block">05 · Audit</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-2xl)] leading-[1.04] tracking-[-0.02em]"
                stagger={0.04}
                triggerOnView
              >
                Defensible on every row. Every time.
              </TextEffect>
              <div className="mt-8 space-y-4">
                {AUDIT.map((it) => (
                  <SpotlightCard key={it.h} className="liquid-glass border-shine border-shine-cool rounded-xl p-6">
                    <div className="flex items-start gap-4">
                      <div className="shrink-0 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                        <it.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                      </div>
                      <div>
                        <h3 className="text-[length:var(--text-base)] font-semibold tracking-[-0.015em]">
                          {it.h}
                        </h3>
                        <p className="mt-2 text-[13px] leading-[1.65] text-[var(--color-fg-secondary)]">
                          {it.b}
                        </p>
                      </div>
                    </div>
                  </SpotlightCard>
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
