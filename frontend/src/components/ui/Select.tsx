'use client';

/**
 * KCC <Select>. Wraps Radix Select.
 *
 * Trigger: filled card-tone surface, hairline border, amber focus ring.
 * Popover: liquid-glass (backdrop blur + saturate) with a solid fallback,
 *   140ms scale-fade animation, portal-mounted so it escapes overflow.
 * Items: 32px rows with amber-tint + check indicator when selected.
 *
 * Use this everywhere instead of native <select>.
 */

import * as RadixSelect from '@radix-ui/react-select';
import { Check, ChevronDown } from 'lucide-react';
import { clsx } from 'clsx';
import { forwardRef, type ReactNode } from 'react';

export interface SelectOption {
  value: string;
  label: string;
  hint?: string;
  disabled?: boolean;
}

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  placeholder?: string;
  className?: string;
  disabled?: boolean;
  size?: 'sm' | 'md';
  ariaLabel?: string;
  /** Width of the popover. 'trigger' matches the trigger; otherwise CSS value. */
  contentWidth?: 'trigger' | string;
}

export function Select({
  value,
  onChange,
  options,
  placeholder = 'Select…',
  className,
  disabled,
  size = 'md',
  ariaLabel,
  contentWidth = 'trigger',
}: SelectProps) {
  return (
    <RadixSelect.Root value={value} onValueChange={onChange} disabled={disabled}>
      <RadixSelect.Trigger
        aria-label={ariaLabel}
        className={clsx('kcc-select-trigger', size === 'sm' && 'kcc-select-trigger-sm', className)}
      >
        <RadixSelect.Value placeholder={<span className="text-content-tertiary">{placeholder}</span>} />
        <RadixSelect.Icon>
          <ChevronDown size={14} className="text-content-tertiary" strokeWidth={2} />
        </RadixSelect.Icon>
      </RadixSelect.Trigger>

      <RadixSelect.Portal>
        <RadixSelect.Content
          position="popper"
          sideOffset={6}
          className="kcc-select-content"
          style={
            contentWidth === 'trigger'
              ? ({ ['--radix-select-content-width' as string]: 'var(--radix-select-trigger-width)' } as React.CSSProperties)
              : ({ width: contentWidth } as React.CSSProperties)
          }
        >
          <RadixSelect.Viewport className="p-1.5">
            {options.map((opt) => (
              <SelectItem key={opt.value} value={opt.value} disabled={opt.disabled} hint={opt.hint}>
                {opt.label}
              </SelectItem>
            ))}
          </RadixSelect.Viewport>
        </RadixSelect.Content>
      </RadixSelect.Portal>
    </RadixSelect.Root>
  );
}

const SelectItem = forwardRef<
  HTMLDivElement,
  { value: string; disabled?: boolean; hint?: string; children: ReactNode }
>(function SelectItem({ value, disabled, hint, children }, ref) {
  return (
    <RadixSelect.Item
      ref={ref}
      value={value}
      disabled={disabled}
      className="kcc-select-item"
    >
      <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
      {hint && <span className="kcc-select-item-hint">{hint}</span>}
      <RadixSelect.ItemIndicator className="kcc-select-item-check">
        <Check size={12} strokeWidth={2.4} />
      </RadixSelect.ItemIndicator>
    </RadixSelect.Item>
  );
});
