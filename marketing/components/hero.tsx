'use client';

import { motion } from 'motion/react';
import { ArrowRight } from 'lucide-react';
import { Eyebrow } from './eyebrow';

export function Hero() {
  return (
    <section className="relative min-h-[100svh] flex items-center overflow-hidden">
      {/* Video background with PNG fallback / poster */}
      <video
        aria-hidden
        autoPlay
        muted
        loop
        playsInline
        preload="metadata"
        poster="/assets/gen/hero.png"
        className="absolute inset-0 z-0 h-full w-full object-cover"
      >
        <source src="/assets/gen/video-hero.mp4" type="video/mp4" />
      </video>
      {/* Radial darken for copy legibility */}
      <div
        aria-hidden
        className="absolute inset-0 z-[1] bg-gradient-to-r from-[var(--color-bg)] via-[var(--color-bg)]/85 to-transparent"
      />
      <div aria-hidden className="absolute inset-0 z-[2] grid-bg opacity-[0.12] pointer-events-none" />
      <div aria-hidden className="absolute inset-0 z-[3] grain pointer-events-none" />

      <div className="relative z-10 w-full mx-auto max-w-7xl px-6 pt-32 pb-20">
        <motion.div
          initial={{ opacity: 0, y: 14 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.22, 0.61, 0.36, 1] }}
          className="max-w-2xl"
        >
          <Eyebrow className="mb-6 inline-block">
            Construction estimating, on rails
          </Eyebrow>

          <h1 className="text-[clamp(2.25rem,5.5vw,4.75rem)] font-semibold leading-[1.04] tracking-tight text-[var(--color-fg)]">
            From DXF to КСС in under three minutes.
          </h1>

          <p className="mt-6 text-[15px] md:text-[17px] leading-relaxed text-[var(--color-fg-secondary)] max-w-xl">
            Upload the drawing. The pipeline reads every layer, pulls live Bulgarian market prices, and
            returns a priced КСС with an audit trail for every row.
          </p>

          <div className="mt-10 flex flex-wrap items-center gap-4">
            <a
              href="#cta"
              className="group inline-flex h-12 items-center gap-2 rounded-lg bg-[var(--color-amber)] px-6 text-[14px] font-medium text-[var(--color-bg)] transition-all hover:bg-[var(--color-amber-soft)] shadow-[0_0_0_1px_rgba(184,115,51,0.2),0_8px_20px_rgba(184,115,51,0.25)]"
            >
              Request access
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
            </a>
            <a
              href="#pipeline"
              className="group inline-flex h-12 items-center gap-2 px-2 text-[14px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
            >
              See how the pipeline works
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
            </a>
          </div>

          <p className="mt-14 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.12em] text-[var(--color-fg-quaternary)]">
            21 migrations applied · SEK01–SEK49 supported · Excel, PDF, CSV export
          </p>
        </motion.div>
      </div>
    </section>
  );
}
