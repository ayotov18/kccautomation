import type { Metadata } from 'next';
import { GeistSans } from 'geist/font/sans';
import { GeistMono } from 'geist/font/mono';
import { Instrument_Serif } from 'next/font/google';
import './globals.css';
import { ClientShell } from './client-shell';

// Display serif used sparingly — empty-state headlines, the once-per-page
// italic accent. Matches the landing page (marketing/app/layout.tsx).
const instrumentSerif = Instrument_Serif({
  subsets: ['latin'],
  weight: '400',
  style: 'italic',
  variable: '--font-display',
  display: 'swap',
});

export const metadata: Metadata = {
  title: {
    default: 'KCC Automation — AI-driven KCC reports from DWG',
    template: '%s · KCC Automation',
  },
  description:
    'Generate Bulgarian Количествено-Стойностна Сметка reports automatically from DWG/DXF drawings. Multi-module detection, RAG against your historical offers, full audit trail.',
  applicationName: 'KCC Automation',
  keywords: [
    'KCC',
    'КСС',
    'количествено-стойностна сметка',
    'Bulgarian construction estimating',
    'DWG to KCC',
    'DXF analysis',
    'BoQ',
    'bills of quantities',
    'construction RAG',
    'AI KCC',
  ],
  authors: [{ name: 'KCC Automation' }],
  formatDetection: { telephone: false, address: false, email: false },
  robots: {
    index: false,
    follow: false,
    nocache: true,
  },
  openGraph: {
    title: 'KCC Automation',
    description: 'AI-driven Bulgarian KCC reports from DWG drawings.',
    type: 'website',
    locale: 'en_GB',
    alternateLocale: ['bg_BG'],
  },
  twitter: { card: 'summary' },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={`dark ${GeistSans.variable} ${GeistMono.variable} ${instrumentSerif.variable}`}
    >
      <body
        className="min-h-screen font-sans"
        style={{
          background: 'var(--oe-bg)',
          color: 'var(--oe-text-primary)',
          fontFeatureSettings: '"ss01", "cv11", "liga", "kern", "calt"',
        }}
      >
        <div className="kcc-grain" aria-hidden />
        <div className="kcc-vignette" aria-hidden />
        <ClientShell>{children}</ClientShell>
      </body>
    </html>
  );
}
