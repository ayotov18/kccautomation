import type { Metadata } from 'next';
import { GeistSans } from 'geist/font/sans';
import { GeistMono } from 'geist/font/mono';
import './globals.css';

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
    <html lang="en" className={`${GeistSans.variable} ${GeistMono.variable}`}>
      <body className="bg-[var(--color-bg)] text-[var(--color-fg)] antialiased">{children}</body>
    </html>
  );
}
