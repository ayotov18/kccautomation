'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { FileCode2, Coins, FileText } from 'lucide-react';
import { ScrollScene } from './scroll-scene';
import { HoverVideo } from './hover-video';

const CARDS = [
  {
    image: '/assets/gen/feature-blueprint.png',
    video: '/assets/gen/video-card-drawing.mp4',
    icon: FileCode2,
    title: 'The drawing talks back.',
    body: 'Upload a DXF, DWG, or PDF. KCC parses every layer, dimension, block, and annotation — walls, columns, openings, steel members. Spatial index, dimension-to-geometry links, feature extraction, the lot. You get back a structured model of the drawing, not a blob of text.',
  },
  {
    image: '/assets/gen/feature-materials.png',
    video: '/assets/gen/video-card-prices.mp4',
    icon: Coins,
    title: 'Live Bulgarian prices, not a 2019 CSV.',
    body: 'KCC maps each quantity to its СЕК cost code, then pulls current market prices — either from supplier pages via the scraping pipeline, or researched on the spot through Perplexity + Claude Opus. Every row shows its price source and confidence.',
  },
  {
    image: '/assets/gen/feature-data.png',
    video: '/assets/gen/video-card-export.mp4',
    icon: FileText,
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
      overlayStrength={0.9}
      scrollLinked
      minHeight="auto"
      className="py-24 md:py-36"
    >
      <div className="relative">
        <div className="max-w-2xl mb-16">
          <Eyebrow className="mb-4 block">What it does</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Three steps replace an afternoon of spreadsheet work.
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
          {CARDS.map((card, i) => (
            <motion.div
              key={card.title}
              initial={{ opacity: 0, y: 16 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-80px' }}
              transition={{ duration: 0.55, delay: i * 0.08, ease: [0.22, 0.61, 0.36, 1] }}
              className="group relative overflow-hidden rounded-2xl border border-[var(--color-hairline)] bg-[var(--color-bg-raised)] h-[460px] flex flex-col"
            >
              <HoverVideo poster={card.image} video={card.video} />
              <div
                aria-hidden
                className="absolute inset-0 bg-gradient-to-t from-[var(--color-bg-raised)] via-[var(--color-bg-raised)]/60 to-transparent"
              />
              <div className="relative mt-auto p-6">
                <div className="mb-4 inline-flex h-9 w-9 items-center justify-center rounded-md border border-[var(--color-hairline-hi)] bg-[var(--color-surface)]/80 backdrop-blur">
                  <card.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                </div>
                <h3 className="text-[19px] font-semibold leading-tight tracking-tight">{card.title}</h3>
                <p className="mt-3 text-[13.5px] leading-relaxed text-[var(--color-fg-secondary)]">
                  {card.body}
                </p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </ScrollScene>
  );
}
