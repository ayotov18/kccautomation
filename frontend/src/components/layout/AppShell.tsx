'use client';

import { useEffect } from 'react';
import { clsx } from 'clsx';
import { FloatingBreadcrumb } from './FloatingBreadcrumb';
import { FloatingCommandBar } from './FloatingCommandBar';
import { FloatingUserMenu } from './FloatingUserMenu';
import { ToastContainer } from '@/components/ui/Toast';
import { useThemeStore } from '@/lib/themeStore';

interface AppShellProps {
  children: React.ReactNode;
}

/**
 * Sidebar-less layout. Three floating pills (breadcrumb · user menu · command
 * bar) sit on top of a full-width content area. Content reserves top/bottom
 * padding so the floating chrome never occludes it.
 */
export function AppShell({ children }: AppShellProps) {
  const initTheme = useThemeStore((s) => s.init);

  useEffect(() => {
    initTheme();
  }, [initTheme]);

  return (
    <div className="relative h-screen overflow-hidden" style={{ zIndex: 2 }}>
      <main
        className={clsx(
          'h-full overflow-y-auto bg-transparent',
          // Reserve space so the floating pills never cover content.
          'pt-[72px] pb-[104px]',
        )}
      >
        {children}
      </main>

      <FloatingBreadcrumb />
      <FloatingUserMenu />
      <FloatingCommandBar />

      <ToastContainer />
    </div>
  );
}
