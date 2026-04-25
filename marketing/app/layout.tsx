import type { Metadata } from 'next';
import { GeistSans } from 'geist/font/sans';
import { GeistMono } from 'geist/font/mono';
import { Instrument_Serif } from 'next/font/google';
import { LenisProvider } from '@/components/lenis-provider';
import './globals.css';

const instrumentSerif = Instrument_Serif({
  subsets: ['latin'],
  weight: ['400'],
  style: ['normal', 'italic'],
  variable: '--font-serif-loaded',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'KCC Automation — DXF to КСС in under three minutes',
  description:
    'Upload a construction drawing. Get back a priced КСС with live Bulgarian market data and a full audit trail. Built in Rust.',
  openGraph: {
    title: 'KCC Automation',
    description: 'DXF → КСС in under three minutes.',
    url: 'https://kccgen.xyz',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'KCC Automation',
    description: 'DXF → КСС in under three minutes.',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={`${GeistSans.variable} ${GeistMono.variable} ${instrumentSerif.variable}`}
    >
      <body className="bg-[var(--color-bg)] text-[var(--color-fg)] antialiased viewport-vignette">
        <LenisProvider />
        {children}
        <div aria-hidden className="fixed-grain" />
      </body>
    </html>
  );
}
