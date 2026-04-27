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
  title: 'KCC Automation',
  description: 'Construction ERP — KCC report generation from engineering drawings',
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
