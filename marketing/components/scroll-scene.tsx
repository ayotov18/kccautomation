'use client';

import { useEffect, useRef, ReactNode } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  video: string;
  poster?: string;
  overlay?: 'left' | 'right' | 'center' | 'bottom' | 'top' | 'none';
  overlayStrength?: number; // 0..1
  className?: string;
  children: ReactNode;
  id?: string;
  /**
   * Scroll-linked: when visible, the video's playhead maps to scroll progress.
   * When not visible, the video pauses and holds its current frame.
   */
  scrollLinked?: boolean;
  /** Autoplay loop instead of scroll-linking. */
  autoplay?: boolean;
  minHeight?: string;
};

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

  useEffect(() => {
    const section = sectionRef.current;
    const vid = videoRef.current;
    if (!section || !vid) return;

    let duration = 0;
    let playing = false;

    const onMeta = () => {
      duration = vid.duration || 0;
    };
    vid.addEventListener('loadedmetadata', onMeta);
    if (vid.readyState >= 1) duration = vid.duration || 0;

    if (autoplay) {
      vid.loop = true;
      vid.muted = true;
      vid.setAttribute('playsinline', '');
      vid.play().catch(() => {});
      return () => vid.removeEventListener('loadedmetadata', onMeta);
    }

    if (!scrollLinked) {
      return () => vid.removeEventListener('loadedmetadata', onMeta);
    }

    // Scroll-linked: map scrollProgress through the section to video.currentTime
    let rafId = 0;
    const update = () => {
      rafId = 0;
      if (!section || !vid || !duration) return;
      const rect = section.getBoundingClientRect();
      const vh = window.innerHeight || 800;
      const total = rect.height + vh;
      const progressed = Math.min(Math.max((vh - rect.top) / total, 0), 1);
      const target = progressed * duration;
      // Only update when delta is big enough to avoid fighting the GPU
      if (Math.abs(target - vid.currentTime) > 0.03) {
        try {
          vid.currentTime = target;
        } catch {}
      }
    };
    const schedule = () => {
      if (!rafId) rafId = requestAnimationFrame(update);
    };
    window.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule);

    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) {
            playing = true;
            schedule();
          } else {
            playing = false;
          }
        }
      },
      { threshold: 0 },
    );
    io.observe(section);
    schedule();

    return () => {
      vid.removeEventListener('loadedmetadata', onMeta);
      window.removeEventListener('scroll', schedule);
      window.removeEventListener('resize', schedule);
      io.disconnect();
      if (rafId) cancelAnimationFrame(rafId);
      void playing;
    };
  }, [scrollLinked, autoplay, video]);

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
        preload="auto"
        poster={poster}
        className="absolute inset-0 h-full w-full object-cover z-0"
      >
        <source src={video} type="video/mp4" />
      </video>
      <div aria-hidden className={cn('absolute inset-0 z-[1] pointer-events-none', overlayClass)} style={{ opacity: overlayStrength }} />
      <div aria-hidden className="absolute inset-0 z-[2] grid-bg opacity-[0.08] pointer-events-none" />
      <div aria-hidden className="absolute inset-0 z-[3] grain pointer-events-none" />
      <div className="relative z-10 w-full mx-auto max-w-7xl px-6">{children}</div>
    </section>
  );
}
