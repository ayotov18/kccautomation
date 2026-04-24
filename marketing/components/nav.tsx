'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';
import { motion, useScroll, useTransform } from 'motion/react';
import { Magnetic } from './ui/magnetic';
import { BorderBeam } from './ui/border-beam';
import { cn } from '@/lib/cn';

const LINKS = [
  { href: '#features', label: 'Features' },
  { href: '#pipeline', label: 'Pipeline' },
  { href: '#stack', label: 'Stack' },
  { href: '#faq', label: 'FAQ' },
];

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const { scrollY } = useScroll();

  const width = useTransform(scrollY, [0, 120], ['min(960px, 94vw)', 'min(760px, 92vw)']);
  const padY = useTransform(scrollY, [0, 120], [10, 6]);

  useEffect(() => {
    return scrollY.on('change', (v) => setScrolled(v > 24));
  }, [scrollY]);

  return (
    <motion.header
      initial={{ y: -40, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.6, ease: [0.22, 0.61, 0.36, 1], delay: 0.2 }}
      style={{ width, paddingTop: padY, paddingBottom: padY }}
      className="fixed left-1/2 -translate-x-1/2 top-3 z-[60]"
    >
      <motion.nav
        animate={{
          backgroundColor: scrolled ? 'oklch(0.14 0.01 260 / 0.72)' : 'oklch(0.14 0.01 260 / 0.3)',
          borderColor: scrolled ? 'oklch(0.32 0.012 260 / 0.7)' : 'oklch(0.32 0.012 260 / 0.3)',
          boxShadow: scrolled
            ? '0 0 0 1px oklch(0.72 0.16 55 / 0.12), 0 18px 60px -20px oklch(0.72 0.16 55 / 0.22), 0 0 0 0.5px oklch(0.32 0.012 260 / 0.6)'
            : '0 0 0 0 transparent',
        }}
        transition={{ type: 'spring', stiffness: 220, damping: 26 }}
        className={cn(
          'relative mx-auto flex items-center justify-between gap-2 rounded-full px-3 md:px-4 backdrop-blur-2xl border overflow-hidden',
        )}
      >
        {scrolled && <BorderBeam size={140} duration={10} colorFrom="oklch(0.82 0.19 62 / 0.9)" colorTo="oklch(0.72 0.16 55 / 0)" />}

        <Link
          href="/"
          className="flex items-center gap-2.5 pl-2 pr-1 py-1.5 text-[14px] font-semibold tracking-tight"
        >
          <span className="relative inline-block h-6 w-6 rounded-full bg-[var(--color-amber)]/90 shadow-[inset_0_0_0_1px_rgba(255,255,255,0.2),0_0_12px_oklch(0.72_0.16_55/0.6)]">
            <span className="absolute inset-[3px] rounded-full bg-[var(--color-bg)]" />
            <span className="absolute inset-[6px] rounded-full bg-[var(--color-amber)]" />
          </span>
          <span>KCC</span>
        </Link>

        <ul className="hidden md:flex items-center gap-1 mx-auto">
          {LINKS.map((l) => (
            <li key={l.href}>
              <a
                href={l.href}
                className="inline-flex h-8 items-center rounded-full px-3.5 text-[12.5px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] hover:bg-[var(--color-surface)]/80 transition-colors"
              >
                {l.label}
              </a>
            </li>
          ))}
        </ul>

        <div className="flex items-center gap-1">
          <a
            href="https://app.kccgen.xyz"
            className="hidden sm:inline-flex h-8 items-center rounded-full px-3.5 text-[12.5px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
          >
            Log in
          </a>
          <Magnetic intensity={0.25}>
            <a
              href="#cta"
              className="relative inline-flex h-8 items-center gap-1.5 rounded-full bg-[var(--color-amber)] px-4 text-[12.5px] font-medium text-[var(--color-bg)] hover:bg-[var(--color-amber-hot)] transition-colors amber-glow"
            >
              Request access
              <span
                aria-hidden
                className="inline-block h-1 w-1 rounded-full bg-[var(--color-bg)]/50"
              />
            </a>
          </Magnetic>
        </div>
      </motion.nav>
    </motion.header>
  );
}
