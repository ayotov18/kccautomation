'use client';

import { motion } from 'motion/react';
import { ArrowRight } from 'lucide-react';
import { ScrollScene } from './scroll-scene';
import { TextEffect } from './ui/text-effect';
import { Magnetic } from './ui/magnetic';
import { BorderBeam } from './ui/border-beam';
import { ProgressiveSeam, AccentGleam } from './ui/edge-bleed';

export function CTA() {
  return (
    <ScrollScene
      id="cta"
      video="/assets/gen/video-bg-cta.mp4"
      poster="/assets/gen/cta.png"
      overlay="center"
      overlayStrength={0.65}
      scrollLinked
      minHeight="auto"
      className="py-36 md:py-48 overflow-hidden relative"
    >
      <ProgressiveSeam direction="top" height={200} />
      <AccentGleam position={{ left: '50%', top: '20%' }} size={1100} opacity={0.18} />

      <div className="max-w-4xl mx-auto text-center">
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.8 }}
        >
          <TextEffect
            as="h2"
            className="text-[length:var(--text-4xl)] leading-[1.04] tracking-[-0.03em] font-medium"
            stagger={0.045}
            triggerOnView
          >
            Ready to stop retyping spreadsheets?
          </TextEffect>
          <p className="mt-8 text-[length:var(--text-base)] leading-[1.6] text-[var(--color-fg-secondary)] max-w-xl mx-auto">
            Request access and we'll show you the full pipeline on one of your own drawings.
          </p>
          <div className="mt-12 flex flex-wrap justify-center items-center gap-4">
            <Magnetic intensity={0.3}>
              <a
                href="https://auth.kccgen.xyz"
                className="group relative inline-flex h-13 items-center gap-2 overflow-hidden rounded-full bg-[var(--color-amber)] px-8 py-3.5 text-[14px] font-medium text-[var(--color-bg)] transition-colors hover:bg-[var(--color-amber-hot)] amber-glow"
              >
                <BorderBeam size={140} duration={6} colorFrom="oklch(1 0 0 / 0.7)" colorTo="oklch(1 0 0 / 0)" />
                Request access
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </a>
            </Magnetic>
          </div>
          <p className="mt-16 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
            Built in Sofia · Rust + Next.js · No vendor lock-in
          </p>
        </motion.div>
      </div>
    </ScrollScene>
  );
}
