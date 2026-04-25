'use client';

import { motion } from 'motion/react';
import { Eyebrow } from './eyebrow';
import { Gauge, Layers, Search, FileBadge, FileSpreadsheet } from 'lucide-react';
import { ScrollScene } from './scroll-scene';
import { HoverVideo } from './hover-video';
import { TextAnimate } from './ui/text-animate';
import { SpotlightCard } from './ui/spotlight-card';
import { ShineBorder } from './ui/shine-border';
import { ProgressiveSeam } from './ui/edge-bleed';

const TILES = [
  {
    span: 'md:col-span-2 md:row-span-2',
    icon: Gauge,
    eyebrow: 'Confidence scoring',
    title: 'Every number knows how sure it is.',
    body: 'Each extracted quantity carries a 0.0–1.0 score based on how it was derived. Shoelace-computed areas score 0.9. Guessed values score 0.4. Anything below 0.6 surfaces to the review widget instead of silently rolling into the total.',
    poster: '/assets/gen/tile-confidence.png',
    video: '/assets/gen/video-tile-confidence.mp4',
  },
  {
    span: 'md:col-span-2',
    icon: Layers,
    eyebrow: 'DRM',
    title: 'Drawing Rule Mapping',
    body: 'layer "steni-beton" → actually brick, use СЕК05 not СЕК04. Rules you write once, applied automatically on every КСС the pipeline generates.',
    poster: '/assets/gen/tile-drm.png',
    video: '/assets/gen/video-tile-drm.mp4',
  },
  {
    span: 'md:col-span-1',
    icon: Search,
    eyebrow: 'Price research',
    title: 'Cited, not guessed.',
    body: 'BrightData-proxied scrapes and a Perplexity + Opus loop. Deduped by normalized key, cached in scraped_price_rows for reuse.',
    poster: '/assets/gen/feature-materials.png',
    video: '/assets/gen/video-card-prices.mp4',
  },
  {
    span: 'md:col-span-1',
    icon: FileBadge,
    eyebrow: 'Audit trail',
    title: 'Defensible on every row.',
    body: 'Which rule fired, which price source was used, and the Opus reasoning. Persisted to kss_audit_trail.',
    poster: '/assets/gen/tile-audit.png',
    video: '/assets/gen/video-tile-audit.mp4',
  },
  {
    span: 'md:col-span-4',
    icon: FileSpreadsheet,
    eyebrow: 'Exports',
    title: 'The files the estimator actually hands over.',
    body: 'Excel via rust_xlsxwriter matching ОБРАЗЕЦ 9.1, PDF via the internal report crate, CSV for anything else. One endpoint per format, streaming, no "please wait while we build your file" screen.',
    poster: '/assets/gen/feature-data.png',
    video: '/assets/gen/video-card-export.mp4',
  },
];

export function Bento() {
  return (
    <ScrollScene
      id="bento"
      video="/assets/gen/video-bg-bento.mp4"
      poster="/assets/gen/bg-bento.png"
      overlay="top"
      overlayStrength={0.92}
      scrollLinked
      minHeight="auto"
      className="py-32 md:py-44 relative"
    >
      <ProgressiveSeam direction="top" height={160} />

      <div className="relative">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">Under the hood</Eyebrow>
          <TextAnimate
            as="h2"
            animation="slideUp"
            by="word"
            duration={0.5}
            className="text-[length:var(--text-3xl)] leading-[1.04] tracking-[-0.025em]"
          >
            Confidence on every number, not just the ones the demo shows.
          </TextAnimate>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 md:auto-rows-[220px]">
          {TILES.map((tile, i) => (
            <motion.div
              key={tile.title}
              initial={{ opacity: 0, y: 14, filter: 'blur(6px)' }}
              whileInView={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.6, delay: i * 0.06 }}
              className={tile.span + ' min-h-[220px]'}
            >
              <SpotlightCard className="border-shine liquid-glass h-full rounded-2xl overflow-hidden relative">
                <ShineBorder
                  borderWidth={1}
                  duration={14 + (i % 3) * 4}
                  shineColor={['oklch(0.82 0.19 62 / 0.55)', 'transparent', 'oklch(0.78 0.14 60 / 0.4)']}
                />
                <HoverVideo
                  poster={tile.poster}
                  video={tile.video}
                  idleOpacity={0.3}
                  activeOpacity={0.88}
                />
                <div
                  aria-hidden
                  className="absolute inset-0 bg-gradient-to-t from-[var(--color-bg-raised)]/95 via-[var(--color-bg-raised)]/60 to-transparent"
                />
                <div className="relative flex h-full flex-col justify-between p-6">
                  <div>
                    <div className="mb-4 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5">
                      <tile.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
                    </div>
                    <div className="font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)] mb-2">
                      {tile.eyebrow}
                    </div>
                    <h3 className="text-[length:var(--text-lg)] font-semibold leading-[1.15] tracking-[-0.015em]">
                      {tile.title}
                    </h3>
                  </div>
                  <p className="mt-4 text-[13px] leading-[1.6] text-[var(--color-fg-secondary)]">
                    {tile.body}
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
