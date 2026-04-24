'use client';

import { motion, useMotionValue, useSpring, useTransform } from 'motion/react';
import { useRef, type ReactNode } from 'react';

type Props = {
  children: ReactNode;
  intensity?: number;
  className?: string;
};

export function Magnetic({ children, intensity = 0.3, className }: Props) {
  const ref = useRef<HTMLDivElement | null>(null);
  const mx = useMotionValue(0);
  const my = useMotionValue(0);

  const springX = useSpring(mx, { stiffness: 220, damping: 18, mass: 0.6 });
  const springY = useSpring(my, { stiffness: 220, damping: 18, mass: 0.6 });

  const x = useTransform(springX, (v) => v * intensity);
  const y = useTransform(springY, (v) => v * intensity);

  function onMove(e: React.MouseEvent<HTMLDivElement>) {
    const el = ref.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    mx.set(e.clientX - rect.left - rect.width / 2);
    my.set(e.clientY - rect.top - rect.height / 2);
  }
  function onLeave() {
    mx.set(0);
    my.set(0);
  }

  return (
    <motion.div
      ref={ref}
      onMouseMove={onMove}
      onMouseLeave={onLeave}
      style={{ x, y }}
      className={className}
    >
      {children}
    </motion.div>
  );
}
