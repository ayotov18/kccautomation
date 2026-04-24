'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';

export function Testimonial() {
  return (
    <section className="relative py-24 md:py-36 overflow-hidden border-y border-[var(--color-hairline)]">
      <div
        aria-hidden
        className="absolute inset-0 opacity-60"
        style={{
          backgroundImage: "url('/assets/gen/testimonial.png')",
          backgroundSize: 'cover',
          backgroundPosition: 'center',
        }}
      />
      <div
        aria-hidden
        className="absolute inset-0 bg-gradient-to-r from-[var(--color-bg)] via-[var(--color-bg)]/80 to-transparent"
      />
      <div aria-hidden className="absolute inset-0 grain pointer-events-none" />

      <div className="relative mx-auto max-w-7xl px-6">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-60px' }}
          transition={{ duration: 0.7 }}
          className="max-w-2xl"
        >
          <Eyebrow className="mb-4 block">In production</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Built by people who've priced a thousand drawings by hand.
          </h2>
          <p className="mt-6 text-[15px] leading-relaxed text-[var(--color-fg-secondary)]">
            KCC replaces the 2–4 hour manual КСС pass with 2–3 minutes of pipeline work and 5 minutes of
            review. The tool exists because the team building it got tired of the alternative.
          </p>

          <div className="mt-12 border-l-2 border-[var(--color-amber)] pl-6 max-w-xl">
            <p className="text-[20px] md:text-[22px] leading-snug tracking-tight text-[var(--color-fg)]">
              “The first estimating tool I've used that admits when it's guessing.”
            </p>
            <p className="mt-4 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
              — placeholder, to be replaced pre-launch
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
