import type { Metadata } from 'next';
import { Inter, JetBrains_Mono, Lora } from 'next/font/google';
import './globals.css';
import { ClientShell } from './client-shell';

// Inter Variable for UI chrome — Linear / Vercel / Granola convention.
// cv11 (single-story a), ss01 (open digits), ss03 (curved l) feature flags
// give it a "designed" rather than default look.
const inter = Inter({
  subsets: ['latin', 'cyrillic'],
  variable: '--font-sans',
  display: 'swap',
});

// JetBrains Mono with tabular-nums for entity counts, dimensions, prices.
const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
  display: 'swap',
});

// Lora — humanist serif. Used ONLY on AI-generated long-form prose
// (the drawing summary and KSS descriptions inside the editor) so that
// content reads as content, distinct from the chrome.
const lora = Lora({
  subsets: ['latin', 'cyrillic'],
  variable: '--font-display',
  display: 'swap',
});

export const metadata: Metadata = {
  title: {
    default: 'KCC Automation — AI-driven KSS reports from DWG',
    template: '%s · KCC Automation',
  },
  description:
    'Generate Bulgarian Количествено-Стойностна Сметка reports automatically from DWG/DXF drawings. Multi-module detection, RAG against your historical offers, full audit trail.',
  applicationName: 'KCC Automation',
  keywords: [
    'KSS',
    'КСС',
    'количествено-стойностна сметка',
    'Bulgarian construction estimating',
    'DWG to KSS',
    'DXF analysis',
    'BoQ',
    'bills of quantities',
    'construction RAG',
    'AI KCC',
  ],
  authors: [{ name: 'KCC Automation' }],
  formatDetection: { telephone: false, address: false, email: false },
  robots: {
    index: false, // internal product UI; auth-gated; no public indexing
    follow: false,
    nocache: true,
  },
  openGraph: {
    title: 'KCC Automation',
    description: 'AI-driven Bulgarian KSS reports from DWG drawings.',
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
      className={`dark ${inter.variable} ${jetbrainsMono.variable} ${lora.variable}`}
    >
      <body
        className="min-h-screen font-sans"
        style={{
          background: 'var(--oe-bg-primary)',
          color: 'var(--oe-text-primary)',
          fontFeatureSettings: '"cv11", "ss01", "ss03"',
        }}
      >
        <ClientShell>{children}</ClientShell>
      </body>
    </html>
  );
}
