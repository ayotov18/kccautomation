'use client';

import { ReactNode } from 'react';
import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { TextEffect } from './ui/text-effect';
import { ProgressiveSeam, AccentGleam } from './ui/edge-bleed';

type Props = {
  eyebrow: string;
  title: string;
  sub: string;
  hero: string;
  children: ReactNode;
};

/**
 * Sub-page hero shell. Static poster image only — no Threads (WebGL) or Particles
 * canvas, to keep memory low when navigating between pages.
 * Threads + heavy canvas live only on the home page.
 */
export function SubPageShell({ eyebrow, title, sub, hero, children }: Props) {
  return (
    <main className="relative">
      <section className="relative min-h-[80svh] flex items-center overflow-hidden">
        <div
          aria-hidden
          className="absolute inset-0 z-0"
          style={{
            backgroundImage: `url('${hero}')`,
            backgroundSize: 'cover',
            backgroundPosition: 'center',
          }}
        />
        <div
          aria-hidden
          className="absolute inset-0 z-[1] bg-gradient-to-r from-[var(--color-bg)] via-[var(--color-bg)]/85 to-transparent"
        />
        <AccentGleam position={{ left: '20%', bottom: '0%' }} size={900} opacity={0.16} />
        <ProgressiveSeam direction="bottom" height={200} className="z-[4]" />

        <div className="relative z-10 w-full mx-auto max-w-7xl px-6 pt-44 pb-32">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.6 }}
            className="max-w-3xl"
          >
            <Eyebrow className="mb-6 inline-block">{eyebrow}</Eyebrow>
            <TextEffect
              as="h1"
              className="text-[length:var(--text-4xl)] leading-[1.04] tracking-[-0.035em] font-medium"
              stagger={0.05}
            >
              {title}
            </TextEffect>
            <motion.p
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.6 }}
              className="mt-8 text-[length:var(--text-lg)] leading-[1.55] text-[var(--color-fg-secondary)] max-w-[58ch]"
            >
              {sub}
            </motion.p>
          </motion.div>
        </div>
      </section>

      {children}
    </main>
  );
}
