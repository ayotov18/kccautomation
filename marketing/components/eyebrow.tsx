import { cn } from '@/lib/cn';

export function Eyebrow({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        'font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.16em] text-[var(--color-fg-tertiary)]',
        className,
      )}
    >
      {children}
    </span>
  );
}
