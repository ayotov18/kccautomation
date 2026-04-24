'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Plus } from 'lucide-react';
import { Eyebrow } from './eyebrow';
import { cn } from '@/lib/cn';

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
    <section id="faq" className="relative py-24 md:py-36 border-t border-[var(--color-hairline)]">
      <div className="mx-auto max-w-7xl px-6 grid md:grid-cols-[1fr_1.4fr] gap-12">
        <div>
          <Eyebrow className="mb-4 block">Questions</Eyebrow>
          <h2 className="text-[clamp(1.75rem,3.4vw,2.6rem)] font-semibold leading-[1.1] tracking-tight">
            Things people ask in the first demo call.
          </h2>
        </div>

        <ul className="divide-y divide-[var(--color-hairline)] border-y border-[var(--color-hairline)]">
          {QA.map((item, i) => (
            <li key={item.q}>
              <button
                type="button"
                onClick={() => setOpen(open === i ? null : i)}
                className="flex w-full items-center justify-between gap-4 py-5 text-left text-[15px] font-medium hover:text-[var(--color-amber-soft)] transition-colors"
              >
                <span>{item.q}</span>
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
                    transition={{ duration: 0.3, ease: [0.22, 0.61, 0.36, 1] }}
                    className="overflow-hidden"
                  >
                    <p className="pb-6 pr-8 text-[14px] leading-relaxed text-[var(--color-fg-secondary)]">
                      {item.a}
                    </p>
                  </motion.div>
                )}
              </AnimatePresence>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
