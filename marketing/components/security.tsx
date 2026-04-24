import { Eyebrow } from './eyebrow';
import { Lock, FileSymlink, Database } from 'lucide-react';

const ITEMS = [
  {
    icon: Lock,
    title: 'JWT auth + Argon2 hashing',
    body: 'Per-user, every query scoped to the calling user.',
  },
  {
    icon: FileSymlink,
    title: 'SHA-256 deduplication',
    body: 'Same file uploaded twice is the same row, not a leak.',
  },
  {
    icon: Database,
    title: 'User-scoped S3 paths',
    body: 'Nothing sits in a shared bucket prefix.',
  },
];

export function Security() {
  return (
    <section className="relative py-24 md:py-32">
      <div className="mx-auto max-w-7xl px-6">
        <div className="max-w-2xl mb-14">
          <Eyebrow className="mb-4 block">On data</Eyebrow>
          <h2 className="text-[clamp(1.75rem,4vw,3rem)] font-semibold leading-[1.08] tracking-tight">
            Your drawings, your prices, your audit trail.
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {ITEMS.map((item) => (
            <div
              key={item.title}
              className="rounded-xl border border-[var(--color-hairline)] bg-[var(--color-bg-raised)] p-6"
            >
              <div className="mb-4 inline-flex h-8 w-8 items-center justify-center rounded-md border border-[var(--color-hairline-hi)]">
                <item.icon className="h-4 w-4 text-[var(--color-amber)]" strokeWidth={1.75} />
              </div>
              <h3 className="text-[14px] font-[family-name:var(--font-mono)] uppercase tracking-[0.08em]">
                {item.title}
              </h3>
              <p className="mt-3 text-[13px] leading-relaxed text-[var(--color-fg-secondary)]">
                {item.body}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
