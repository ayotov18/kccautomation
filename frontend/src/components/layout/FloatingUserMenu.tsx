'use client';

import { useEffect, useRef, useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  Home,
  Settings as SettingsIcon,
  LogOut,
  Moon,
  Sun,
  Keyboard,
  HelpCircle,
  User,
} from 'lucide-react';
import { clsx } from 'clsx';
import { useAuthStore } from '@/lib/store';
import { useThemeStore } from '@/lib/themeStore';

/**
 * Floating profile / home pill — top-LEFT.
 *
 * Layout (all circles, zero edges):
 *   [ HOME avatar ]  [ settings gear ]  [ theme ]
 *
 * - HOME avatar (primary): click → go to /dashboard. Long-press or hover
 *   reveals a small tooltip "Начало".
 * - Settings gear: click → opens the rounded dropdown (profile, shortcuts,
 *   help, logout).
 * - Theme: click → toggle light/dark.
 */
export function FloatingUserMenu() {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const logout = useAuthStore((s) => s.logout);
  const resolved = useThemeStore((s) => s.resolved);
  const toggleTheme = useThemeStore((s) => s.toggle);

  useEffect(() => {
    if (!open) return;
    const onClick = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(false); };
    document.addEventListener('mousedown', onClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [open]);

  return (
    <div ref={rootRef} className="fixed top-4 left-4 z-40">
      <div className="kcc-floating-surface flex items-center gap-1 p-1">
        {/* HOME — primary round action. Clicking the avatar goes home. */}
        <button
          onClick={() => router.push('/dashboard')}
          title="Начало"
          aria-label="Начало"
          className="group w-10 h-10 flex items-center justify-center rounded-full transition-all duration-200 hover:scale-[1.03] active:scale-95"
          style={{
            background: 'var(--oe-accent-soft-bg)',
            color: 'var(--oe-accent)',
          }}
        >
          <Home size={16} strokeWidth={2.25} className="transition-transform group-hover:-rotate-6" />
        </button>

        {/* Settings dropdown trigger */}
        <button
          onClick={() => setOpen((v) => !v)}
          title="Настройки"
          aria-label="Меню на потребителя"
          aria-expanded={open}
          className={clsx(
            'w-9 h-9 flex items-center justify-center rounded-full transition-colors',
            open
              ? 'bg-white/10 text-content-primary'
              : 'text-content-secondary hover:bg-white/5 hover:text-content-primary',
          )}
        >
          <SettingsIcon size={15} />
        </button>

        {/* Theme toggle */}
        <button
          onClick={toggleTheme}
          title={resolved === 'light' ? 'Тъмен режим' : 'Светъл режим'}
          className="w-9 h-9 flex items-center justify-center rounded-full text-content-secondary hover:bg-white/5 hover:text-content-primary transition-colors"
        >
          {resolved === 'light' ? <Moon size={15} /> : <Sun size={15} />}
        </button>
      </div>

      {open && (
        <div className="kcc-floating-surface kcc-floating-panel absolute left-0 top-full mt-2 w-60 p-1.5 oe-fade-in">
          <MenuItem
            icon={<User size={14} />}
            label="Профил"
            sub="Твоят акаунт"
            href="/settings"
            onSelect={() => setOpen(false)}
          />
          <MenuItem
            icon={<SettingsIcon size={14} />}
            label="Настройки"
            sub="Предпочитания, тема"
            href="/settings"
            onSelect={() => setOpen(false)}
          />
          <MenuItem
            icon={<Keyboard size={14} />}
            label="Клавишни комбинации"
            sub="⌘K за бързо меню"
            onSelect={() => {
              setOpen(false);
              const e = new KeyboardEvent('keydown', { key: 'k', metaKey: true });
              window.dispatchEvent(e);
            }}
          />
          <MenuItem
            icon={<HelpCircle size={14} />}
            label="Помощ"
            href="/help"
            onSelect={() => setOpen(false)}
          />

          <div className="h-px bg-white/5 my-1.5" />

          <MenuItem
            icon={<LogOut size={14} />}
            label="Изход"
            onSelect={() => { setOpen(false); logout(); }}
            danger
          />
        </div>
      )}
    </div>
  );
}

function MenuItem({
  icon,
  label,
  sub,
  href,
  onSelect,
  danger,
}: {
  icon: React.ReactNode;
  label: string;
  sub?: string;
  href?: string;
  onSelect: () => void;
  danger?: boolean;
}) {
  const cls = clsx(
    'flex items-center gap-2.5 w-full px-2.5 py-2 rounded-xl text-left transition-colors',
    danger
      ? 'text-red-400 hover:bg-red-500/10'
      : 'text-content-secondary hover:text-content-primary hover:bg-white/5',
  );
  const body = (
    <>
      <span
        className={clsx(
          'w-7 h-7 rounded-full flex items-center justify-center flex-none',
          danger ? 'bg-red-500/10' : 'bg-white/5',
        )}
      >
        {icon}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-sm truncate">{label}</span>
        {sub && <span className="block text-[11px] text-content-tertiary truncate">{sub}</span>}
      </span>
    </>
  );
  if (href) {
    return (
      <Link href={href} onClick={onSelect} className={cls}>
        {body}
      </Link>
    );
  }
  return (
    <button type="button" onClick={onSelect} className={cls}>
      {body}
    </button>
  );
}
