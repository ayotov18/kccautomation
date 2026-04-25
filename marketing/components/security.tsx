'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { Lock, FileSymlink, Database } from 'lucide-react';
import { HyperText } from './ui/hyper-text';
import { ShineBorder } from './ui/shine-border';
import { SpotlightCard } from './ui/spotlight-card';

const ITEMS = [
  {
    icon: Lock,
    title: 'JWT auth + Argon2 hashing',
    body: 'Per-user, every query scoped to the calling user.',
  },
  {
    icon: FileSymlink,
    title: 'SHA-256 deduplication',
    body: 'Same file uploaded twice is the same row, not a leak.',
  },
  {
    icon: Database,
    title: 'User-scoped S3 paths',
    body: 'Nothing sits in a shared bucket prefix.',
  },
];

export function Security() {
  return (
    <section className="relative py-28 md:py-36 overflow-hidden">
      <div aria-hidden className="absolute inset-0 grid-bg opacity-[0.06] pointer-events-none" />
      <div
        aria-hidden
        className="absolute inset-0 pointer-events-none"
        style={{ background: 'radial-gradient(circle at 80% 50%, oklch(0.4 0.04 240 / 0.12), transparent 60%)' }}
      />
      <div className="relative mx-auto max-w-7xl px-6">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">On data</Eyebrow>
          <HyperText
            as="h2"
            className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
            duration={1100}
            startOnView
          >
            Your drawings, your prices, your audit trail.
          </HyperText>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {ITEMS.map((item, i) => (
            <motion.div
              key={item.title}
              initial={{ opacity: 0, y: 12 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-40px' }}
              transition={{ duration: 0.5, delay: i * 0.08 }}
            >
              <SpotlightCard className="liquid-glass border-shine rounded-2xl p-7 relative overflow-hidden">
                <ShineBorder
                  borderWidth={1}
                  duration={18 + i * 4}
                  shineColor={['oklch(0.4 0.04 240 / 0.5)', 'transparent', 'oklch(0.82 0.19 62 / 0.4)']}
                />
                <div className="relative mb-4 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                  <item.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                </div>
                <h3 className="text-[14px] font-[family-name:var(--font-mono)] uppercase tracking-[0.08em]">
                  {item.title}
                </h3>
                <p className="mt-3 text-[13px] leading-[1.65] text-[var(--color-fg-secondary)]">
                  {item.body}
                </p>
              </SpotlightCard>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
