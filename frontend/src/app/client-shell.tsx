'use client';

import { usePathname } from 'next/navigation';
import { AppShell } from '@/components/layout/AppShell';

const NO_SHELL_ROUTES = ['/']; // login page has its own layout

export function ClientShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();

  // Login page doesn't use the shell
  if (NO_SHELL_ROUTES.includes(pathname)) {
    return <>{children}</>;
  }

  return <AppShell>{children}</AppShell>;
}
