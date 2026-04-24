'use client';

import { motion } from 'motion/react';
import { ArrowRight } from 'lucide-react';

export function CTA() {
  return (
    <section id="cta" className="relative py-32 md:py-44 overflow-hidden">
      <div
        aria-hidden
        className="absolute inset-0 opacity-70"
        style={{
          backgroundImage: "url('/assets/gen/cta.png')",
          backgroundSize: 'cover',
          backgroundPosition: 'center',
        }}
      />
      <div
        aria-hidden
        className="absolute inset-0 bg-gradient-to-b from-[var(--color-bg)]/80 via-[var(--color-bg)]/60 to-[var(--color-bg)]"
      />
      <div aria-hidden className="absolute inset-0 grain pointer-events-none" />

      <div className="relative mx-auto max-w-4xl px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 14 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.7 }}
        >
          <h2 className="text-[clamp(2rem,5vw,3.6rem)] font-semibold leading-[1.06] tracking-tight">
            Ready to stop retyping spreadsheets?
          </h2>
          <p className="mt-6 text-[16px] leading-relaxed text-[var(--color-fg-secondary)] max-w-xl mx-auto">
            Request access and we'll show you the full pipeline on one of your own drawings.
          </p>
          <div className="mt-10 flex flex-wrap justify-center items-center gap-4">
            <a
              href="mailto:hello@kccgen.xyz?subject=KCC%20Automation%20%E2%80%94%20request%20access"
              className="group inline-flex h-12 items-center gap-2 rounded-lg bg-[var(--color-amber)] px-7 text-[14px] font-medium text-[var(--color-bg)] transition-all hover:bg-[var(--color-amber-soft)] shadow-[0_0_0_1px_rgba(184,115,51,0.2),0_8px_24px_rgba(184,115,51,0.3)]"
            >
              Request access
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
            </a>
          </div>
          <p className="mt-14 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.12em] text-[var(--color-fg-quaternary)]">
            Built in Sofia · Rust + Next.js · No vendor lock-in
          </p>
        </motion.div>
      </div>
    </section>
  );
}
