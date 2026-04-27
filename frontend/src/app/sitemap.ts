import type { MetadataRoute } from 'next';

/**
 * Sitemap is intentionally minimal. The product UI is auth-gated and
 * marked `robots: { index: false }`. We expose only the marketing-ish
 * landing surfaces here so any link previewer (Slack, iMessage, Linear)
 * has something to attach metadata to.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const base = process.env.NEXT_PUBLIC_APP_URL ?? 'https://app.kccgen.xyz';
  return [
    { url: `${base}/`, changeFrequency: 'monthly' },
    { url: `${base}/login`, changeFrequency: 'monthly' },
  ];
}
