'use client';

import { motion } from 'motion/react';
import { cn } from '@/lib/cn';

type Props = {
  children: string;
  as?: 'h1' | 'h2' | 'h3' | 'p' | 'span';
  className?: string;
  delay?: number;
  stagger?: number;
  triggerOnView?: boolean;
};

/**
 * Canonical word-by-word reveal.
 * - per-word CLIP wrapper (overflow:hidden, display:inline-block, align-bottom)
 * - padding-bottom on clip to accommodate descenders
 * - NBSP between words so flex gaps don't break spacing
 * - text-wrap:balance on the outer tag to prevent orphan words
 */
export function TextEffect({
  children,
  as: Tag = 'h1',
  className,
  delay = 0,
  stagger = 0.05,
  triggerOnView = false,
}: Props) {
  const words = children.split(' ');

  const word = (w: string, i: number) => (
    <span
      key={i}
      aria-hidden
      className="inline-block overflow-hidden align-bottom leading-[1.05] pb-[0.18em]"
      style={{ verticalAlign: 'bottom' }}
    >
      <motion.span
        className="inline-block will-change-transform"
        initial={{ y: '115%' }}
        {...(triggerOnView
          ? { whileInView: { y: '0%' }, viewport: { once: true, margin: '-10%' } }
          : { animate: { y: '0%' } })}
        transition={{
          duration: 0.75,
          ease: [0.22, 1, 0.36, 1],
          delay: delay + i * stagger,
        }}
      >
        {w}
      </motion.span>
      {i < words.length - 1 && ' '}
    </span>
  );

  return (
    <Tag
      className={cn('block text-balance', className)}
      style={{ textWrap: 'balance' }}
      aria-label={children}
    >
      {words.map(word)}
    </Tag>
  );
}
