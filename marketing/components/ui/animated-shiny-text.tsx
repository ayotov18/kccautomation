import {
  type ComponentPropsWithoutRef,
  type CSSProperties,
  type FC,
} from 'react';

import { cn } from '@/lib/cn';

export interface AnimatedShinyTextProps extends ComponentPropsWithoutRef<'span'> {
  shimmerWidth?: number;
}

export const AnimatedShinyText: FC<AnimatedShinyTextProps> = ({
  children,
  className,
  shimmerWidth = 100,
  ...props
}) => {
  return (
    <span
      style={{ '--shiny-width': `${shimmerWidth}px` } as CSSProperties}
      className={cn(
        'inline-block text-[var(--color-fg-tertiary)]',
        'animate-shiny-text bg-clip-text bg-no-repeat',
        '[background-position:0_0] [background-size:var(--shiny-width)_100%]',
        '[transition:background-position_1s_cubic-bezier(.6,.6,0,1)_infinite]',
        'bg-[linear-gradient(to_right,transparent,var(--color-fg)_50%,transparent)]',
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
};
