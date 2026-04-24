'use client';

import { motion, type Transition } from 'motion/react';
import { cn } from '@/lib/cn';

type Props = {
  children: string;
  as?: 'h1' | 'h2' | 'h3' | 'p' | 'span';
  className?: string;
  delay?: number;
  stagger?: number;
  by?: 'word' | 'char';
};

const spring: Transition = { type: 'spring', stiffness: 220, damping: 26, mass: 0.8 };

export function TextEffect({
  children,
  as: Tag = 'h1',
  className,
  delay = 0,
  stagger = 0.055,
  by = 'word',
}: Props) {
  const units = by === 'word' ? children.split(/(\s+)/) : Array.from(children);

  return (
    <Tag className={cn('inline-block', className)}>
      <span aria-hidden className="inline-block">
        {units.map((unit, i) => {
          if (unit === ' ' || unit.trim() === '') return <span key={i}>&nbsp;</span>;
          return (
            <motion.span
              key={i}
              initial={{ y: '120%', opacity: 0 }}
              animate={{ y: '0%', opacity: 1 }}
              transition={{ ...spring, delay: delay + i * stagger }}
              style={{ display: 'inline-block', willChange: 'transform' }}
            >
              {unit}
              {by === 'word' && i < units.length - 1 && ' '}
            </motion.span>
          );
        })}
      </span>
      <span className="sr-only">{children}</span>
    </Tag>
  );
}
