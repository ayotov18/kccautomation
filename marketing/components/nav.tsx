'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';
import { cn } from '@/lib/cn';

export function Nav() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  return (
    <header
      className={cn(
        'fixed inset-x-0 top-0 z-50 transition-all duration-300',
        scrolled
          ? 'backdrop-blur-xl bg-[var(--color-bg)]/70 border-b border-[var(--color-hairline)]'
          : 'bg-transparent',
      )}
    >
      <nav className="mx-auto max-w-7xl px-6 py-4 flex items-center justify-between">
        <Link
          href="/"
          className="flex items-center gap-2 text-[15px] font-semibold tracking-tight"
        >
          <span className="inline-block h-5 w-5 rounded-[4px] bg-[var(--color-amber)]/90 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.2)]" />
          KCC Automation
        </Link>
        <div className="flex items-center gap-2">
          <a
            href="#stack"
            className="hidden sm:inline-flex h-9 px-3 items-center text-[13px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
          >
            Stack
          </a>
          <a
            href="#faq"
            className="hidden sm:inline-flex h-9 px-3 items-center text-[13px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
          >
            FAQ
          </a>
          <a
            href="https://app.kccgen.xyz"
            className="hidden sm:inline-flex h-9 px-3 items-center text-[13px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
          >
            Log in
          </a>
          <a
            href="#cta"
            className="inline-flex h-9 px-4 items-center rounded-md bg-[var(--color-amber)] text-[var(--color-bg)] text-[13px] font-medium hover:bg-[var(--color-amber-soft)] transition-colors"
          >
            Request access
          </a>
        </div>
      </nav>
    </header>
  );
}
