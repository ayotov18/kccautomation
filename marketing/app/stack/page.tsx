import type { Metadata } from 'next';
import { Nav } from '@/components/nav';
import { Footer } from '@/components/footer';
import { ScrollProgress } from '@/components/scroll-progress';
import { SubPageShell } from '@/components/sub-page-shell';
import { Stack as StackSection } from '@/components/stack';
import { Eyebrow } from '@/components/eyebrow';
import { TextEffect } from '@/components/ui/text-effect';
import { SpotlightCard } from '@/components/ui/spotlight-card';
import { LiquidGlass } from '@/components/ui/liquid-glass';
import { ProgressiveSeam, AccentGleam } from '@/components/ui/edge-bleed';

export const metadata: Metadata = {
  title: 'Stack — KCC Automation',
  description:
    'Rust workspace, 9 crates. Axum + sqlx on the API, BullMQ-style queues on the worker, Postgres 16, Redis 7, Next.js 15. Built for the hot path.',
};

const CRATES = [
  { name: 'kcc-api', role: 'HTTP API', body: 'Axum on Tokio. Auth, uploads, exports, polling endpoints. Runs sqlx::migrate! on boot.' },
  { name: 'kcc-worker', role: 'Background runtime', body: 'Pulls jobs from Redis. Owns the heavy lifting: parsing, KSS generation, AI research.' },
  { name: 'kcc-core', role: 'Domain', body: 'Pure logic. No frameworks. Imported by every other crate.' },
  { name: 'kcc-dxf', role: 'Drawing parser', body: 'nom-based DXF parser, ODA shell-out for DWG, geometry primitives, dimension linking.' },
  { name: 'kcc-report', role: 'Output', body: 'Excel via rust_xlsxwriter, PDF, CSV. Single trait per format.' },
  { name: 'erp-core', role: 'ERP primitives', body: 'Money, units, codes. Designed for shared use across boq/costs/assemblies.' },
  { name: 'erp-boq', role: 'Bill of quantities', body: 'BOQ assembly + traversal. Used by the КСС generator.' },
  { name: 'erp-costs', role: 'Costing', body: 'Labour + material + mechanisation + overhead split. Sanity bands.' },
  { name: 'erp-assemblies', role: 'Composition', body: 'Pre-built assemblies (e.g. 25cm masonry wall) feeding the BOQ.' },
];

const PRINCIPLES = [
  {
    h: 'Pure core, framework edges',
    b: 'kcc-core has no axum, no sqlx, no tokio in its public types. Frameworks live on the rim. Hexagonal-ish.',
  },
  {
    h: 'Compile-time SQL',
    b: 'sqlx prepares every query against the live schema in CI. A typo in a column name is a build failure, not a 500.',
  },
  {
    h: 'Migrations applied on boot',
    b: 'kcc-api runs sqlx::migrate!() at startup. Deploy and migrate are the same operation.',
  },
  {
    h: 'Background work is durable',
    b: 'Redis-backed queues with retries and dead-letter. The worker is restartable mid-job.',
  },
  {
    h: 'Frontend stays thin',
    b: 'Next.js 15 app router; server components where they help. The API is the source of truth, the UI is a view.',
  },
];

export default function StackPage() {
  return (
    <>
      <ScrollProgress />
      <Nav />
      <SubPageShell
        eyebrow="Engineering"
        title="Rust on the hot path. Everything else where it belongs."
        sub="Nine crates in one workspace, Postgres 16 with sqlx-managed migrations, Redis-backed queues, and a Next.js 15 operator UI. Compile-time SQL, hexagonal-ish boundaries, no surprises in production."
        hero="/assets/gen/page-stack-hero.png"
      >
        {/* Crate grid */}
        <section className="relative py-32 md:py-40 overflow-hidden">
          <ProgressiveSeam direction="top" height={160} />
          <AccentGleam position={{ right: '10%', top: '0%' }} size={900} opacity={0.1} />
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">The workspace</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Nine crates, one Cargo workspace.
              </TextEffect>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {CRATES.map((c) => (
                <SpotlightCard key={c.name} className="liquid-glass border-shine rounded-2xl p-6 min-h-[200px]">
                  <div className="flex items-baseline justify-between mb-3">
                    <span className="font-[family-name:var(--font-mono)] text-[13px] tracking-[-0.005em] text-[var(--color-fg)]">
                      {c.name}
                    </span>
                    <span className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.16em] text-[var(--color-amber)]">
                      {c.role}
                    </span>
                  </div>
                  <p className="mt-2 text-[13px] leading-[1.6] text-[var(--color-fg-secondary)]">
                    {c.body}
                  </p>
                </SpotlightCard>
              ))}
            </div>
          </div>
        </section>

        {/* Principles */}
        <section className="relative py-28 md:py-36 overflow-hidden border-t border-white/5">
          <div className="mx-auto max-w-7xl px-6">
            <div className="max-w-2xl mb-14">
              <Eyebrow className="mb-4 block">Design choices</Eyebrow>
              <TextEffect
                as="h2"
                className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                stagger={0.04}
                triggerOnView
              >
                Five rules we don't break.
              </TextEffect>
            </div>
            <LiquidGlass intensity="soft" className="rounded-2xl overflow-hidden">
              <ul className="divide-y divide-white/5">
                {PRINCIPLES.map((p, i) => (
                  <li key={p.h} className="grid grid-cols-1 md:grid-cols-[300px_1fr] px-7 py-6">
                    <span className="font-[family-name:var(--font-mono)] text-[12px] uppercase tracking-[0.1em] flex items-center gap-2">
                      <span className="text-[var(--color-fg-quaternary)] mr-1">
                        {String(i + 1).padStart(2, '0')}
                      </span>
                      {p.h}
                    </span>
                    <span className="mt-1 md:mt-0 text-[13.5px] leading-[1.65] text-[var(--color-fg-secondary)]">
                      {p.b}
                    </span>
                  </li>
                ))}
              </ul>
            </LiquidGlass>
          </div>
        </section>

        {/* Reuse the live stack table component */}
        <StackSection />
      </SubPageShell>
      <Footer />
    </>
  );
}
