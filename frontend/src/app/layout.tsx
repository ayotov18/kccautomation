import type { Metadata } from 'next';
import './globals.css';
import { ClientShell } from './client-shell';

export const metadata: Metadata = {
  title: 'KCC Automation',
  description: 'Construction ERP — KCC report generation from engineering drawings',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen" style={{ background: 'var(--oe-bg-primary)', color: 'var(--oe-text-primary)' }}>
        <ClientShell>{children}</ClientShell>
      </body>
    </html>
  );
}
