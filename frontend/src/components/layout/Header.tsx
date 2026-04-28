'use client';

import { useState, useRef, useEffect } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { clsx } from 'clsx';
import {
  Search,
  Sun,
  Moon,
  User,
  LogOut,
  Menu,
  ChevronRight,
} from 'lucide-react';
import { useThemeStore } from '@/lib/themeStore';
import { useAuthStore } from '@/lib/store';

interface HeaderProps {
  onMobileMenuToggle: () => void;
}

/* ---- Breadcrumb generation ---- */

interface Crumb {
  label: string;
  path?: string;
}

const routeLabels: Record<string, string> = {
  '': 'Dashboard',
  dashboard: 'Dashboard',
  projects: 'Projects',
  boq: 'BOQ Editor',
  drawings: 'Drawings',
  upload: 'Upload',
  viewer: 'Viewer',
  kss: 'KCC Report',
  prepare: 'AI Prepare',
  report: 'Report',
  costs: 'Cost Database',
  assemblies: 'Assemblies',
  validation: 'Validation',
  schedule: '4D Schedule',
  costmodel: '5D Cost Model',
  tendering: 'Tendering',
  cde: 'CDE',
  prices: 'Price Management',
  settings: 'Settings',
  login: 'Login',
  'drm-stats': 'DRM Stats',
};

function buildBreadcrumbs(pathname: string): Crumb[] {
  const segments = pathname.split('/').filter(Boolean);

  if (segments.length === 0) {
    return [{ label: 'Dashboard' }];
  }

  const crumbs: Crumb[] = [{ label: 'Dashboard', path: '/dashboard' }];
  let path = '';

  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i];
    path += `/${seg}`;
    const label = routeLabels[seg] ?? seg;
    const isLast = i === segments.length - 1;
    crumbs.push({ label, path: isLast ? undefined : path });
  }

  return crumbs;
}

/* ---- User dropdown ---- */

function UserMenu() {
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const logout = useAuthStore((s) => s.logout);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    if (open) document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [open]);

  return (
    <div ref={menuRef} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className={clsx(
          'flex items-center justify-center h-8 w-8 rounded-full',
          'bg-[var(--oe-bg-tertiary)] text-[var(--oe-text-secondary)]',
          'hover:bg-[var(--oe-border)] transition-colors',
        )}
        title="User menu"
      >
        <User size={16} />
      </button>

      {open && (
        <div
          className={clsx(
            'absolute right-0 top-full mt-2 w-56 rounded-xl overflow-hidden',
            'bg-[var(--oe-bg-elevated)] border border-[var(--oe-border)]',
            'shadow-[var(--oe-shadow-lg)] z-50',
            'oe-fade-in',
          )}
        >
          <div className="py-1">
            <Link
              href="/settings"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2 px-4 py-2 text-sm text-[var(--oe-text-secondary)] hover:bg-[var(--oe-bg-secondary)] transition-colors no-underline"
            >
              <User size={14} />
              Account Settings
            </Link>
            <button
              onClick={() => {
                setOpen(false);
                logout();
              }}
              className="flex items-center gap-2 w-full px-4 py-2 text-sm text-[var(--oe-error)] hover:bg-[var(--oe-error-bg)] transition-colors"
            >
              <LogOut size={14} />
              Sign Out
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

/* ---- Main Header ---- */

export function Header({ onMobileMenuToggle }: HeaderProps) {
  const pathname = usePathname();
  const crumbs = buildBreadcrumbs(pathname);
  const resolved = useThemeStore((s) => s.resolved);
  const toggle = useThemeStore((s) => s.toggle);

  return (
    <header
      className={clsx(
        'h-[var(--oe-header-height)] flex items-center justify-between px-4 lg:px-6',
        'bg-[var(--oe-header-bg)] backdrop-blur-xl border-b border-border-light/50',
        'flex-shrink-0',
      )}
    >
      {/* Left: mobile hamburger + breadcrumbs */}
      <div className="flex items-center gap-3 min-w-0">
        <button
          onClick={onMobileMenuToggle}
          className="lg:hidden flex items-center justify-center h-8 w-8 rounded-lg hover:bg-[var(--oe-bg-secondary)] transition-colors"
        >
          <Menu size={18} className="text-[var(--oe-text-secondary)]" />
        </button>

        <nav className="flex items-center gap-1 min-w-0">
          {crumbs.map((crumb, idx) => (
            <span key={idx} className="flex items-center gap-1 min-w-0">
              {idx > 0 && (
                <ChevronRight
                  size={12}
                  className="flex-shrink-0 text-[var(--oe-text-tertiary)]"
                />
              )}
              {crumb.path ? (
                <Link
                  href={crumb.path}
                  className="text-sm text-[var(--oe-text-secondary)] hover:text-[var(--oe-text-primary)] truncate transition-colors no-underline"
                >
                  {crumb.label}
                </Link>
              ) : (
                <span className="text-sm font-medium text-[var(--oe-text-primary)] truncate">
                  {crumb.label}
                </span>
              )}
            </span>
          ))}
        </nav>
      </div>

      {/* Right: search, theme, user */}
      <div className="flex items-center gap-2">
        {/* Search (visual placeholder) */}
        <div
          className={clsx(
            'hidden md:flex items-center gap-2 h-8 px-3 rounded-lg',
            'bg-[var(--oe-bg-secondary)] border border-[var(--oe-border)]',
            'text-[var(--oe-text-tertiary)] text-sm cursor-pointer',
            'hover:border-[var(--oe-text-tertiary)] transition-colors',
          )}
        >
          <Search size={14} />
          <span>Search...</span>
          <kbd className="hidden lg:inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded bg-[var(--oe-bg-tertiary)] text-[var(--oe-text-tertiary)]">
            <span className="text-xs">&#x2318;</span>K
          </kbd>
        </div>

        {/* Theme toggle */}
        <button
          onClick={toggle}
          className={clsx(
            'flex items-center justify-center h-8 w-8 rounded-lg',
            'hover:bg-[var(--oe-bg-secondary)] transition-colors',
            'text-[var(--oe-text-secondary)]',
          )}
          title={resolved === 'light' ? 'Switch to dark mode' : 'Switch to light mode'}
        >
          {resolved === 'light' ? <Moon size={16} /> : <Sun size={16} />}
        </button>

        {/* User menu */}
        <UserMenu />
      </div>
    </header>
  );
}
