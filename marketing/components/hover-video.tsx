'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  poster: string;
  video?: string | null;
  className?: string;
  idleOpacity?: number;
  activeOpacity?: number;
  overlay?: string;
};

/**
 * Poster-by-default, plays on hover.
 *
 * Memory-aware:
 * - The <video> element is NOT mounted at first paint; only the poster image is.
 * - On first hover (and when the card is in viewport), we mount <video> and
 *   attach src. After the user leaves and the card scrolls off-screen, we
 *   un-mount the video to free the decoder.
 */
export function HoverVideo({
  poster,
  video,
  className,
  idleOpacity = 0.7,
  activeOpacity = 0.95,
  overlay,
}: Props) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [shouldMount, setShouldMount] = useState(false);
  const [hasInteracted, setHasInteracted] = useState(false);

  // When the card scrolls completely out of view AFTER having been hovered, unmount the <video>.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (!e.isIntersecting && hasInteracted && shouldMount) {
            // free the decoder
            setShouldMount(false);
          }
        }
      },
      { threshold: 0, rootMargin: '200px' },
    );
    io.observe(el);
    return () => io.disconnect();
  }, [hasInteracted, shouldMount]);

  const onEnter = useCallback(() => {
    if (!video) return;
    setShouldMount(true);
    setHasInteracted(true);
    requestAnimationFrame(() => {
      const v = videoRef.current;
      if (v) {
        v.currentTime = 0;
        v.play().catch(() => {});
      }
    });
  }, [video]);

  const onLeave = useCallback(() => {
    const v = videoRef.current;
    if (v) v.pause();
  }, []);

  return (
    <div
      ref={containerRef}
      className={cn('absolute inset-0', className)}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      onFocus={onEnter}
      onBlur={onLeave}
    >
      <div
        aria-hidden
        className="absolute inset-0 transition-opacity duration-500"
        style={{
          backgroundImage: `url('${poster}')`,
          backgroundSize: 'cover',
          backgroundPosition: 'center',
          opacity: idleOpacity,
        }}
      />
      {video && shouldMount && (
        <video
          ref={videoRef}
          aria-hidden
          muted
          playsInline
          loop
          preload="metadata"
          poster={poster}
          disablePictureInPicture
          disableRemotePlayback
          className="absolute inset-0 h-full w-full object-cover transition-opacity duration-500"
          style={{ opacity: 0 }}
          onPlaying={(e) => {
            (e.currentTarget as HTMLVideoElement).style.opacity = String(activeOpacity);
          }}
          onPause={(e) => {
            (e.currentTarget as HTMLVideoElement).style.opacity = '0';
          }}
        >
          <source src={video} type="video/mp4" />
        </video>
      )}
      {overlay && <div aria-hidden className={overlay} />}
    </div>
  );
}
