'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { FileCode2, Coins, FileText } from 'lucide-react';
import { ScrollScene } from './scroll-scene';
import { HoverVideo } from './hover-video';
import { TextEffect } from './ui/text-effect';
import { SpotlightCard } from './ui/spotlight-card';
import { ProgressiveSeam, AccentGleam } from './ui/edge-bleed';

const CARDS = [
  {
    image: '/assets/gen/feature-blueprint.png',
    video: '/assets/gen/video-card-drawing.mp4',
    icon: FileCode2,
    eyebrow: 'Parse',
    title: 'The drawing talks back.',
    body: 'Upload a DXF, DWG, or PDF. KCC parses every layer, dimension, block, and annotation — walls, columns, openings, steel members. Spatial index, dimension-to-geometry links, feature extraction, the lot. You get back a structured model of the drawing, not a blob of text.',
  },
  {
    image: '/assets/gen/feature-materials.png',
    video: '/assets/gen/video-card-prices.mp4',
    icon: Coins,
    eyebrow: 'Price',
    title: 'Live Bulgarian prices, not a 2019 CSV.',
    body: 'KCC maps each quantity to its СЕК cost code, then pulls current market prices — either from supplier pages via the scraping pipeline, or researched on the spot through Perplexity + Claude Opus. Every row shows its price source and confidence.',
  },
  {
    image: '/assets/gen/feature-data.png',
    video: '/assets/gen/video-card-export.mp4',
    icon: FileText,
    eyebrow: 'Export',
    title: 'A КСС you can hand to a client.',
    body: 'Grouped by СЕК, labour + material + mechanisation + overhead broken out, audit trail on every line, ready to export to Excel (ОБРАЗЕЦ 9.1 compatible), PDF, or CSV. Changes you make in the UI feed back as corrections the pipeline learns from.',
  },
];

export function Features() {
  return (
    <ScrollScene
      id="features"
      video="/assets/gen/video-bg-features.mp4"
      poster="/assets/gen/bg-features.png"
      overlay="top"
      overlayStrength={0.88}
      scrollLinked
      minHeight="auto"
      className="py-32 md:py-44 relative"
    >
      <ProgressiveSeam direction="top" height={200} />
      <AccentGleam position={{ left: '10%', top: '-5%' }} size={800} opacity={0.12} />

      <div className="relative">
        <div className="max-w-2xl mb-16">
          <Eyebrow className="mb-4 block">What it does</Eyebrow>
          <TextEffect
            as="h2"
            className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
            stagger={0.04}
            triggerOnView
          >
            Three steps replace an afternoon of spreadsheet work.
          </TextEffect>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
          {CARDS.map((card, i) => (
            <motion.div
              key={card.title}
              initial={{ opacity: 0, y: 18, filter: 'blur(8px)' }}
              whileInView={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
              viewport={{ once: true, margin: '-80px' }}
              transition={{ duration: 0.7, delay: i * 0.08, ease: [0.22, 0.61, 0.36, 1] }}
              className="h-[500px]"
            >
              <SpotlightCard className="border-shine liquid-glass h-full flex flex-col rounded-2xl overflow-hidden">
                <div className="relative h-[58%] overflow-hidden">
                  <HoverVideo
                    poster={card.image}
                    video={card.video}
                    idleOpacity={0.85}
                    activeOpacity={1}
                  />
                  <div
                    aria-hidden
                    className="absolute inset-0 bg-gradient-to-t from-[var(--color-bg-raised)]/90 via-[var(--color-bg-raised)]/20 to-transparent"
                  />
                </div>
                <div className="relative flex-1 p-7 flex flex-col justify-between">
                  <div>
                    <div className="flex items-center justify-between mb-4">
                      <div className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                        <card.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                      </div>
                      <span className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)]">
                        {card.eyebrow}
                      </span>
                    </div>
                    <h3 className="text-[length:var(--text-lg)] font-semibold leading-[1.15] tracking-[-0.015em]">
                      {card.title}
                    </h3>
                  </div>
                  <p className="mt-4 text-[13.5px] leading-[1.6] text-[var(--color-fg-secondary)]">
                    {card.body}
                  </p>
                </div>
              </SpotlightCard>
            </motion.div>
          ))}
        </div>
      </div>
    </ScrollScene>
  );
}
