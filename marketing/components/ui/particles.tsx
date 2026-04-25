'use client';

import { useEffect, useRef } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  quantity?: number;
  color?: string;
  size?: number;
  staticity?: number;
  ease?: number;
  className?: string;
};

/**
 * Dust-style amber particles. Cheap canvas loop.
 * Visibility-gated: rAF only runs when the canvas is on-screen.
 * Skipped entirely under prefers-reduced-motion.
 */
export function Particles({
  quantity = 60,
  color = 'oklch(0.72 0.16 55)',
  size = 0.8,
  staticity = 35,
  ease = 42,
  className,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const visibleRef = useRef(false);
  const rafRef = useRef(0);

  type Particle = {
    x: number;
    y: number;
    translateX: number;
    translateY: number;
    size: number;
    alpha: number;
    targetAlpha: number;
    dx: number;
    dy: number;
    magnetism: number;
  };
  const particlesRef = useRef<Particle[]>([]);
  const mouse = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const container = containerRef.current;
    const canvas = canvasRef.current;
    if (!container || !canvas) return;

    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const count = quantity;

    function spawn(): Particle {
      const rect = container!.getBoundingClientRect();
      return {
        x: Math.random() * rect.width,
        y: Math.random() * rect.height,
        translateX: 0,
        translateY: 0,
        size: Math.random() * 2 + size,
        alpha: 0,
        targetAlpha: Math.random() * 0.5 + 0.1,
        dx: (Math.random() - 0.5) * 0.1,
        dy: (Math.random() - 0.5) * 0.1,
        magnetism: 0.1 + Math.random() * 4,
      };
    }

    function init() {
      particlesRef.current = Array.from({ length: count }, spawn);
    }

    function resize() {
      if (!container || !canvas) return;
      const rect = container.getBoundingClientRect();
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = rect.width + 'px';
      canvas.style.height = rect.height + 'px';
      ctx!.scale(dpr, dpr);
    }

    function draw() {
      if (!ctx || !canvas || !container) return;
      const rect = container.getBoundingClientRect();
      ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);
      for (const p of particlesRef.current) {
        p.x += p.dx;
        p.y += p.dy;
        p.alpha += (p.targetAlpha - p.alpha) * 0.02;
        p.translateX += (mouse.current.x / (staticity / p.magnetism) - p.translateX) / ease;
        p.translateY += (mouse.current.y / (staticity / p.magnetism) - p.translateY) / ease;
        if (p.x < 0 || p.x > rect.width || p.y < 0 || p.y > rect.height) {
          Object.assign(p, spawn());
        }
        ctx.translate(p.translateX, p.translateY);
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.globalAlpha = p.alpha;
        ctx.fill();
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      }
    }

    function tick() {
      if (!visibleRef.current) return;
      draw();
      rafRef.current = requestAnimationFrame(tick);
    }

    function onMouse(e: MouseEvent) {
      if (!container) return;
      const rect = container.getBoundingClientRect();
      mouse.current.x = e.clientX - rect.left - rect.width / 2;
      mouse.current.y = e.clientY - rect.top - rect.height / 2;
    }

    function onResize() {
      resize();
      init();
    }

    resize();
    init();

    const ro = new ResizeObserver(onResize);
    ro.observe(container);
    window.addEventListener('mousemove', onMouse);

    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting && !visibleRef.current) {
            visibleRef.current = true;
            tick();
          } else if (!e.isIntersecting && visibleRef.current) {
            visibleRef.current = false;
            cancelAnimationFrame(rafRef.current);
            rafRef.current = 0;
          }
        }
      },
      { threshold: 0 },
    );
    io.observe(container);

    return () => {
      cancelAnimationFrame(rafRef.current);
      window.removeEventListener('mousemove', onMouse);
      ro.disconnect();
      io.disconnect();
    };
  }, [quantity, color, size, staticity, ease]);

  return (
    <div ref={containerRef} aria-hidden className={cn('pointer-events-none absolute inset-0', className)}>
      <canvas ref={canvasRef} />
    </div>
  );
}
