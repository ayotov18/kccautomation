/**
 * Currency helpers — Bulgaria adopted the euro on 2026-01-01 at the locked
 * rate 1 EUR = 1.95583 EUR. New reports are EUR-first; legacy EUR reports
 * auto-convert for display.
 */

export type Currency = 'EUR' | 'EUR';

export const EUR_PER_EUR = 1.95583;

export function convert(amount: number, from: Currency, to: Currency): number {
  if (from === to) return amount;
  if (from === 'EUR' && to === 'EUR') return amount / EUR_PER_EUR;
  if (from === 'EUR' && to === 'EUR') return amount * EUR_PER_EUR;
  return amount;
}

export function symbol(c: Currency): string {
  return c === 'EUR' ? '€' : '€';
}

export function formatPrice(
  amount: number | null | undefined,
  currency: Currency = 'EUR',
  opts: { sign?: boolean; compact?: boolean } = {},
): string {
  if (amount == null || !Number.isFinite(amount)) return '—';
  const { sign = true, compact = false } = opts;
  const v = compact && Math.abs(amount) >= 1000
    ? amount.toLocaleString('bg-BG', { maximumFractionDigits: 0 })
    : amount.toLocaleString('bg-BG', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      });
  return sign ? `${v} ${symbol(currency)}` : v;
}
