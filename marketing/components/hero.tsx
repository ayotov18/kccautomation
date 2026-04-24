'use client';

import { motion } from 'motion/react';
import { ArrowRight } from 'lucide-react';
import { Eyebrow } from './eyebrow';
import { ScrollScene } from './scroll-scene';
import { TextEffect } from './ui/text-effect';
import { Magnetic } from './ui/magnetic';
import { BorderBeam } from './ui/border-beam';
import { Particles } from './ui/particles';
import { NumberTicker } from './ui/number-ticker';

export function Hero() {
  return (
    <ScrollScene
      id="hero"
      video="/assets/gen/video-bg-hero.mp4"
      poster="/assets/gen/hero.png"
      overlay="left"
      overlayStrength={0.78}
      scrollLinked
      minHeight="110svh"
      className="flex items-center"
    >
      <Particles quantity={70} className="z-[4]" />

      <div className="relative pt-44 pb-28 z-10">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="max-w-2xl"
        >
          <Eyebrow className="mb-6 inline-block">Construction estimating, on rails</Eyebrow>

          <TextEffect
            as="h1"
            className="text-[clamp(2.25rem,5.5vw,4.75rem)] font-semibold leading-[1.04] tracking-tight block"
            stagger={0.06}
          >
            From DXF to КСС in under three minutes.
          </TextEffect>

          <motion.p
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.6 }}
            className="mt-6 text-[15px] md:text-[17px] leading-relaxed text-[var(--color-fg-secondary)] max-w-xl"
          >
            Upload the drawing. The pipeline reads every layer, pulls live Bulgarian market prices, and
            returns a priced КСС with an audit trail for every row.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.75 }}
            className="mt-10 flex flex-wrap items-center gap-4"
          >
            <Magnetic intensity={0.3}>
              <a
                href="#cta"
                className="group relative inline-flex h-12 items-center gap-2 overflow-hidden rounded-full bg-[var(--color-amber)] px-7 text-[14px] font-medium text-[var(--color-bg)] transition-colors hover:bg-[var(--color-amber-hot)] amber-glow"
              >
                <BorderBeam size={120} duration={6} colorFrom="oklch(1 0 0 / 0.7)" colorTo="oklch(1 0 0 / 0)" />
                Request access
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </a>
            </Magnetic>
            <a
              href="#pipeline"
              className="group inline-flex h-12 items-center gap-2 px-2 text-[14px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
            >
              See how the pipeline works
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
            </a>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.95 }}
            className="mt-16 grid grid-cols-3 gap-6 max-w-lg"
          >
            {[
              { v: 21, s: 'migrations' },
              { v: 49, s: 'СЕК codes', prefix: 'SEK01–' },
              { v: 3, s: 'export formats' },
            ].map((stat) => (
              <div key={stat.s} className="border-l border-[var(--color-hairline-hi)] pl-4">
                <div className="font-[family-name:var(--font-mono)] text-[22px] md:text-[26px] text-[var(--color-amber)] tabular-nums">
                  {stat.prefix && (
                    <span className="text-[var(--color-fg-tertiary)] text-[14px] mr-0.5">{stat.prefix}</span>
                  )}
                  <NumberTicker value={stat.v} />
                </div>
                <div className="mt-1 font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
                  {stat.s}
                </div>
              </div>
            ))}
          </motion.div>
        </motion.div>
      </div>
    </ScrollScene>
  );
}
