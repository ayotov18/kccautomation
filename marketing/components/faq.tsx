'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Plus } from 'lucide-react';
import { Eyebrow } from './eyebrow';
import { cn } from '@/lib/cn';
import { TextAnimate } from './ui/text-animate';
import { LiquidGlass } from './ui/liquid-glass';

const QA = [
  {
    q: 'Does it replace the estimator?',
    a: "No. It replaces the part of the estimator's day where they type a spreadsheet. Review still lives with the human.",
  },
  {
    q: 'What if a layer is mis-labelled in the drawing?',
    a: 'You write a DRM rule once and KCC applies it on every future drawing with that layer name. Corrections compound.',
  },
  {
    q: 'Which prices does the AI pull?',
    a: 'Bulgarian market, researched live via Perplexity with sonar-pro, then refined by Claude Opus into line items. Every row stores its source so you can re-check.',
  },
  {
    q: 'Which drawing formats work today?',
    a: 'DXF and PDF, fully. DWG works when the host has the ODA File Converter installed. Plans for IFC and RVT sit behind the current roadmap.',
  },
];

export function Faq() {
  const [open, setOpen] = useState<number | null>(0);

  return (
    <section
      id="faq"
      className="relative py-32 md:py-40 border-t border-[var(--color-hairline)] overflow-hidden"
    >
      <div aria-hidden className="absolute inset-0 grid-bg opacity-[0.05]" />

      <div className="relative mx-auto max-w-7xl px-6 grid md:grid-cols-2 gap-8 items-stretch">
        {/* Left — image + video half */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          whileInView={{ opacity: 1, x: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.22, 0.61, 0.36, 1] }}
          className="relative"
        >
          <LiquidGlass intensity="standard" className="relative aspect-[4/5] md:aspect-auto md:h-full overflow-hidden">
            <video
              aria-hidden
              autoPlay
              muted
              loop
              playsInline
              preload="metadata"
              poster="/assets/gen/testimonial.png"
              className="absolute inset-0 h-full w-full object-cover"
            >
              <source src="/assets/gen/video-bg-testimonial.mp4" type="video/mp4" />
            </video>
            <div
              aria-hidden
              className="absolute inset-0 bg-gradient-to-tr from-[var(--color-bg)] via-[var(--color-bg)]/50 to-transparent"
            />
            <div className="relative h-full flex flex-col justify-between p-8 md:p-10 z-10">
              <div>
                <Eyebrow className="mb-4 block">Questions</Eyebrow>
                <TextAnimate
                  as="h2"
                  animation="slideLeft"
                  by="word"
                  duration={0.55}
                  className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
                >
                  Things people ask in the first demo call.
                </TextAnimate>
              </div>
              <p className="text-[13px] font-[family-name:var(--font-mono)] uppercase tracking-[0.16em] text-[var(--color-fg-quaternary)]">
                Still have one? hello@kccgen.xyz
              </p>
            </div>
          </LiquidGlass>
        </motion.div>

        {/* Right — accordion */}
        <motion.ul
          initial={{ opacity: 0, x: 20 }}
          whileInView={{ opacity: 1, x: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.8, ease: [0.22, 0.61, 0.36, 1], delay: 0.1 }}
          className="divide-y divide-white/5 border-y border-white/5 self-center"
        >
          {QA.map((item, i) => (
            <li key={item.q}>
              <button
                type="button"
                onClick={() => setOpen(open === i ? null : i)}
                className="flex w-full items-center justify-between gap-4 py-6 text-left text-[15px] font-medium hover:text-[var(--color-amber-soft)] transition-colors"
              >
                <span className="tracking-[-0.01em]">{item.q}</span>
                <Plus
                  className={cn(
                    'h-4 w-4 shrink-0 text-[var(--color-fg-tertiary)] transition-transform duration-300',
                    open === i && 'rotate-45 text-[var(--color-amber)]',
                  )}
                />
              </button>
              <AnimatePresence initial={false}>
                {open === i && (
                  <motion.div
                    key="content"
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.35, ease: [0.22, 0.61, 0.36, 1] }}
                    className="overflow-hidden"
                  >
                    <p className="pb-6 pr-8 text-[14px] leading-[1.65] text-[var(--color-fg-secondary)]">
                      {item.a}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </li>
          ))}
        </motion.ul>
      </div>
    </section>
  );
}
