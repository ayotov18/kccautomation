'use client';

import {
  useRef,
  type ComponentPropsWithoutRef,
  type FC,
  type ReactNode,
} from 'react';
import { motion, MotionValue, useScroll, useTransform } from 'motion/react';

import { cn } from '@/lib/cn';

export interface TextRevealProps extends ComponentPropsWithoutRef<'div'> {
  children: string;
}

export const TextReveal: FC<TextRevealProps> = ({ children, className }) => {
  const sectionRef = useRef<HTMLDivElement | null>(null);
  const { scrollYProgress } = useScroll({ target: sectionRef });

  if (typeof children !== 'string') {
    throw new Error('TextReveal: children must be a string');
  }

  const words = children.split(' ');

  return (
    <div ref={sectionRef} className={cn('relative z-0 h-[150vh]', className)}>
      <div className="sticky top-0 mx-auto flex h-[60svh] max-w-5xl items-center bg-transparent">
        <span className="flex flex-wrap text-[length:var(--text-3xl)] md:text-[length:var(--text-4xl)] leading-[1.04] tracking-[-0.025em] font-medium text-[var(--color-fg-quaternary)]">
          {words.map((word, i) => {
            const start = i / words.length;
            const end = start + 1 / words.length;
            return (
              <Word key={i} progress={scrollYProgress} range={[start, end]}>
                {word}
              </Word>
            );
          })}
        </span>
      </div>
    </div>
  );
};

interface WordProps {
  children: ReactNode;
  progress: MotionValue<number>;
  range: [number, number];
}

const Word: FC<WordProps> = ({ children, progress, range }) => {
  const opacity = useTransform(progress, range, [0, 1]);
  return (
    <span className="relative mx-1 lg:mx-1.5">
      <span className="absolute opacity-30">{children}</span>
      <motion.span style={{ opacity }} className="text-[var(--color-fg)]">
        {children}
      </motion.span>
    </span>
  );
};
