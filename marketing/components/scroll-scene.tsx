'use client';

import { useEffect, useRef, ReactNode } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  video: string;
  poster?: string;
  overlay?: 'left' | 'right' | 'center' | 'bottom' | 'top' | 'none';
  overlayStrength?: number;
  className?: string;
  children: ReactNode;
  id?: string;
  /** When section enters view, scrub video.currentTime by scroll progress. */
  scrollLinked?: boolean;
  /** Loop autoplay regardless of scroll. */
  autoplay?: boolean;
  minHeight?: string;
};

/**
 * Video-backed section.
 *
 * Memory-aware:
 * - preload="metadata" (not "auto") — only fetches headers until needed.
 * - Lazy src attach: <source> is only added when the section enters viewport.
 *   When the section leaves viewport we pause, detach src, and call .load() to
 *   release decoded frames from VRAM.
 * - Scroll-linked currentTime writes are throttled to ~50 ms and only run while
 *   the section is in view.
 */
export function ScrollScene({
  video,
  poster,
  overlay = 'left',
  overlayStrength = 0.82,
  className,
  children,
  id,
  scrollLinked = true,
  autoplay = false,
  minHeight = '100svh',
}: Props) {
  const sectionRef = useRef<HTMLElement | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const visibleRef = useRef(false);
  const lastWriteRef = useRef(0);
  const rafRef = useRef(0);
  const durationRef = useRef(0);

  useEffect(() => {
    const section = sectionRef.current;
    const vid = videoRef.current;
    if (!section || !vid) return;

    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    function attachSrc() {
      if (!vid) return;
      if (vid.querySelector('source')) return;
      const src = document.createElement('source');
      src.src = video;
      src.type = 'video/mp4';
      vid.appendChild(src);
      vid.load();
    }

    function detachSrc() {
      if (!vid) return;
      const existing = vid.querySelector('source');
      if (existing) vid.removeChild(existing);
      try {
        vid.pause();
        vid.removeAttribute('src');
        vid.load(); // forces release of decoded frames
      } catch {}
    }

    function onMeta() {
      if (vid) durationRef.current = vid.duration || 0;
    }
    vid.addEventListener('loadedmetadata', onMeta);

    function tickScroll() {
      rafRef.current = 0;
      if (!section || !vid || !visibleRef.current) return;
      const now = performance.now();
      if (now - lastWriteRef.current < 50) return; // throttle to ~20fps writes
      const dur = durationRef.current;
      if (!dur) return;
      const rect = section.getBoundingClientRect();
      const vh = window.innerHeight || 800;
      const total = rect.height + vh;
      const progressed = Math.min(Math.max((vh - rect.top) / total, 0), 1);
      const target = progressed * dur;
      if (Math.abs(target - vid.currentTime) > 0.05) {
        try {
          vid.currentTime = target;
          lastWriteRef.current = now;
        } catch {}
      }
    }

    function scheduleScroll() {
      if (rafRef.current) return;
      rafRef.current = requestAnimationFrame(tickScroll);
    }

    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting && !visibleRef.current) {
            visibleRef.current = true;
            attachSrc();
            if (autoplay && !reduce) {
              vid.loop = true;
              vid.muted = true;
              vid.setAttribute('playsinline', '');
              vid.play().catch(() => {});
            }
            if (scrollLinked && !autoplay) scheduleScroll();
          } else if (!e.isIntersecting && visibleRef.current) {
            visibleRef.current = false;
            if (autoplay) vid.pause();
            // release video memory while off-screen
            detachSrc();
          }
        }
      },
      { threshold: 0, rootMargin: '120px' },
    );
    io.observe(section);

    if (scrollLinked && !autoplay && !reduce) {
      window.addEventListener('scroll', scheduleScroll, { passive: true });
      window.addEventListener('resize', scheduleScroll);
    }

    return () => {
      io.disconnect();
      vid.removeEventListener('loadedmetadata', onMeta);
      window.removeEventListener('scroll', scheduleScroll);
      window.removeEventListener('resize', scheduleScroll);
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      detachSrc();
    };
  }, [video, scrollLinked, autoplay]);

  const overlayClass =
    overlay === 'left'
      ? 'bg-gradient-to-r from-[var(--color-bg)] via-[var(--color-bg)]/80 to-transparent'
      : overlay === 'right'
        ? 'bg-gradient-to-l from-[var(--color-bg)] via-[var(--color-bg)]/80 to-transparent'
        : overlay === 'top'
          ? 'bg-gradient-to-b from-[var(--color-bg)] via-[var(--color-bg)]/60 to-transparent'
          : overlay === 'bottom'
            ? 'bg-gradient-to-t from-[var(--color-bg)] via-[var(--color-bg)]/60 to-transparent'
            : overlay === 'center'
              ? 'bg-[radial-gradient(ellipse_at_center,transparent_30%,rgba(10,10,12,0.9)_100%)]'
              : 'bg-transparent';

  return (
    <section
      ref={sectionRef}
      id={id}
      className={cn('relative overflow-hidden', className)}
      style={{ minHeight }}
    >
      <video
        ref={videoRef}
        aria-hidden
        muted
        playsInline
        preload="metadata"
        poster={poster}
        disablePictureInPicture
        disableRemotePlayback
        className="absolute inset-0 h-full w-full object-cover z-0"
        // src/source attached lazily on intersect
      />
      <div
        aria-hidden
        className={cn('absolute inset-0 z-[1] pointer-events-none', overlayClass)}
        style={{ opacity: overlayStrength }}
      />
      <div aria-hidden className="absolute inset-0 z-[2] grid-bg opacity-[0.08] pointer-events-none" />
      <div aria-hidden className="absolute inset-0 z-[3] grain pointer-events-none" />
      <div className="relative z-10 w-full mx-auto max-w-7xl px-6">{children}</div>
    </section>
  );
}
