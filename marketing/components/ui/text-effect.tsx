'use client';

import { TextAnimate } from './text-animate';

type Props = {
  children: string;
  as?: 'h1' | 'h2' | 'h3' | 'p' | 'span';
  className?: string;
  delay?: number;
  stagger?: number;
  triggerOnView?: boolean;
};

/**
 * Backwards-compat wrapper. Existing callsites keep working; new code should use
 * TextAnimate directly. Maps to a clean slide-up + fade by word.
 */
export function TextEffect({
  children,
  as = 'h2',
  className,
  delay = 0,
  triggerOnView = true,
}: Props) {
  return (
    <TextAnimate
      as={as}
      animation="slideUp"
      by="word"
      duration={0.5}
      delay={delay}
      startOnView={triggerOnView}
      className={className}
    >
      {children}
    </TextAnimate>
  );
}
