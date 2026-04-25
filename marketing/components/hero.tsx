'use client';

import { motion } from 'motion/react';
import { ArrowRight } from 'lucide-react';
import { Eyebrow } from './eyebrow';
import { ScrollScene } from './scroll-scene';
import { TextAnimate } from './ui/text-animate';
import { AuroraText } from './ui/aurora-text';
import { AnimatedShinyText } from './ui/animated-shiny-text';
import { Magnetic } from './ui/magnetic';
import { BorderBeam } from './ui/border-beam';
import { ShineBorder } from './ui/shine-border';
import { Particles } from './ui/particles';
import { NumberTicker } from './ui/number-ticker';
import { ProgressiveSeam, AccentGleam } from './ui/edge-bleed';
import dynamic from 'next/dynamic';

const Threads = dynamic(() => import('./backgrounds/threads'), { ssr: false });

export function Hero() {
  return (
    <ScrollScene
      id="hero"
      video="/assets/gen/video-bg-hero.mp4"
      poster="/assets/gen/hero.png"
      overlay="left"
      overlayStrength={0.76}
      scrollLinked
      minHeight="112svh"
      className="flex items-center"
    >
      <Threads className="absolute inset-0 z-[3] opacity-[0.35]" amplitude={0.6} distance={0.12} />
      <Particles quantity={30} className="z-[4]" />
      <AccentGleam position={{ left: '30%', bottom: '-10%' }} size={900} opacity={0.16} />
      <ProgressiveSeam direction="bottom" height={200} className="z-[5]" />

      <div className="relative pt-44 pb-32 z-10">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="max-w-2xl"
        >
          <Eyebrow className="mb-6 inline-block">
            <AnimatedShinyText shimmerWidth={120}>Construction estimating, on rails</AnimatedShinyText>
          </Eyebrow>

          <h1 className="text-[length:var(--text-4xl)] md:text-[length:var(--text-5xl)] leading-[1.02] tracking-[-0.035em] font-medium">
            <TextAnimate
              as="span"
              animation="blurInUp"
              by="word"
              duration={0.55}
              delay={0.05}
              once
              className="block"
            >
              From DXF to КСС in under
            </TextAnimate>
            <span className="block mt-1">
              <AuroraText speed={0.8}>three minutes.</AuroraText>
            </span>
          </h1>

          <motion.p
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.7 }}
            className="mt-8 text-[length:var(--text-lg)] leading-[1.55] text-[var(--color-fg-secondary)] max-w-[52ch]"
          >
            Upload the drawing. The pipeline reads every layer, pulls live Bulgarian market prices, and
            returns a priced КСС with an audit trail for every row.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.85 }}
            className="mt-10 flex flex-wrap items-center gap-4"
          >
            <Magnetic intensity={0.3}>
              <a
                href="https://auth.kccgen.xyz"
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
            transition={{ duration: 0.6, delay: 1.05 }}
            className="mt-20 grid grid-cols-3 gap-6 max-w-lg"
          >
            {[
              { v: 21, s: 'migrations', d: 14 },
              { v: 49, s: 'СЕК codes', prefix: 'SEK01–', d: 18 },
              { v: 3, s: 'export formats', d: 16 },
            ].map((stat) => (
              <div
                key={stat.s}
                className="relative liquid-glass rounded-xl px-4 py-4 border-l border-[var(--color-hairline-hi)] overflow-hidden"
              >
                <ShineBorder
                  borderWidth={1}
                  duration={stat.d}
                  shineColor={['oklch(0.82 0.19 62 / 0.6)', 'transparent', 'oklch(0.78 0.14 60 / 0.4)']}
                />
                <div className="relative font-[family-name:var(--font-mono)] text-[22px] md:text-[26px] text-[var(--color-amber)] tabular-nums tracking-[-0.02em]">
                  {stat.prefix && (
                    <span className="text-[var(--color-fg-tertiary)] text-[14px] mr-0.5">
                      {stat.prefix}
                    </span>
                  )}
                  <NumberTicker value={stat.v} />
                </div>
                <div className="relative mt-1 font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
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
