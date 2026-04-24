const ITEMS = [
  'RUST',
  'AXUM',
  'SQLX',
  'POSTGRES 16',
  'REDIS 7',
  'NEXT.JS 15',
  'S3',
  'BRIGHTDATA',
  'OPENROUTER',
];

export function Marquee() {
  const doubled = [...ITEMS, ...ITEMS];
  return (
    <section className="relative overflow-hidden border-y border-[var(--color-hairline)] bg-[var(--color-bg-raised)]">
      <div className="relative py-7">
        <div className="flex whitespace-nowrap animate-marquee will-change-transform">
          {doubled.map((item, i) => (
            <span
              key={`${item}-${i}`}
              className="mx-10 font-[family-name:var(--font-mono)] text-[12px] uppercase tracking-[0.24em] text-[var(--color-fg-tertiary)]"
            >
              <span className="inline-block h-1 w-1 rounded-full bg-[var(--color-amber)] mr-3 align-middle" />
              {item}
            </span>
          ))}
        </div>
        <div
          aria-hidden
          className="pointer-events-none absolute inset-y-0 left-0 w-24 bg-gradient-to-r from-[var(--color-bg-raised)] to-transparent"
        />
        <div
          aria-hidden
          className="pointer-events-none absolute inset-y-0 right-0 w-24 bg-gradient-to-l from-[var(--color-bg-raised)] to-transparent"
        />
      </div>
    </section>
  );
}
