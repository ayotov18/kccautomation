import type { MetadataRoute } from 'next';

/**
 * The product UI is auth-gated; we don't want crawlers indexing
 * authenticated routes. Allow only the public landing + login.
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: [
      { userAgent: '*', allow: ['/$', '/login'], disallow: '/' },
    ],
    sitemap: `${process.env.NEXT_PUBLIC_APP_URL ?? 'https://app.kccgen.xyz'}/sitemap.xml`,
  };
}
