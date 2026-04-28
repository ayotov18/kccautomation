'use client';

/**
 * Pricing defaults — formerly /settings/pricing. Lives inside /prices because
 * pricing settings are a price concern, not a generic app setting. The form
 * persists via api.setPricingDefaults; values are injected into AI prompts and
 * used as defaults in every generated KSS.
 */

import { useEffect, useState } from 'react';
import { api } from '@/lib/api';
import {
  DEFAULT_PRICING,
  PRICING_PRESETS,
  type PricingDefaults,
  type PricingPresetId,
  type LaborRateBand,
} from '@/types/config';
import { Check, Save, RotateCcw } from 'lucide-react';
import { symbol } from '@/lib/currency';
import { Select } from '@/components/ui/Select';

type Saving = 'idle' | 'saving' | 'saved' | 'error';

export function PricingDefaultsSection() {
  const [state, setState] = useState<PricingDefaults>(DEFAULT_PRICING);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState<Saving>('idle');

  useEffect(() => {
    api
      .getPricingDefaults()
      .then((d) => setState(d ?? DEFAULT_PRICING))
      .catch(() => setState(DEFAULT_PRICING))
      .finally(() => setLoading(false));
  }, []);

  const save = async () => {
    setSaving('saving');
    try {
      await api.setPricingDefaults(state);
      setSaving('saved');
      setTimeout(() => setSaving('idle'), 1800);
    } catch {
      setSaving('error');
      setTimeout(() => setSaving('idle'), 2000);
    }
  };

  const applyPreset = (id: PricingPresetId) => {
    const p = PRICING_PRESETS.find((x) => x.id === id);
    if (!p) return;
    setState((s) => ({ ...s, ...p.overrides, active_preset: id }));
  };

  const resetAll = () => setState(DEFAULT_PRICING);

  const setRate = (key: keyof PricingDefaults['labor_rates'], band: LaborRateBand) =>
    setState((s) => ({ ...s, labor_rates: { ...s.labor_rates, [key]: band } }));

  return (
    <section className="oe-card p-5 space-y-5">
      <header className="flex items-start justify-between gap-4 flex-wrap">
        <div>
          <h2 className="text-base font-medium text-content-primary">
            Pricing defaults{' '}
            <span className="oe-badge ml-2" data-variant="info">your settings</span>
          </h2>
          <p className="mt-1 text-[12.5px] text-content-tertiary max-w-xl">
            Markups (contingency / delivery / profit / VAT) and labour-rate
            bands the AI uses on every generated КСС. <strong>Skipped</strong>{' '}
            when an offer is pinned to a drawing — the offer becomes the
            source of truth and only VAT is applied on top.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button onClick={resetAll} className="oe-btn-ghost oe-btn-sm">
            <RotateCcw size={13} /> По подразбиране
          </button>
          <button
            onClick={save}
            disabled={saving === 'saving'}
            className="oe-btn-primary oe-btn-sm"
          >
            {saving === 'saved' ? (
              <>
                <Check size={14} /> Запазено
              </>
            ) : (
              <>
                <Save size={13} /> {saving === 'saving' ? 'Запазване…' : 'Запази'}
              </>
            )}
          </button>
        </div>
      </header>

      {loading ? (
        <div className="p-8 text-center text-content-tertiary text-sm">Зареждане…</div>
      ) : (
        <div className="space-y-5">
          <SubSection title="Профил" subtitle="Един клик за стандартна конфигурация">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {PRICING_PRESETS.map((p) => {
                const active = state.active_preset === p.id;
                return (
                  <button
                    key={p.id}
                    onClick={() => applyPreset(p.id)}
                    className={`text-left rounded-xl px-4 py-3 border transition-colors ${
                      active
                        ? 'border-[color:var(--oe-accent)]/50 bg-[color:var(--oe-accent-soft-bg)]'
                        : 'border-border-light hover:border-border hover:bg-surface-secondary/60'
                    }`}
                  >
                    <div className="flex items-center justify-between">
                      <span
                        className={`text-sm font-medium ${
                          active ? 'text-[color:var(--oe-accent)]' : 'text-content-primary'
                        }`}
                      >
                        {p.label}
                      </span>
                      {active && <Check size={14} className="text-[color:var(--oe-accent)]" />}
                    </div>
                    <p className="text-xs text-content-tertiary mt-0.5">{p.description}</p>
                  </button>
                );
              })}
            </div>
          </SubSection>

          <SubSection title="Валута и ДДС">
            <div className="grid grid-cols-2 gap-4">
              <Field label="Валута">
                <Select
                  ariaLabel="Валута"
                  value={state.currency}
                  onChange={(v) =>
                    setState((s) => ({ ...s, currency: v as 'EUR' | 'EUR' }))
                  }
                  options={[{ value: 'EUR', label: 'EUR (€) — стандартна от 2026' }]}
                />
              </Field>
              <PctField
                label="ДДС ставка"
                value={state.vat_rate_pct}
                onChange={(v) => setState((s) => ({ ...s, vat_rate_pct: v }))}
              />
            </div>
          </SubSection>

          <SubSection
            title="Допълнителни разходи (ДР)"
            subtitle="Процентни надбавки върху всеки фактор. Взети от индустриалните източници (СЕК, УСН)."
          >
            <div className="grid grid-cols-2 gap-4">
              <PctField
                label="ДР над ФРЗ (труд)"
                hint="Осигуровки, отпуски, ППО, неприродително време"
                value={state.dr_labor_pct}
                onChange={(v) => setState((s) => ({ ...s, dr_labor_pct: v }))}
              />
              <PctField
                label="ДР над материали"
                hint="Доставно-складови разходи"
                value={state.dr_materials_pct}
                onChange={(v) => setState((s) => ({ ...s, dr_materials_pct: v }))}
              />
              <PctField
                label="ДР над лека механизация"
                value={state.dr_light_machinery_pct}
                onChange={(v) => setState((s) => ({ ...s, dr_light_machinery_pct: v }))}
              />
              <PctField
                label="ДР над тежка механизация"
                value={state.dr_heavy_machinery_pct}
                onChange={(v) => setState((s) => ({ ...s, dr_heavy_machinery_pct: v }))}
              />
            </div>
          </SubSection>

          <SubSection title="Надбавки" subtitle="Прилагат се след преките разходи + ДР">
            <div className="grid grid-cols-3 gap-4">
              <PctField
                label="Непредвидени"
                hint="ЗОП таван — до 10%"
                value={state.contingency_pct}
                onChange={(v) => setState((s) => ({ ...s, contingency_pct: v }))}
              />
              <PctField
                label="Печалба"
                hint="Публ. търгове 8-15%, частни до 20%"
                value={state.profit_pct}
                onChange={(v) => setState((s) => ({ ...s, profit_pct: v }))}
              />
              <AmountField
                label={`Транспорт (${symbol(state.currency)})`}
                value={state.transport_slab_eur}
                onChange={(v) => setState((s) => ({ ...s, transport_slab_eur: v }))}
              />
            </div>
          </SubSection>

          <SubSection
            title={`Ставки на труда (${symbol(state.currency)}/час)`}
            subtitle="Диапазони за 2026 — инжектират се като ценови котви в AI заявката"
          >
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <RateRow
                label="Зидар"
                value={state.labor_rates.mason}
                onChange={(b) => setRate('mason', b)}
              />
              <RateRow
                label="Кофражист"
                value={state.labor_rates.formwork}
                onChange={(b) => setRate('formwork', b)}
              />
              <RateRow
                label="Армировач"
                value={state.labor_rates.rebar}
                onChange={(b) => setRate('rebar', b)}
              />
              <RateRow
                label="Бояджия"
                value={state.labor_rates.painter}
                onChange={(b) => setRate('painter', b)}
              />
              <RateRow
                label="Електротехник"
                value={state.labor_rates.electrician}
                onChange={(b) => setRate('electrician', b)}
              />
              <RateRow
                label="Водопроводчик"
                value={state.labor_rates.plumber}
                onChange={(b) => setRate('plumber', b)}
              />
              <RateRow
                label="Заварчик"
                value={state.labor_rates.welder}
                onChange={(b) => setRate('welder', b)}
              />
              <RateRow
                label="Помощник"
                value={state.labor_rates.helper}
                onChange={(b) => setRate('helper', b)}
              />
            </div>
          </SubSection>
        </div>
      )}
    </section>
  );
}

function SubSection({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-3">
      <div>
        <h3 className="text-[11px] font-semibold uppercase tracking-wider text-content-secondary">
          {title}
        </h3>
        {subtitle && (
          <p className="text-[11.5px] text-content-tertiary mt-1">{subtitle}</p>
        )}
      </div>
      {children}
    </div>
  );
}

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block">
      <span className="block text-xs text-content-secondary mb-1.5">{label}</span>
      {children}
      {hint && <span className="block text-[11px] text-content-tertiary mt-1">{hint}</span>}
    </label>
  );
}

function PctField({
  label,
  hint,
  value,
  onChange,
}: {
  label: string;
  hint?: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <Field label={label} hint={hint}>
      <div className="relative">
        <input
          type="number"
          step="0.5"
          min="0"
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
          className="oe-input pr-10"
        />
        <span className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-content-tertiary">
          %
        </span>
      </div>
    </Field>
  );
}

function AmountField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <Field label={label}>
      <input
        type="number"
        step="50"
        min="0"
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
        className="oe-input"
      />
    </Field>
  );
}

function RateRow({
  label,
  value,
  onChange,
}: {
  label: string;
  value: LaborRateBand;
  onChange: (b: LaborRateBand) => void;
}) {
  return (
    <div className="flex items-center gap-3 p-3 rounded-lg bg-surface-secondary/40 border border-border-light/60">
      <span className="flex-1 text-sm text-content-primary">{label}</span>
      <input
        type="number"
        step="0.5"
        min="0"
        value={value.low}
        onChange={(e) => onChange({ ...value, low: parseFloat(e.target.value) || 0 })}
        className="oe-input w-20 text-center font-mono"
        aria-label={`${label} — ниска ставка`}
      />
      <span className="text-content-tertiary text-xs">–</span>
      <input
        type="number"
        step="0.5"
        min="0"
        value={value.high}
        onChange={(e) => onChange({ ...value, high: parseFloat(e.target.value) || 0 })}
        className="oe-input w-20 text-center font-mono"
        aria-label={`${label} — висока ставка`}
      />
    </div>
  );
}
