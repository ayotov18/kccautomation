'use client';

import { useEffect, useState, useCallback, useMemo } from 'react';
import { api, type QuantityNorm, type ProjectDistribution, type QuantitySource, type QuantityRun, type QuantityMaterial } from '@/lib/api';

type Tab = 'norms' | 'distributions' | 'sources' | 'history';

const SEK_GROUPS: { code: string; label: string }[] = [
  { code: '', label: 'Всички групи' },
  { code: '01', label: 'СЕК01 — Земни работи' },
  { code: '02', label: 'СЕК02 — Кофраж' },
  { code: '03', label: 'СЕК03 — Армировка' },
  { code: '04', label: 'СЕК04 — Бетон' },
  { code: '05', label: 'СЕК05 — Зидария' },
  { code: '06', label: 'СЕК06 — Мазилки' },
  { code: '07', label: 'СЕК07 — Настилки' },
  { code: '08', label: 'СЕК08 — Бояджийски' },
  { code: '09', label: 'СЕК09 — Изолации' },
  { code: '10', label: 'СЕК10 — Дограма' },
  { code: '11', label: 'СЕК11 — ВиК' },
  { code: '12', label: 'СЕК12 — Електро' },
  { code: '20', label: 'СЕК20+ — Инсталации' },
];

const BUILDING_TYPES: { key: string; label: string }[] = [
  { key: 'residential_apartment', label: 'Жилищна сграда' },
  { key: 'bungalow', label: 'Еднофамилна къща' },
  { key: 'office', label: 'Офис сграда' },
  { key: 'school', label: 'Училище / общ. сграда' },
  { key: 'road', label: 'Път / инфраструктура' },
];

// Quantity norms — embedded inside /prices. Same 4 tabs (norms,
// distributions, sources, history) but rendered without the page chrome
// (no outer max-w/padding, no h1) so it slots into PricesPage.
export function QuantityNormsSection() {
  const [tab, setTab] = useState<Tab>('norms');

  return (
    <section className="oe-card p-5 space-y-4">
      <div>
        <h2 className="text-base font-medium text-content-primary">Quantity norms</h2>
        <p className="mt-1 text-[12.5px] text-content-tertiary">
          Нормативи за разход на труд и материали на единица работа. Подавани на AI
          като anchor, за да не &bdquo;халюцинира&ldquo; количества.
        </p>
      </div>

      <div className="flex gap-1 p-1 bg-surface-tertiary/40 rounded-xl w-fit">
        {([
          ['norms', 'Норми'],
          ['distributions', 'Разпределения'],
          ['sources', 'Източници'],
          ['history', 'История'],
        ] as [Tab, string][]).map(([key, label]) => (
          <button
            key={key}
            onClick={() => setTab(key)}
            className={`px-4 py-1.5 text-sm rounded-lg transition-all ${
              tab === key
                ? 'bg-surface-elevated text-content-primary shadow-sm'
                : 'text-content-secondary hover:text-content-primary'
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {tab === 'norms' && <NormsTab />}
      {tab === 'distributions' && <DistributionsTab />}
      {tab === 'sources' && <SourcesTab />}
      {tab === 'history' && <HistoryTab />}
    </section>
  );
}

// ═══════════════════════════════════════════════════════════════
// NORMS TAB — searchable table + side drawer for detail/edit
// ═══════════════════════════════════════════════════════════════

function NormsTab() {
  const [norms, setNorms] = useState<QuantityNorm[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [sekGroup, setSekGroup] = useState('');
  const [sourceFilter, setSourceFilter] = useState('');
  const [onlyMine, setOnlyMine] = useState(false);
  const [sources, setSources] = useState<QuantitySource[]>([]);
  const [selected, setSelected] = useState<QuantityNorm | null>(null);
  const [adding, setAdding] = useState(false);
  const [importing, setImporting] = useState(false);
  const [scraping, setScraping] = useState(false);
  const [scrapeJobId, setScrapeJobId] = useState<string | null>(null);
  const [scrapeProgress, setScrapeProgress] = useState(0);

  const fetchNorms = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.listQuantityNorms({
        search: search || undefined,
        sek_group: sekGroup || undefined,
        source: sourceFilter || undefined,
        only_mine: onlyMine || undefined,
        limit: 500,
      });
      setNorms(res.items);
      setTotal(res.total);
    } catch { /* */ }
    setLoading(false);
  }, [search, sekGroup, sourceFilter, onlyMine]);

  useEffect(() => {
    const t = setTimeout(fetchNorms, 250);
    return () => clearTimeout(t);
  }, [fetchNorms]);

  useEffect(() => {
    api.listQuantitySources().then(setSources).catch(() => {});
  }, []);

  useEffect(() => {
    if (!scrapeJobId) return;
    const interval = setInterval(async () => {
      try {
        const job = await api.getJob(scrapeJobId);
        setScrapeProgress(job.progress);
        if (job.status === 'done' || job.status === 'failed') {
          setScraping(false);
          setScrapeJobId(null);
          fetchNorms();
        }
      } catch { /* */ }
    }, 2000);
    return () => clearInterval(interval);
  }, [scrapeJobId, fetchNorms]);

  const handleScrape = async () => {
    setScraping(true);
    setScrapeProgress(0);
    try {
      const { job_id } = await api.triggerQuantityScrape();
      setScrapeJobId(job_id);
    } catch (e) {
      setScraping(false);
      alert((e as Error).message);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Изтриване на норма?')) return;
    try {
      await api.deleteQuantityNorm(id);
      setNorms(prev => prev.filter(n => n.id !== id));
      setSelected(null);
    } catch (e) {
      alert((e as Error).message);
    }
  };

  return (
    <div className="space-y-4">
      <section className="oe-card p-4">
        <div className="flex flex-wrap gap-3 items-center">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Търси по описание или СЕК код…"
            className="flex-1 min-w-64 px-3 py-2 bg-surface-tertiary border border-border-light rounded-lg text-sm focus:outline-none focus:border-sky-400"
          />
          <select
            value={sekGroup}
            onChange={(e) => setSekGroup(e.target.value)}
            className="px-3 py-2 bg-surface-tertiary border border-border-light rounded-lg text-sm"
          >
            {SEK_GROUPS.map(g => <option key={g.code} value={g.code}>{g.label}</option>)}
          </select>
          <select
            value={sourceFilter}
            onChange={(e) => setSourceFilter(e.target.value)}
            className="px-3 py-2 bg-surface-tertiary border border-border-light rounded-lg text-sm"
          >
            <option value="">Всички източници</option>
            {sources.map(s => <option key={s.id} value={s.site_name}>{s.site_name}</option>)}
          </select>
          <label className="flex items-center gap-2 text-sm text-content-secondary">
            <input type="checkbox" checked={onlyMine} onChange={(e) => setOnlyMine(e.target.checked)} className="accent-sky-400" />
            Само мои
          </label>
          <div className="ml-auto flex gap-2">
            <button onClick={() => setImporting(true)} className="oe-btn-ghost oe-btn-sm">Импорт CSV</button>
            <button
              onClick={handleScrape}
              disabled={scraping}
              className="oe-btn-primary oe-btn-sm"
              title="Извлича норми от конфигурираните източници (Ytong, Wienerberger, Sika, Mapei, Weber, Knauf, Fibran, НКЖИ, АПИ…)"
            >
              {scraping ? `Скрейпване… ${scrapeProgress}%` : '⚡ Скрейпни норми'}
            </button>
            <button onClick={() => setAdding(true)} className="oe-btn-primary oe-btn-sm">+ Нова норма</button>
          </div>
        </div>
        {scraping && (
          <div className="mt-3">
            <div className="w-full bg-surface-tertiary/60 rounded-full h-1.5">
              <div className="bg-sky-400 h-1.5 rounded-full transition-all" style={{ width: `${scrapeProgress}%` }} />
            </div>
          </div>
        )}
        <div className="flex items-center justify-between mt-3 text-xs text-content-tertiary">
          <span>{norms.length} / {total} показани</span>
        </div>
      </section>

      <section className="oe-card overflow-hidden">
        {loading ? (
          <div className="p-12 text-center text-content-tertiary text-sm">Зареждане…</div>
        ) : norms.length === 0 ? (
          <div className="p-12 text-center text-content-tertiary text-sm">Няма намерени норми.</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead className="bg-surface-tertiary/40 text-content-secondary text-xs uppercase tracking-wide">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">СЕК Код</th>
                  <th className="px-4 py-2.5 text-left font-medium">Описание</th>
                  <th className="px-4 py-2.5 text-left font-medium">Мярка</th>
                  <th className="px-4 py-2.5 text-right font-medium">Труд (h)</th>
                  <th className="px-4 py-2.5 text-left font-medium">Материали</th>
                  <th className="px-4 py-2.5 text-left font-medium">Източник</th>
                  <th className="px-4 py-2.5 text-right font-medium">Conf.</th>
                </tr>
              </thead>
              <tbody>
                {norms.map((n) => {
                  const mats = Array.isArray(n.materials) ? n.materials : [];
                  const totalH = (n.labor_qualified_h || 0) + (n.labor_helper_h || 0);
                  return (
                    <tr
                      key={n.id}
                      onClick={() => setSelected(n)}
                      className="border-t border-border-light/30 hover:bg-sky-50/30 dark:hover:bg-sky-500/5 cursor-pointer transition-colors"
                    >
                      <td className="px-4 py-2 font-mono text-xs text-sky-500 dark:text-sky-300">{n.sek_code}</td>
                      <td className="px-4 py-2 max-w-md truncate">{n.description_bg}</td>
                      <td className="px-4 py-2 text-content-secondary">{n.work_unit}</td>
                      <td className="px-4 py-2 text-right font-mono text-xs">{totalH.toFixed(2)}</td>
                      <td className="px-4 py-2 text-xs text-content-tertiary max-w-xs truncate">
                        {mats.slice(0, 2).map((m, i) => (
                          <span key={i} className="inline-block mr-2">
                            {m.name}: <span className="font-mono">{m.qty}{m.unit}</span>
                          </span>
                        ))}
                        {mats.length > 2 && <span>+{mats.length - 2}…</span>}
                      </td>
                      <td className="px-4 py-2 text-xs">
                        {n.user_id ? (
                          <span className="bg-sky-50 text-sky-700 dark:bg-sky-500/10 dark:text-sky-300 px-2 py-0.5 rounded-full">мой</span>
                        ) : (
                          <span className="text-content-tertiary">{n.source}</span>
                        )}
                      </td>
                      <td className="px-4 py-2 text-right">
                        <ConfidenceDot value={n.confidence} />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {selected && (
        <NormDrawer
          norm={selected}
          onClose={() => setSelected(null)}
          onSaved={() => { setSelected(null); fetchNorms(); }}
          onDelete={handleDelete}
        />
      )}

      {adding && (
        <NormDrawer
          norm={emptyNorm()}
          isNew
          onClose={() => setAdding(false)}
          onSaved={() => { setAdding(false); fetchNorms(); }}
        />
      )}

      {importing && (
        <ImportCsvDialog
          onClose={() => setImporting(false)}
          onImported={() => { setImporting(false); fetchNorms(); }}
        />
      )}
    </div>
  );
}

function ConfidenceDot({ value }: { value: number }) {
  const pct = Math.max(0, Math.min(1, value || 0));
  const color = pct >= 0.85 ? 'bg-emerald-400' : pct >= 0.65 ? 'bg-sky-400' : 'bg-amber-400';
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-content-tertiary">
      <span className={`w-2 h-2 rounded-full ${color}`} />
      {(pct * 100).toFixed(0)}%
    </span>
  );
}

function emptyNorm(): QuantityNorm {
  return {
    sek_code: '',
    description_bg: '',
    work_unit: 'М2',
    labor_qualified_h: 0,
    labor_helper_h: 0,
    labor_trade: '',
    materials: [],
    machinery: [],
    source: 'manual',
    source_url: '',
    confidence: 0.9,
  };
}

// ═══════════════════════════════════════════════════════════════
// NORM DRAWER — side drawer for view / edit / create
// ═══════════════════════════════════════════════════════════════

function NormDrawer({
  norm,
  isNew,
  onClose,
  onSaved,
  onDelete,
}: {
  norm: QuantityNorm;
  isNew?: boolean;
  onClose: () => void;
  onSaved: () => void;
  onDelete?: (id: string) => void;
}) {
  const [form, setForm] = useState<QuantityNorm>(() => ({
    ...norm,
    materials: Array.isArray(norm.materials) ? norm.materials : [],
    machinery: Array.isArray(norm.machinery) ? norm.machinery : [],
  }));
  const [saving, setSaving] = useState(false);
  const materials = form.materials as QuantityMaterial[];
  const machinery = form.machinery as QuantityMaterial[];
  const isBuiltin = !isNew && !form.user_id;

  const handleSave = async () => {
    setSaving(true);
    try {
      const payload = {
        sek_code: form.sek_code,
        description_bg: form.description_bg,
        work_unit: form.work_unit,
        labor_qualified_h: Number(form.labor_qualified_h) || 0,
        labor_helper_h: Number(form.labor_helper_h) || 0,
        labor_trade: form.labor_trade || null,
        materials: form.materials,
        machinery: form.machinery,
        source: form.source || 'manual',
        source_url: form.source_url || null,
        confidence: Number(form.confidence) || 0.9,
      };
      if (isNew || !form.id) {
        await api.createQuantityNorm(payload);
      } else {
        await api.updateQuantityNorm(form.id, payload);
      }
      onSaved();
    } catch (e) {
      alert((e as Error).message);
    }
    setSaving(false);
  };

  const updateMaterial = (idx: number, patch: Partial<QuantityMaterial>) => {
    const next = [...materials];
    next[idx] = { ...next[idx], ...patch };
    setForm(f => ({ ...f, materials: next }));
  };
  const addMaterial = () => setForm(f => ({ ...f, materials: [...materials, { name: '', qty: 0, unit: 'кг' }] }));
  const removeMaterial = (idx: number) => setForm(f => ({ ...f, materials: materials.filter((_, i) => i !== idx) }));

  const updateMachine = (idx: number, patch: Partial<QuantityMaterial>) => {
    const next = [...machinery];
    next[idx] = { ...next[idx], ...patch };
    setForm(f => ({ ...f, machinery: next }));
  };
  const addMachine = () => setForm(f => ({ ...f, machinery: [...machinery, { name: '', qty: 0, unit: 'маш.-ч' }] }));
  const removeMachine = (idx: number) => setForm(f => ({ ...f, machinery: machinery.filter((_, i) => i !== idx) }));

  return (
    <div className="fixed inset-0 z-50 flex">
      <div className="flex-1 bg-black/40 backdrop-blur-sm" onClick={onClose} />
      <div className="w-full max-w-xl h-full bg-surface-primary border-l border-border-light overflow-y-auto">
        <div className="p-6 space-y-5">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold">
                {isNew ? 'Нова норма' : 'Редакция на норма'}
              </h2>
              {isBuiltin && (
                <p className="text-xs text-amber-500 mt-1">Вградена норма — записването ще създаде ваша версия.</p>
              )}
            </div>
            <button onClick={onClose} className="text-content-tertiary hover:text-content-primary text-xl leading-none">&#10005;</button>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <Field label="СЕК код *">
              <input
                type="text"
                value={form.sek_code}
                onChange={(e) => setForm(f => ({ ...f, sek_code: e.target.value }))}
                className="oe-input"
                placeholder="СЕК05.002"
              />
            </Field>
            <Field label="Мярка *">
              <select value={form.work_unit} onChange={(e) => setForm(f => ({ ...f, work_unit: e.target.value }))} className="oe-input">
                {['М2', 'М3', 'м', 'кг', 'тон', 'бр.', 'компл.', 'L'].map(u => <option key={u}>{u}</option>)}
              </select>
            </Field>
          </div>

          <Field label="Описание *">
            <textarea
              value={form.description_bg}
              onChange={(e) => setForm(f => ({ ...f, description_bg: e.target.value }))}
              rows={2}
              className="oe-input"
            />
          </Field>

          <div className="grid grid-cols-3 gap-3">
            <Field label="Квалифициран труд (h)">
              <input
                type="number"
                step="0.01"
                value={form.labor_qualified_h}
                onChange={(e) => setForm(f => ({ ...f, labor_qualified_h: Number(e.target.value) }))}
                className="oe-input font-mono"
              />
            </Field>
            <Field label="Помощен труд (h)">
              <input
                type="number"
                step="0.01"
                value={form.labor_helper_h}
                onChange={(e) => setForm(f => ({ ...f, labor_helper_h: Number(e.target.value) }))}
                className="oe-input font-mono"
              />
            </Field>
            <Field label="Работна категория">
              <input
                type="text"
                value={form.labor_trade || ''}
                onChange={(e) => setForm(f => ({ ...f, labor_trade: e.target.value }))}
                className="oe-input"
                placeholder="зидар, армировчик…"
              />
            </Field>
          </div>

          <section>
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-medium">Материали на единица</h3>
              <button onClick={addMaterial} className="text-xs text-sky-500 hover:text-sky-400">+ Добави</button>
            </div>
            <div className="space-y-2">
              {materials.length === 0 && <p className="text-xs text-content-tertiary">Няма добавени материали.</p>}
              {materials.map((m, i) => (
                <div key={i} className="flex gap-2">
                  <input value={m.name} onChange={(e) => updateMaterial(i, { name: e.target.value })} placeholder="Име" className="flex-1 oe-input" />
                  <input type="number" step="0.001" value={m.qty} onChange={(e) => updateMaterial(i, { qty: Number(e.target.value) })} className="w-20 oe-input font-mono" />
                  <input value={m.unit} onChange={(e) => updateMaterial(i, { unit: e.target.value })} className="w-16 oe-input" placeholder="кг" />
                  <button onClick={() => removeMaterial(i)} className="text-content-tertiary hover:text-red-400">&#10005;</button>
                </div>
              ))}
            </div>
          </section>

          <section>
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-medium">Машини / оборудване</h3>
              <button onClick={addMachine} className="text-xs text-sky-500 hover:text-sky-400">+ Добави</button>
            </div>
            <div className="space-y-2">
              {machinery.length === 0 && <p className="text-xs text-content-tertiary">Няма добавени машини.</p>}
              {machinery.map((m, i) => (
                <div key={i} className="flex gap-2">
                  <input value={m.name} onChange={(e) => updateMachine(i, { name: e.target.value })} placeholder="Име" className="flex-1 oe-input" />
                  <input type="number" step="0.001" value={m.qty} onChange={(e) => updateMachine(i, { qty: Number(e.target.value) })} className="w-20 oe-input font-mono" />
                  <input value={m.unit} onChange={(e) => updateMachine(i, { unit: e.target.value })} className="w-20 oe-input" placeholder="маш.-ч" />
                  <button onClick={() => removeMachine(i)} className="text-content-tertiary hover:text-red-400">&#10005;</button>
                </div>
              ))}
            </div>
          </section>

          <div className="grid grid-cols-2 gap-3">
            <Field label="Източник">
              <input
                type="text"
                value={form.source}
                onChange={(e) => setForm(f => ({ ...f, source: e.target.value }))}
                className="oe-input"
              />
            </Field>
            <Field label="Confidence (0-1)">
              <input
                type="number"
                step="0.05"
                min="0"
                max="1"
                value={form.confidence}
                onChange={(e) => setForm(f => ({ ...f, confidence: Number(e.target.value) }))}
                className="oe-input font-mono"
              />
            </Field>
          </div>

          <Field label="URL на източника">
            <input
              type="text"
              value={form.source_url || ''}
              onChange={(e) => setForm(f => ({ ...f, source_url: e.target.value }))}
              placeholder="https://…"
              className="oe-input"
            />
          </Field>

          <div className="flex items-center justify-between pt-4 border-t border-border-light">
            {!isNew && form.id && form.user_id && onDelete ? (
              <button onClick={() => onDelete(form.id!)} className="oe-btn-danger oe-btn-sm">Изтрий</button>
            ) : <span />}
            <div className="flex gap-2">
              <button onClick={onClose} className="oe-btn-ghost oe-btn-sm">Отказ</button>
              <button onClick={handleSave} disabled={saving || !form.sek_code || !form.description_bg} className="oe-btn-primary oe-btn-sm">
                {saving ? 'Записва…' : 'Запази'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="block text-xs text-content-secondary mb-1">{label}</span>
      {children}
    </label>
  );
}

// ═══════════════════════════════════════════════════════════════
// IMPORT CSV DIALOG — expects header: sek_code,description_bg,work_unit,labor_qualified_h,labor_helper_h,source,source_url
// ═══════════════════════════════════════════════════════════════

function ImportCsvDialog({ onClose, onImported }: { onClose: () => void; onImported: () => void }) {
  const [file, setFile] = useState<File | null>(null);
  const [onDup, setOnDup] = useState<'skip' | 'replace'>('skip');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const handleImport = async () => {
    if (!file) return;
    setBusy(true);
    setErr(null);
    try {
      const text = await file.text();
      const lines = text.split(/\r?\n/).filter(l => l.trim().length > 0);
      if (lines.length < 2) throw new Error('CSV е празен.');
      const header = lines[0].split(',').map(h => h.trim());
      const idx = (k: string) => header.indexOf(k);
      const iCode = idx('sek_code');
      const iDesc = idx('description_bg');
      const iUnit = idx('work_unit');
      const iQ = idx('labor_qualified_h');
      const iH = idx('labor_helper_h');
      const iSrc = idx('source');
      if (iCode < 0 || iDesc < 0 || iUnit < 0) {
        throw new Error('CSV трябва да съдържа колони: sek_code, description_bg, work_unit.');
      }
      const norms = lines.slice(1).map(line => {
        const cells = splitCsvLine(line);
        return {
          sek_code: cells[iCode] || '',
          description_bg: cells[iDesc] || '',
          work_unit: cells[iUnit] || 'М2',
          labor_qualified_h: iQ >= 0 ? parseFloat(cells[iQ] || '0') : 0,
          labor_helper_h: iH >= 0 ? parseFloat(cells[iH] || '0') : 0,
          labor_trade: null,
          materials: [],
          machinery: [],
          source: iSrc >= 0 ? (cells[iSrc] || 'csv') : 'csv',
          source_url: null,
          confidence: 0.9,
        };
      }).filter(n => n.sek_code && n.description_bg);
      const res = await api.bulkImportQuantityNorms(norms, onDup);
      alert(`Импортирани: ${res.created} създадени, ${res.updated} обновени.`);
      onImported();
    } catch (e) {
      setErr((e as Error).message);
    }
    setBusy(false);
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-surface-elevated border border-border-light rounded-2xl p-6 w-full max-w-lg space-y-4">
        <h3 className="text-lg font-semibold">Импорт на норми (CSV)</h3>
        <p className="text-xs text-content-secondary">
          Очакван header: <code className="font-mono text-content-primary">sek_code,description_bg,work_unit,labor_qualified_h,labor_helper_h,source,source_url</code>
        </p>
        <input type="file" accept=".csv" onChange={(e) => setFile(e.target.files?.[0] || null)} className="block w-full text-sm" />
        <div className="flex items-center gap-4 text-sm">
          <label className="flex items-center gap-2">
            <input type="radio" name="dup" checked={onDup === 'skip'} onChange={() => setOnDup('skip')} className="accent-sky-400" />
            Пропусни дубликати
          </label>
          <label className="flex items-center gap-2">
            <input type="radio" name="dup" checked={onDup === 'replace'} onChange={() => setOnDup('replace')} className="accent-sky-400" />
            Замести дубликати
          </label>
        </div>
        {err && <p className="text-xs text-red-400">{err}</p>}
        <div className="flex justify-end gap-2 pt-2 border-t border-border-light">
          <button onClick={onClose} className="oe-btn-ghost oe-btn-sm">Отказ</button>
          <button onClick={handleImport} disabled={!file || busy} className="oe-btn-primary oe-btn-sm">
            {busy ? 'Импорт…' : 'Импортирай'}
          </button>
        </div>
      </div>
    </div>
  );
}

function splitCsvLine(line: string): string[] {
  const out: string[] = [];
  let cur = '';
  let inQuote = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') {
      if (inQuote && line[i + 1] === '"') { cur += '"'; i++; }
      else inQuote = !inQuote;
    } else if (c === ',' && !inQuote) {
      out.push(cur.trim());
      cur = '';
    } else {
      cur += c;
    }
  }
  out.push(cur.trim());
  return out;
}

// ═══════════════════════════════════════════════════════════════
// DISTRIBUTIONS TAB — building-type cards with sparklines
// ═══════════════════════════════════════════════════════════════

function DistributionsTab() {
  const [dists, setDists] = useState<ProjectDistribution[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeType, setActiveType] = useState(BUILDING_TYPES[0].key);
  const [editing, setEditing] = useState<ProjectDistribution | null>(null);

  const fetch = useCallback(async () => {
    setLoading(true);
    try {
      const res = await api.listProjectDistributions();
      setDists(res);
    } catch { /* */ }
    setLoading(false);
  }, []);

  useEffect(() => { fetch(); }, [fetch]);

  const forType = useMemo(() => dists.filter(d => d.building_type === activeType), [dists, activeType]);

  return (
    <div className="space-y-4">
      <section className="oe-card p-4">
        <div className="flex items-center gap-2 flex-wrap">
          {BUILDING_TYPES.map(bt => (
            <button
              key={bt.key}
              onClick={() => setActiveType(bt.key)}
              className={`px-3 py-1.5 text-sm rounded-full transition-all ${
                activeType === bt.key
                  ? 'bg-sky-500/15 text-sky-600 dark:text-sky-300 border border-sky-400/30'
                  : 'bg-surface-tertiary/40 text-content-secondary hover:text-content-primary border border-transparent'
              }`}
            >
              {bt.label}
            </button>
          ))}
          <button
            onClick={() => setEditing({ building_type: activeType, metric_key: '', metric_label_bg: '', unit: '', median_value: 0, sample_size: 0 })}
            className="ml-auto oe-btn-primary oe-btn-sm"
          >
            + Нова метрика
          </button>
        </div>
      </section>

      {loading ? (
        <div className="oe-card p-12 text-center text-sm text-content-tertiary">Зареждане…</div>
      ) : forType.length === 0 ? (
        <div className="oe-card p-12 text-center text-sm text-content-tertiary">Няма добавени метрики за този тип сграда.</div>
      ) : (
        <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
          {forType.map(d => (
            <DistributionCard key={d.id || d.metric_key} dist={d} onClick={() => setEditing(d)} />
          ))}
        </div>
      )}

      {editing && (
        <DistributionEditor
          dist={editing}
          onClose={() => setEditing(null)}
          onSaved={() => { setEditing(null); fetch(); }}
        />
      )}
    </div>
  );
}

function DistributionCard({ dist, onClick }: { dist: ProjectDistribution; onClick: () => void }) {
  const min = dist.min_value ?? dist.median_value;
  const max = dist.max_value ?? dist.median_value;
  const pctMid = max > min ? ((dist.median_value - min) / (max - min)) * 100 : 50;
  return (
    <button
      onClick={onClick}
      className="oe-card p-4 text-left hover:border-sky-400/40 transition-colors"
    >
      <div className="text-xs text-content-tertiary font-mono mb-1">{dist.metric_key}</div>
      <div className="text-sm font-medium text-content-primary mb-3">{dist.metric_label_bg}</div>
      <div className="text-2xl font-semibold text-sky-500 dark:text-sky-300">
        {dist.median_value.toLocaleString('bg-BG', { maximumFractionDigits: 2 })}
        <span className="text-xs text-content-tertiary font-normal ml-1">{dist.unit}</span>
      </div>
      <div className="mt-3 h-1.5 bg-surface-tertiary/60 rounded-full relative">
        <div className="absolute inset-y-0 left-0 bg-sky-400/30 rounded-full" style={{ width: '100%' }} />
        <div className="absolute top-1/2 -translate-y-1/2 w-2.5 h-2.5 bg-sky-400 rounded-full shadow" style={{ left: `calc(${pctMid}% - 5px)` }} />
      </div>
      <div className="flex justify-between mt-1.5 text-[10px] text-content-tertiary font-mono">
        <span>{min.toFixed(2)}</span>
        <span>n={dist.sample_size}</span>
        <span>{max.toFixed(2)}</span>
      </div>
      {dist.notes && <p className="text-xs text-content-tertiary mt-2 line-clamp-2">{dist.notes}</p>}
    </button>
  );
}

function DistributionEditor({ dist, onClose, onSaved }: { dist: ProjectDistribution; onClose: () => void; onSaved: () => void }) {
  const [form, setForm] = useState<ProjectDistribution>(dist);
  const [saving, setSaving] = useState(false);
  const isNew = !form.id;

  const save = async () => {
    setSaving(true);
    try {
      await api.upsertProjectDistribution({
        building_type: form.building_type,
        metric_key: form.metric_key,
        metric_label_bg: form.metric_label_bg,
        unit: form.unit,
        min_value: form.min_value ?? null,
        max_value: form.max_value ?? null,
        median_value: Number(form.median_value) || 0,
        sample_size: Number(form.sample_size) || 0,
        source: form.source || null,
        notes: form.notes || null,
      });
      onSaved();
    } catch (e) {
      alert((e as Error).message);
    }
    setSaving(false);
  };

  const del = async () => {
    if (!form.id || !confirm('Изтриване?')) return;
    try {
      await api.deleteProjectDistribution(form.id);
      onSaved();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-surface-elevated border border-border-light rounded-2xl p-6 w-full max-w-lg space-y-3">
        <h3 className="text-lg font-semibold">{isNew ? 'Нова метрика' : 'Редакция'}</h3>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Тип сграда">
            <select value={form.building_type} onChange={(e) => setForm(f => ({ ...f, building_type: e.target.value }))} className="oe-input">
              {BUILDING_TYPES.map(bt => <option key={bt.key} value={bt.key}>{bt.label}</option>)}
            </select>
          </Field>
          <Field label="Мярка">
            <input value={form.unit} onChange={(e) => setForm(f => ({ ...f, unit: e.target.value }))} className="oe-input" placeholder="м3/м2 РЗП" />
          </Field>
        </div>
        <Field label="Ключ (metric_key)">
          <input value={form.metric_key} onChange={(e) => setForm(f => ({ ...f, metric_key: e.target.value }))} className="oe-input font-mono" placeholder="concrete_per_rzp" />
        </Field>
        <Field label="Етикет (BG)">
          <input value={form.metric_label_bg} onChange={(e) => setForm(f => ({ ...f, metric_label_bg: e.target.value }))} className="oe-input" />
        </Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Мин"><input type="number" step="0.01" value={form.min_value ?? ''} onChange={(e) => setForm(f => ({ ...f, min_value: e.target.value === '' ? null : Number(e.target.value) }))} className="oe-input font-mono" /></Field>
          <Field label="Медиана *"><input type="number" step="0.01" value={form.median_value} onChange={(e) => setForm(f => ({ ...f, median_value: Number(e.target.value) }))} className="oe-input font-mono" /></Field>
          <Field label="Макс"><input type="number" step="0.01" value={form.max_value ?? ''} onChange={(e) => setForm(f => ({ ...f, max_value: e.target.value === '' ? null : Number(e.target.value) }))} className="oe-input font-mono" /></Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Брой проекти"><input type="number" value={form.sample_size} onChange={(e) => setForm(f => ({ ...f, sample_size: Number(e.target.value) }))} className="oe-input font-mono" /></Field>
          <Field label="Източник"><input value={form.source || ''} onChange={(e) => setForm(f => ({ ...f, source: e.target.value }))} className="oe-input" /></Field>
        </div>
        <Field label="Бележки"><textarea rows={2} value={form.notes || ''} onChange={(e) => setForm(f => ({ ...f, notes: e.target.value }))} className="oe-input" /></Field>
        <div className="flex justify-between items-center pt-3 border-t border-border-light">
          {!isNew && <button onClick={del} className="oe-btn-danger oe-btn-sm">Изтрий</button>}
          <div className="flex gap-2 ml-auto">
            <button onClick={onClose} className="oe-btn-ghost oe-btn-sm">Отказ</button>
            <button onClick={save} disabled={saving} className="oe-btn-primary oe-btn-sm">
              {saving ? 'Записва…' : 'Запази'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════
// SOURCES TAB — list + toggle + add custom
// ═══════════════════════════════════════════════════════════════

function SourcesTab() {
  const [sources, setSources] = useState<QuantitySource[]>([]);
  const [loading, setLoading] = useState(true);
  const [newSite, setNewSite] = useState('');
  const [newUrl, setNewUrl] = useState('');
  const [newDesc, setNewDesc] = useState('');

  const fetch = useCallback(async () => {
    setLoading(true);
    try {
      const s = await api.listQuantitySources();
      setSources(s);
    } catch { /* */ }
    setLoading(false);
  }, []);

  useEffect(() => { fetch(); }, [fetch]);

  const handleAdd = async () => {
    if (!newSite.trim() || !newUrl.trim()) return;
    try {
      await api.createQuantitySource({
        site_name: newSite.trim(),
        base_url: newUrl.trim(),
        description: newDesc.trim() || undefined,
        parser_template: 'generic',
      });
      setNewSite(''); setNewUrl(''); setNewDesc('');
      fetch();
    } catch (e) { alert((e as Error).message); }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Изтриване?')) return;
    try {
      await api.deleteQuantitySource(id);
      fetch();
    } catch (e) { alert((e as Error).message); }
  };

  return (
    <div className="space-y-4">
      <section className="oe-card p-5">
        <h2 className="text-base font-semibold mb-3">Източници за норми</h2>
        {loading ? <p className="text-sm text-content-tertiary">Зареждане…</p> : (
          <div className="space-y-2">
            {sources.map(s => (
              <div key={s.id} className="flex items-center justify-between bg-surface-tertiary/40 rounded-lg px-4 py-2.5">
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm">{s.site_name}</span>
                    {s.is_builtin && <span className="text-[10px] uppercase tracking-wide bg-sky-500/15 text-sky-600 dark:text-sky-300 px-1.5 py-0.5 rounded">built-in</span>}
                    {!s.enabled && <span className="text-[10px] uppercase tracking-wide bg-amber-500/15 text-amber-600 dark:text-amber-300 px-1.5 py-0.5 rounded">off</span>}
                  </div>
                  <p className="text-xs text-content-tertiary mt-0.5 truncate">
                    <a href={s.base_url} target="_blank" rel="noreferrer" className="hover:text-sky-400">{s.base_url}</a>
                  </p>
                  {s.description && <p className="text-xs text-content-secondary mt-0.5 truncate">{s.description}</p>}
                </div>
                {!s.is_builtin && (
                  <button onClick={() => handleDelete(s.id)} className="text-xs text-red-400 hover:text-red-300 ml-3">Премахни</button>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="oe-card p-5">
        <h3 className="text-base font-semibold mb-3">+ Добави собствен източник</h3>
        <div className="grid gap-2 md:grid-cols-[180px_1fr_1fr_auto]">
          <input value={newSite} onChange={(e) => setNewSite(e.target.value)} placeholder="site_name" className="oe-input" />
          <input value={newUrl} onChange={(e) => setNewUrl(e.target.value)} placeholder="https://…" className="oe-input" />
          <input value={newDesc} onChange={(e) => setNewDesc(e.target.value)} placeholder="Кратко описание" className="oe-input" />
          <button onClick={handleAdd} disabled={!newSite.trim() || !newUrl.trim()} className="oe-btn-primary oe-btn-sm">Добави</button>
        </div>
      </section>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════
// HISTORY TAB — scrape runs
// ═══════════════════════════════════════════════════════════════

function HistoryTab() {
  const [runs, setRuns] = useState<QuantityRun[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listQuantityRuns().then(r => { setRuns(r); setLoading(false); }).catch(() => setLoading(false));
  }, []);

  if (loading) return <div className="oe-card p-12 text-center text-sm text-content-tertiary">Зареждане…</div>;
  if (runs.length === 0) return <div className="oe-card p-12 text-center text-sm text-content-tertiary">Няма скрейп истории.</div>;

  return (
    <div className="oe-card overflow-hidden">
      <table className="w-full text-sm">
        <thead className="bg-surface-tertiary/40 text-content-secondary text-xs uppercase tracking-wide">
          <tr>
            <th className="px-4 py-2.5 text-left font-medium">Старт</th>
            <th className="px-4 py-2.5 text-left font-medium">Статус</th>
            <th className="px-4 py-2.5 text-right font-medium">Източници</th>
            <th className="px-4 py-2.5 text-right font-medium">Създадени</th>
            <th className="px-4 py-2.5 text-right font-medium">Обновени</th>
            <th className="px-4 py-2.5 text-right font-medium">Време</th>
          </tr>
        </thead>
        <tbody>
          {runs.map(r => (
            <tr key={r.id} className="border-t border-border-light/30">
              <td className="px-4 py-2 text-xs text-content-secondary">{new Date(r.started_at).toLocaleString('bg-BG')}</td>
              <td className="px-4 py-2">
                <span className={`text-xs px-2 py-0.5 rounded-full ${
                  r.status === 'done' ? 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-300' :
                  r.status === 'failed' ? 'bg-red-500/15 text-red-600 dark:text-red-300' :
                  'bg-sky-500/15 text-sky-600 dark:text-sky-300'
                }`}>{r.status}</span>
              </td>
              <td className="px-4 py-2 text-right font-mono text-xs">{r.successful_sources}/{r.total_sources}</td>
              <td className="px-4 py-2 text-right font-mono text-xs">{r.norms_created}</td>
              <td className="px-4 py-2 text-right font-mono text-xs">{r.norms_updated}</td>
              <td className="px-4 py-2 text-right font-mono text-xs text-content-tertiary">{r.elapsed_ms ? `${(r.elapsed_ms / 1000).toFixed(1)}s` : '—'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
