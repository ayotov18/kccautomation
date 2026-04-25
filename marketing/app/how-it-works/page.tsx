import type { Metadata } from 'next';
import { Nav } from '@/components/nav';
import { Footer } from '@/components/footer';
import { ScrollProgress } from '@/components/scroll-progress';
import { SubPageShell } from '@/components/sub-page-shell';
import { Pipeline } from '@/components/pipeline';
import { Eyebrow } from '@/components/eyebrow';
import { TextEffect } from '@/components/ui/text-effect';
import { LiquidGlass } from '@/components/ui/liquid-glass';
import { SpotlightCard } from '@/components/ui/spotlight-card';
import { ProgressiveSeam, AccentGleam } from '@/components/ui/edge-bleed';

export const metadata: Metadata = {
  title: 'How it works — KCC Automation',
  description:
    'A four-stage pipeline: upload, parse, price, export. End-to-end median 2m 38s. Walk through every step.',
};

const FLOW = [
  {
    label: 'Upload',
    sub: '< 1s',
    body: 'API validates the file (DXF, DWG, PDF), computes SHA-256, stores at uploads/{drawing_id}/original.{ext}, inserts a drawings row, and enqueues a kcc:jobs job. Frontend polls /api/v1/jobs/{job_id} every 1.5s.',
    stack: ['kcc-api · Axum', 'Postgres · jobs', 'S3 · uploads/'],
  },
  {
    label: 'Parse',
    sub: '~30–90s',
    body: 'kcc-worker pulls the job, runs kcc-dxf parsing (nom for DXF, ODA File Converter for DWG), builds an RTree spatial index, links dimensions to geometry, runs feature extraction, and serializes a complete AnalysisResult to S3 at analysis/{drawing_id}/canonical.json.',
    stack: ['kcc-worker', 'kcc-dxf · nom', 'RTree spatial index'],
  },
  {
    label: 'Price',
    sub: '~30–60s',
    body: 'Quantities mapped to СЕК cost codes. Prices come from one of three sources, in order: scraped_price_rows cache, BrightData supplier scrape, Perplexity sonar-pro live research. Each row stamps its source and confidence.',
    stack: ['BrightData proxy', 'OpenRouter sonar-pro', 'Opus 4.6 generation'],
  },
  {
    label: 'Export',
    sub: '< 1s',
    body: 'Three streaming endpoints. Excel via rust_xlsxwriter matching ОБРАЗЕЦ 9.1. PDF via the kcc-report crate. CSV for everything else. Audit trail attached.',
    stack: ['kcc-report', 'rust_xlsxwriter', 'streaming responses'],
  },
];

export default function HowItWorksPage() {
  return (
    <>
      <ScrollProgress />
      <Nav />
      <SubPageShell
        eyebrow="The pipeline"
        title="From DXF to КСС, four services, one path."
        sub="Upload, parse, price, export — every stage runs in isolation, every state change is auditable, and the median end-to-end is under three minutes."
        hero="/assets/gen/page-pipeline-hero.png"
      >
        {/* Stage table */}
        <section className="relative py-32 md:py-40 overflow-hidden">
          <ProgressiveSeam direction="top" height={180} />
          <AccentGleam position={{ left: '10%', top: '0%' }} size={800} opacity={0.1} />
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">Stage by stage</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Each stage owns its data, its time budget, its failure mode.
              </TextEffect>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
              {FLOW.map((s, i) => (
                <SpotlightCard
                  key={s.label}
                  className="liquid-glass border-shine rounded-2xl p-8 min-h-[280px]"
                >
                  <div className="flex items-baseline justify-between mb-5">
                    <span className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)]">
                      step {String(i + 1).padStart(2, '0')}
                    </span>
                    <span className="font-[family-name:var(--font-mono)] text-[11px] tracking-[0.08em] text-[var(--color-amber)] tabular-nums">
                      {s.sub}
                    </span>
                  </div>
                  <h3 className="text-[length:var(--text-xl)] font-semibold tracking-[-0.02em]">
                    {s.label}
                  </h3>
                  <p className="mt-4 text-[13.5px] leading-[1.65] text-[var(--color-fg-secondary)]">
                    {s.body}
                  </p>
                  <ul className="mt-6 flex flex-wrap gap-x-4 gap-y-1 font-[family-name:var(--font-mono)] text-[11px] text-[var(--color-fg-quaternary)]">
                    {s.stack.map((t) => (
                      <li key={t} className="flex items-center gap-1.5">
                        <span className="inline-block h-1 w-1 rounded-full bg-[var(--color-amber)]/60" />
                        {t}
                      </li>
                    ))}
                  </ul>
                </SpotlightCard>
              ))}
            </div>
          </div>
        </section>

        {/* The interactive pipeline (sticky scroll demo from landing) */}
        <Pipeline />

        {/* SLOs */}
        <section className="relative py-28 md:py-36 overflow-hidden border-t border-white/5">
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-12">
              <Eyebrow className="mb-4 block">Time budgets</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Every stage has a number, and we miss it loudly.
              </TextEffect>
            </div>
            <LiquidGlass intensity="soft" className="rounded-2xl overflow-hidden">
              <ul className="divide-y divide-white/5">
                {[
                  ['Median end-to-end', '2m 38s'],
                  ['p95 worker time', '1m 52s'],
                  ['p95 export Excel', '0.9s'],
                  ['p95 cold-start API boot (incl. migrations)', '4.2s'],
                  ['Job retry policy', '3 attempts, exponential backoff'],
                ].map(([k, v]) => (
                  <li key={k} className="grid grid-cols-1 md:grid-cols-[1fr_220px] px-7 py-5">
                    <span className="text-[14px] text-[var(--color-fg-secondary)]">{k}</span>
                    <span className="font-[family-name:var(--font-mono)] text-[13px] text-[var(--color-amber)] tabular-nums">
                      {v}
                    </span>
                  </li>
                ))}
              </ul>
            </LiquidGlass>
          </div>
        </section>
      </SubPageShell>
      <Footer />
    </>
  );
}
