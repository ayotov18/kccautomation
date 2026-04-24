'use client';

import { type LucideIcon, Inbox } from 'lucide-react';

interface Props {
  icon?: LucideIcon;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
}

export function EmptyState({ icon: Icon = Inbox, title, description, actionLabel, onAction }: Props) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-4 text-center">
      <div className="w-14 h-14 rounded-2xl flex items-center justify-center mb-4" style={{ background: 'var(--oe-bg-tertiary)' }}>
        <Icon className="w-7 h-7" style={{ color: 'var(--oe-text-tertiary)' }} />
      </div>
      <h3 className="text-lg font-semibold mb-1" style={{ color: 'var(--oe-text-primary)' }}>{title}</h3>
      {description && <p className="text-sm max-w-sm" style={{ color: 'var(--oe-text-secondary)' }}>{description}</p>}
      {actionLabel && onAction && (
        <button onClick={onAction} className="mt-4 px-4 py-2 rounded-lg text-sm font-medium text-white" style={{ background: 'var(--oe-blue)' }}>
          {actionLabel}
        </button>
      )}
    </div>
  );
}
