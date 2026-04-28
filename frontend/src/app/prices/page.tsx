'use client';

import { useEffect, useState, useCallback } from 'react';
import { api } from '@/lib/api';
import type { ScrapedPriceItem, ScrapeSource } from '@/types';
import { PriceLibrarySection } from '@/components/prices/PriceLibrarySection';
import { QuantityNormsSection } from '@/components/prices/QuantityNormsSection';
import { PricingDefaultsSection } from '@/components/prices/PricingDefaultsSection';
import { Select } from '@/components/ui/Select';

interface PriceListInfo {
  id: string;
  name: string;
  item_count: number;
  source?: string;
  is_default?: boolean;
  created_at: string;
}

export default function PricesPage() {
  // Price lists
  const [priceLists, setPriceLists] = useState<PriceListInfo[]>([]);
  const [loadingLists, setLoadingLists] = useState(true);

  // Scrape sources
  const [sources, setSources] = useState<ScrapeSource[]>([]);
  const [loadingSources, setLoadingSources] = useState(true);
  const [newUrl, setNewUrl] = useState('');
  const [newSiteName, setNewSiteName] = useState('');

  // Scraped prices browser
  const [scrapedPrices, setScrapedPrices] = useState<ScrapedPriceItem[]>([]);
  const [pricesTotal, setPricesTotal] = useState(0);
  const [loadingPrices, setLoadingPrices] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [sourceFilter, setSourceFilter] = useState('');
  const [pricesOffset, setPricesOffset] = useState(0);
  const PAGE_SIZE = 100;

  // Add price modal
  const [showAddModal, setShowAddModal] = useState(false);
  const [addForm, setAddForm] = useState({ sek_code: '', item_name: '', unit: 'М2', price_min_eur: '', price_max_eur: '', notes: '' });

  // Scraping state
  const [scraping, setScraping] = useState(false);
  const [scrapeJobId, setScrapeJobId] = useState<string | null>(null);
  const [scrapeProgress, setScrapeProgress] = useState(0);

  // Upload
  const [uploading, setUploading] = useState(false);

  // Active tab. The prices page is multi-faceted but one screen — tabs keep
  // the page short and let users land in the section that matches their task.
  const [tab, setTab] = useState<'library' | 'defaults' | 'norms' | 'browse'>('library');

  const fetchPriceLists = useCallback(async () => {
    try {
      const lists = await api.listPriceLists();
      setPriceLists(lists as PriceListInfo[]);
    } catch { /* */ }
    setLoadingLists(false);
  }, []);

  const fetchSources = useCallback(async () => {
    try {
      const s = await api.listScrapeSources();
      setSources(s);
    } catch { /* */ }
    setLoadingSources(false);
  }, []);

  const fetchScrapedPrices = useCallback(async (offset = 0, append = false) => {
    setLoadingPrices(true);
    try {
      const result = await api.listScrapedPrices({
        limit: PAGE_SIZE,
        offset,
        search: searchQuery || undefined,
        site: sourceFilter || undefined,
        source_type: undefined,
      });
      if (append) {
        setScrapedPrices(prev => [...prev, ...result.items]);
      } else {
        setScrapedPrices(result.items);
      }
      setPricesTotal(result.total);
      setPricesOffset(offset + result.items.length);
    } catch { /* */ }
    setLoadingPrices(false);
  }, [searchQuery, sourceFilter]);

  useEffect(() => {
    fetchPriceLists();
    fetchSources();
    fetchScrapedPrices();
  }, [fetchPriceLists, fetchSources, fetchScrapedPrices]);

  // Poll scrape job
  useEffect(() => {
    if (!scrapeJobId) return;
    const interval = setInterval(async () => {
      try {
        const job = await api.getJob(scrapeJobId);
        setScrapeProgress(job.progress);
        if (job.status === 'done' || job.status === 'failed') {
          setScraping(false);
          setScrapeJobId(null);
          fetchPriceLists();
          fetchScrapedPrices();
        }
      } catch { /* */ }
    }, 2000);
    return () => clearInterval(interval);
  }, [scrapeJobId, fetchPriceLists, fetchScrapedPrices]);

  const handleScrape = async () => {
    setScraping(true);
    setScrapeProgress(0);
    try {
      const { job_id } = await api.triggerScrape();
      setScrapeJobId(job_id);
    } catch {
      setScraping(false);
    }
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true);
    try {
      await api.uploadPriceList(file);
      fetchPriceLists();
    } catch { /* */ }
    setUploading(false);
    e.target.value = '';
  };

  const handleSetDefault = async (id: string) => {
    await api.setDefaultPriceList(id);
    fetchPriceLists();
  };

  const handleAddSource = async () => {
    if (!newUrl.trim()) return;
    await api.addScrapeSource(newSiteName || 'custom', newUrl.trim());
    setNewUrl('');
    setNewSiteName('');
    fetchSources();
  };

  const handleToggleSource = async (id: string, enabled: boolean) => {
    await api.toggleScrapeSource(id, enabled);
    fetchSources();
  };

  const handleDeleteSource = async (id: string) => {
    await api.deleteScrapeSource(id);
    fetchSources();
  };

  const formatDate = (iso: string) => new Date(iso).toLocaleDateString('bg-BG');

  // Server-side search — refetch when search changes
  useEffect(() => {
    const timer = setTimeout(() => { fetchScrapedPrices(0); }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery, sourceFilter, fetchScrapedPrices]);

  const handleAddPrice = async () => {
    if (!addForm.item_name.trim()) return;
    try {
      await api.createPriceRow({
        sek_code: addForm.sek_code || undefined,
        item_name: addForm.item_name,
        unit: addForm.unit || undefined,
        price_min_eur: addForm.price_min_eur ? parseFloat(addForm.price_min_eur) : undefined,
        price_max_eur: addForm.price_max_eur ? parseFloat(addForm.price_max_eur) : undefined,
        notes: addForm.notes || undefined,
      });
      setShowAddModal(false);
      setAddForm({ sek_code: '', item_name: '', unit: 'М2', price_min_eur: '', price_max_eur: '', notes: '' });
      fetchScrapedPrices(0);
    } catch { /* */ }
  };

  const handleArchive = async (id: string) => {
    if (!confirm('Archive this price entry?')) return;
    try {
      await api.archivePriceRow(id);
      setScrapedPrices(prev => prev.filter(p => p.id !== id));
      setPricesTotal(prev => prev - 1);
    } catch { /* */ }
  };

  const filteredPrices = scrapedPrices;

  return (
    <div className="oe-fade-in">
      <div className="max-w-6xl mx-auto px-6 py-10 space-y-6">
        <header className="space-y-2">
          <div className="oe-eyebrow">Prices</div>
          <h1 className="text-[26px] font-semibold tracking-[-0.025em] text-content-primary">
            Your price intelligence
          </h1>
          <p className="text-[12.5px] text-content-tertiary max-w-xl">
            Library for RAG, defaults injected into AI prompts, quantity norms
            as anchors, and a browse view of scraped + manual prices. Link any
            offer to a drawing for 1:1 generation.
          </p>
        </header>

        <div className="oe-tab-row">
          {(
            [
              ['library', 'Library'],
              ['defaults', 'Defaults'],
              ['norms', 'Norms'],
              ['browse', 'Browse'],
            ] as Array<[typeof tab, string]>
          ).map(([k, label]) => (
            <button
              key={k}
              onClick={() => setTab(k)}
              data-active={tab === k}
              className="oe-tab"
            >
              {label}
            </button>
          ))}
        </div>

        {tab === 'library' && <PriceLibrarySection />}
        {tab === 'defaults' && <PricingDefaultsSection />}
        {tab === 'norms' && <QuantityNormsSection />}

        {tab === 'browse' && (
          <>
        {/* My Price Lists */}
        <section className="oe-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">My Price Lists</h2>
            <div className="flex gap-3">
              <label className={`px-4 py-2 rounded text-sm cursor-pointer ${uploading ? 'bg-gray-700' : 'bg-gray-700 hover:bg-gray-600'}`}>
                {uploading ? 'Uploading...' : '+ Upload CSV'}
                <input type="file" accept=".csv" className="hidden" onChange={handleUpload} disabled={uploading} />
              </label>
              <button
                onClick={handleScrape}
                disabled={scraping}
                className="oe-btn-primary"
              >
                {scraping ? `Scraping... ${scrapeProgress}%` : '+ Scrape New Prices'}
              </button>
            </div>
          </div>

          {scraping && (
            <div className="mb-4">
              <div className="w-full bg-surface-tertiary rounded-full h-2">
                <div className="bg-emerald-500 h-2 rounded-full transition-all" style={{ width: `${scrapeProgress}%` }} />
              </div>
            </div>
          )}

          {loadingLists ? (
            <p className="text-content-tertiary text-sm">Loading...</p>
          ) : priceLists.length === 0 ? (
            <p className="text-content-tertiary text-sm">No price lists yet. Upload a CSV or scrape market prices.</p>
          ) : (
            <div className="space-y-3">
              {priceLists.map((list) => (
                <div key={list.id} className="flex items-center justify-between bg-surface-tertiary/50 rounded-lg px-4 py-3">
                  <div>
                    <div className="flex items-center gap-2">
                      {list.is_default && <span className="text-yellow-400 text-xs">&#9733;</span>}
                      <span className="font-medium">{list.name}</span>
                      {list.source === 'brightdata' && (
                        <span className="text-xs bg-emerald-900 text-emerald-300 px-2 py-0.5 rounded">scraped</span>
                      )}
                      {(!list.source || list.source === 'upload') && (
                        <span className="text-xs bg-gray-700 text-content-secondary px-2 py-0.5 rounded">uploaded</span>
                      )}
                    </div>
                    <p className="text-xs text-content-tertiary mt-1">
                      {list.item_count} items | {formatDate(list.created_at)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    {!list.is_default && (
                      <button
                        onClick={() => handleSetDefault(list.id)}
                        className="text-xs text-content-secondary hover:text-yellow-400 px-2 py-1"
                      >
                        Set Default
                      </button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>

        {/* Scrape Sources */}
        <section className="oe-card p-6">
          <h2 className="text-lg font-semibold mb-4">Scrape Sources</h2>

          {loadingSources ? (
            <p className="text-content-tertiary text-sm">Loading...</p>
          ) : (
            <div className="space-y-2 mb-4">
              {/* Built-in sources */}
              {[
                { name: 'daibau.bg', desc: '1,504 construction work prices' },
                { name: 'mr-bricolage.bg', desc: 'Building materials retail' },
                { name: 'bauhaus.bg', desc: 'Building materials retail' },
                { name: 'smr.sek-bg.com', desc: '3 free items per SEK group' },
              ].map((builtin) => {
                const existing = sources.find(s => s.site_name === builtin.name);
                const enabled = existing?.enabled ?? true;
                return (
                  <div key={builtin.name} className="flex items-center justify-between bg-surface-tertiary/30 rounded px-4 py-2">
                    <div className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={enabled}
                        onChange={() => existing && handleToggleSource(existing.id, !enabled)}
                        className="accent-emerald-500"
                      />
                      <div>
                        <span className="font-medium text-sm">{builtin.name}</span>
                        <span className="text-xs text-content-tertiary ml-2">({builtin.desc})</span>
                      </div>
                    </div>
                    <span className="text-xs text-content-tertiary">built-in</span>
                  </div>
                );
              })}

              {/* Custom sources */}
              {sources.filter(s => !s.is_builtin).map((src) => (
                <div key={src.id} className="flex items-center justify-between bg-surface-tertiary/30 rounded px-4 py-2">
                  <div className="flex items-center gap-3">
                    <input
                      type="checkbox"
                      checked={src.enabled}
                      onChange={() => handleToggleSource(src.id, !src.enabled)}
                      className="accent-emerald-500"
                    />
                    <div>
                      <span className="font-medium text-sm">{src.site_name}</span>
                      <span className="text-xs text-content-tertiary ml-2 break-all">{src.base_url}</span>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDeleteSource(src.id)}
                    className="text-xs text-red-400 hover:text-red-300 px-2"
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Add custom URL */}
          <div className="flex gap-2 mt-4">
            <input
              type="text"
              value={newSiteName}
              onChange={(e) => setNewSiteName(e.target.value)}
              placeholder="Site name"
              className="w-32 px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm focus:outline-none focus:border-gray-500"
            />
            <input
              type="text"
              value={newUrl}
              onChange={(e) => setNewUrl(e.target.value)}
              placeholder="https://example.com/prices"
              className="flex-1 px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm focus:outline-none focus:border-gray-500"
            />
            <button
              onClick={handleAddSource}
              disabled={!newUrl.trim()}
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 rounded text-sm"
            >
              + Add
            </button>
          </div>
        </section>

        {/* Price Browser */}
        <section className="oe-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Price Browser</h2>
            <div className="flex items-center gap-3">
              <span className="text-xs text-content-tertiary">
                Showing {scrapedPrices.length} of {pricesTotal}
              </span>
              <button
                onClick={() => setShowAddModal(true)}
                className="px-3 py-1.5 oe-btn-primary rounded text-sm font-medium"
              >
                + Add Price
              </button>
            </div>
          </div>

          <div className="flex gap-3 mb-4">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search by description or SEK code..."
              className="flex-1 px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm focus:outline-none focus:border-gray-500"
            />
            <Select
              size="sm"
              ariaLabel="Filter by source"
              value={sourceFilter}
              onChange={setSourceFilter}
              options={[
                { value: '', label: 'All sources' },
                { value: 'daibau.bg', label: 'daibau.bg' },
                { value: 'mr-bricolage.bg', label: 'mr-bricolage.bg' },
                { value: 'manual', label: 'Manual' },
              ]}
            />
          </div>

          {loadingPrices ? (
            <p className="text-content-tertiary text-sm">Loading...</p>
          ) : filteredPrices.length === 0 ? (
            <p className="text-content-tertiary text-sm">No scraped prices yet. Click &quot;Scrape New Prices&quot; above.</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left text-content-secondary border-b border-border-light">
                    <th className="px-3 py-2">СЕК Код</th>
                    <th className="px-3 py-2">Описание</th>
                    <th className="px-3 py-2">Мярка</th>
                    <th className="px-3 py-2">Цена (€)</th>
                    <th className="px-3 py-2">Цена (€)</th>
                    <th className="px-3 py-2">Източник</th>
                    <th className="px-3 py-2">Дата</th>
                    <th className="px-3 py-2 w-8"></th>
                  </tr>
                </thead>
                <tbody>
                  {filteredPrices.map((p) => (
                    <tr key={p.id} className="border-b border-border-light/50 hover:bg-surface-tertiary/30">
                      <td className="px-3 py-2 font-mono text-xs text-emerald-400">{p.sek_code || '-'}</td>
                      <td className="px-3 py-2">{p.item_name}</td>
                      <td className="px-3 py-2 text-content-secondary">{p.unit || '-'}</td>
                      <td className="px-3 py-2">
                        {p.price_min_eur != null && p.price_max_eur != null
                          ? `${p.price_min_eur.toFixed(2)} - ${p.price_max_eur.toFixed(2)} €`
                          : '-'}
                      </td>
                      <td className="px-3 py-2 text-content-tertiary text-xs">
                        {p.price_min_eur != null && p.price_max_eur != null
                          ? `${p.price_min_eur.toFixed(2)} - ${p.price_max_eur.toFixed(2)} €`
                          : '-'}
                      </td>
                      <td className="px-3 py-2 text-xs">
                        {p.is_manual ? (
                          <span className="bg-blue-900/50 text-blue-300 px-1.5 py-0.5 rounded">manual</span>
                        ) : p.is_user_edited ? (
                          <span className="bg-sky-900/40 text-sky-200 px-1.5 py-0.5 rounded">edited</span>
                        ) : p.site === 'ai_research' ? (
                          <span className="bg-sky-900/40 text-sky-300 px-1.5 py-0.5 rounded">AI research</span>
                        ) : (
                          <span className="text-content-tertiary">{p.site}</span>
                        )}
                      </td>
                      <td className="px-3 py-2 text-content-tertiary text-xs">{formatDate(p.captured_at)}</td>
                      <td className="px-3 py-1">
                        <button onClick={() => handleArchive(p.id)} className="text-content-tertiary hover:text-red-400 text-xs" title="Archive">&#10005;</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {/* Load More */}
          {scrapedPrices.length < pricesTotal && !loadingPrices && (
            <button
              onClick={() => fetchScrapedPrices(pricesOffset, true)}
              className="mt-4 w-full py-2 bg-surface-tertiary hover:bg-gray-700 rounded text-sm text-content-secondary"
            >
              Load more ({pricesTotal - scrapedPrices.length} remaining)
            </button>
          )}
        </section>

        {/* Add Price Modal */}
        {showAddModal && (
          <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center">
            <div className="bg-surface-elevated border border-border-light rounded-lg p-6 w-full max-w-lg space-y-4">
              <h3 className="text-lg font-semibold">Add Manual Price</h3>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs text-content-secondary mb-1">СЕК код</label>
                  <input type="text" value={addForm.sek_code} onChange={e => setAddForm(f => ({...f, sek_code: e.target.value}))} placeholder="СЕК05.002" className="w-full px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm" />
                </div>
                <div>
                  <label className="block text-xs text-content-secondary mb-1">Мярка</label>
                  <Select
                    ariaLabel="Мярка"
                    value={addForm.unit}
                    onChange={(v) => setAddForm((f) => ({ ...f, unit: v }))}
                    options={['М2', 'М3', 'м', 'бр.', 'кг', 'тон', 'компл.'].map((u) => ({
                      value: u,
                      label: u,
                    }))}
                  />
                </div>
              </div>
              <div>
                <label className="block text-xs text-content-secondary mb-1">Описание *</label>
                <input type="text" value={addForm.item_name} onChange={e => setAddForm(f => ({...f, item_name: e.target.value}))} placeholder="Тухлена зидария 29 см" className="w-full px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs text-content-secondary mb-1">Мин. цена (€)</label>
                  <input type="number" step="0.01" value={addForm.price_min_eur} onChange={e => setAddForm(f => ({...f, price_min_eur: e.target.value}))} className="w-full px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm" />
                </div>
                <div>
                  <label className="block text-xs text-content-secondary mb-1">Макс. цена (€)</label>
                  <input type="number" step="0.01" value={addForm.price_max_eur} onChange={e => setAddForm(f => ({...f, price_max_eur: e.target.value}))} className="w-full px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm" />
                </div>
              </div>
              <div>
                <label className="block text-xs text-content-secondary mb-1">Бележки</label>
                <input type="text" value={addForm.notes} onChange={e => setAddForm(f => ({...f, notes: e.target.value}))} className="w-full px-3 py-2 bg-surface-tertiary border border-border-light rounded text-sm" />
              </div>
              <div className="flex justify-end gap-3 pt-2">
                <button onClick={() => setShowAddModal(false)} className="px-4 py-2 text-sm text-content-secondary hover:text-content-primary">Cancel</button>
                <button onClick={handleAddPrice} disabled={!addForm.item_name.trim()} className="px-4 py-2 oe-btn-primary disabled:bg-gray-700 rounded text-sm font-medium">Add Price</button>
              </div>
            </div>
          </div>
        )}
          </>
        )}
      </div>
    </div>
  );
}
