'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { ScrollScene } from './scroll-scene';
import { TextAnimate } from './ui/text-animate';
import { LiquidGlass } from './ui/liquid-glass';
import { ProgressiveSeam } from './ui/edge-bleed';

export function Testimonial() {
  return (
    <ScrollScene
      id="testimonial"
      video="/assets/gen/video-bg-testimonial.mp4"
      poster="/assets/gen/testimonial.png"
      overlay="left"
      overlayStrength={0.78}
      scrollLinked
      minHeight="auto"
      className="py-32 md:py-40 relative"
    >
      <ProgressiveSeam direction="top" height={160} />
      <ProgressiveSeam direction="bottom" height={160} />

      <motion.div
        initial={{ opacity: 0, y: 18 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: '-60px' }}
        transition={{ duration: 0.75 }}
        className="max-w-2xl"
      >
        <Eyebrow className="mb-4 block">In production</Eyebrow>
        <TextAnimate
          as="h2"
          animation="fadeIn"
          by="line"
          duration={0.7}
          className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
        >
          {`Built by people\nwho've priced a thousand drawings by hand.`}
        </TextAnimate>
        <p className="mt-6 text-[length:var(--text-base)] leading-[1.6] text-[var(--color-fg-secondary)]">
          KCC replaces the 2–4 hour manual КСС pass with 2–3 minutes of pipeline work and 5 minutes of
          review. The tool exists because the team building it got tired of the alternative.
        </p>
        <LiquidGlass intensity="soft" className="mt-10 p-8 max-w-xl rounded-2xl border-l-2 border-[var(--color-amber)]">
          <p className="font-[family-name:var(--font-serif-loaded)] italic text-[22px] md:text-[26px] leading-[1.3] tracking-[-0.01em] text-[var(--color-fg)]">
            &ldquo;The first estimating tool I've used that admits when it's guessing.&rdquo;
          </p>
          <p className="mt-5 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
            — placeholder, to be replaced pre-launch
          </p>
        </LiquidGlass>
      </motion.div>
    </ScrollScene>
  );
}
