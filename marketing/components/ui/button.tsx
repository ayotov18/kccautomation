import { cn } from '@/lib/cn';
import { cva, type VariantProps } from 'class-variance-authority';
import { forwardRef } from 'react';

const buttonStyles = cva(
  'inline-flex items-center justify-center gap-2 font-medium transition-all whitespace-nowrap disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-amber)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--color-bg)]',
  {
    variants: {
      variant: {
        primary:
          'bg-[var(--color-amber)] text-[var(--color-bg)] hover:bg-[var(--color-amber-soft)] shadow-[0_0_0_1px_rgba(184,115,51,0.2),0_8px_20px_rgba(184,115,51,0.25)]',
        secondary:
          'bg-[var(--color-surface)] text-[var(--color-fg)] border border-[var(--color-hairline-hi)] hover:bg-[var(--color-surface-hi)]',
        ghost:
          'bg-transparent text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)]',
      },
      size: {
        sm: 'h-8 px-3 text-[13px] rounded-md',
        md: 'h-10 px-5 text-[14px] rounded-lg',
        lg: 'h-12 px-6 text-[15px] rounded-lg',
      },
    },
    defaultVariants: { variant: 'primary', size: 'md' },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonStyles> {
  asChild?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => (
    <button ref={ref} className={cn(buttonStyles({ variant, size }), className)} {...props} />
  ),
);

Button.displayName = 'Button';
