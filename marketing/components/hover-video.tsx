'use client';

import { useCallback, useRef } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  poster: string;
  video?: string | null;
  className?: string;
  /** Opacity of the video when idle */
  idleOpacity?: number;
  /** Opacity when hover/visible playing */
  activeOpacity?: number;
  overlay?: string;
};

export function HoverVideo({
  poster,
  video,
  className,
  idleOpacity = 0.7,
  activeOpacity = 0.95,
  overlay,
}: Props) {
  const videoRef = useRef<HTMLVideoElement | null>(null);

  const onEnter = useCallback(() => {
    const v = videoRef.current;
    if (!v) return;
    v.currentTime = 0;
    v.play().catch(() => {});
  }, []);

  const onLeave = useCallback(() => {
    const v = videoRef.current;
    if (!v) return;
    v.pause();
  }, []);

  return (
    <div
      className={cn('absolute inset-0', className)}
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      onFocus={onEnter}
      onBlur={onLeave}
    >
      {/* Poster image — always shown, fades slightly when video plays */}
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
      {video && (
        <video
          ref={videoRef}
          aria-hidden
          muted
          playsInline
          loop
          preload="metadata"
          poster={poster}
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
