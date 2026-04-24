const COLUMNS = [
  {
    label: 'Product',
    links: [
      { name: 'Features', href: '#features' },
      { name: 'Pipeline', href: '#pipeline' },
      { name: 'Stack', href: '#stack' },
      { name: 'Pricing (soon)', href: '#' },
    ],
  },
  {
    label: 'Company',
    links: [
      { name: 'About', href: '#' },
      { name: 'Blog (soon)', href: '#' },
      { name: 'Contact', href: 'mailto:hello@kccgen.xyz' },
    ],
  },
  {
    label: 'Legal',
    links: [
      { name: 'Terms', href: '#' },
      { name: 'Privacy', href: '#' },
      { name: 'Status', href: '#' },
    ],
  },
];

export function Footer() {
  return (
    <footer className="relative border-t border-[var(--color-hairline)] bg-[var(--color-bg-raised)]">
      <div className="mx-auto max-w-7xl px-6 py-16">
        <div className="grid grid-cols-2 md:grid-cols-[1.5fr_repeat(3,1fr)] gap-10">
          <div>
            <div className="flex items-center gap-2 text-[15px] font-semibold tracking-tight">
              <span className="inline-block h-5 w-5 rounded-[4px] bg-[var(--color-amber)]/90" />
              KCC Automation
            </div>
            <p className="mt-4 text-[13px] leading-relaxed text-[var(--color-fg-secondary)] max-w-xs">
              Construction estimating, on rails. DXF in, КСС out.
            </p>
          </div>

          {COLUMNS.map((col) => (
            <div key={col.label}>
              <p className="mb-4 font-[family-name:var(--font-mono)] text-[10px] uppercase tracking-[0.18em] text-[var(--color-fg-quaternary)]">
                {col.label}
              </p>
              <ul className="space-y-2.5">
                {col.links.map((link) => (
                  <li key={link.name}>
                    <a
                      href={link.href}
                      className="text-[13px] text-[var(--color-fg-secondary)] hover:text-[var(--color-fg)] transition-colors"
                    >
                      {link.name}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-14 pt-8 border-t border-[var(--color-hairline)] flex flex-wrap items-center justify-between gap-4 font-[family-name:var(--font-mono)] text-[11px] uppercase tracking-[0.14em] text-[var(--color-fg-quaternary)]">
          <span>© 2026 KCC Automation</span>
          <span>kccgen.xyz</span>
        </div>
      </div>
    </footer>
  );
}
