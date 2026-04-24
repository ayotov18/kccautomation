'use client';

import { motion, useInView, useMotionValue, useSpring, useTransform } from 'motion/react';
import { useEffect, useRef } from 'react';
import { cn } from '@/lib/cn';

type Props = {
  value: number;
  from?: number;
  duration?: number;
  className?: string;
  suffix?: string;
  prefix?: string;
  decimals?: number;
};

export function NumberTicker({
  value,
  from = 0,
  className,
  suffix,
  prefix,
  decimals = 0,
}: Props) {
  const ref = useRef<HTMLSpanElement | null>(null);
  const mv = useMotionValue(from);
  const sv = useSpring(mv, { stiffness: 90, damping: 24, mass: 1 });
  const display = useTransform(sv, (n) => {
    const f = decimals === 0 ? Math.round(n) : Number(n.toFixed(decimals));
    return `${prefix ?? ''}${f.toLocaleString('en-US')}${suffix ?? ''}`;
  });

  const inView = useInView(ref, { once: true, margin: '-80px' });

  useEffect(() => {
    if (inView) mv.set(value);
  }, [inView, value, mv]);

  return (
    <motion.span ref={ref} className={cn('tabular-nums', className)}>
      {display}
    </motion.span>
  );
}
