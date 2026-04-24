'use client';

import { useState } from 'react';
import { Database, Search } from 'lucide-react';
import { api } from '@/lib/api';
import { EmptyState } from '@/components/ui/EmptyState';

export default function CostsPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Array<{ id: string; code: string; description: string; unit: string; rate: string; region: string }>>([]);
  const [loading, setLoading] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const items = await api.searchCosts(query);
      setResults(items);
    } catch { /* */ }
    setLoading(false);
  };

  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Cost Database</h1>
      <p className="oe-section-subtitle mb-6">Search construction cost items</p>

      <div className="flex gap-3 mb-6">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4" style={{ color: 'var(--oe-text-tertiary)' }} />
          <input
            value={query} onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search costs (e.g., concrete, rebar, formwork)..."
            className="w-full pl-10 pr-4 py-2 rounded-lg border text-sm"
            style={{ background: 'var(--oe-bg-secondary)', borderColor: 'var(--oe-border)', color: 'var(--oe-text-primary)' }}
          />
        </div>
        <button onClick={handleSearch} disabled={loading} className="px-4 py-2 rounded-lg text-sm font-medium text-white" style={{ background: 'var(--oe-blue)' }}>
          {loading ? 'Searching...' : 'Search'}
        </button>
      </div>

      {results.length === 0 ? (
        <EmptyState icon={Database} title="No cost items" description="Search the cost database or import items from CSV." actionLabel="Import Costs" />
      ) : (
        <div className="oe-card overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr style={{ background: 'var(--oe-bg-secondary)', color: 'var(--oe-text-secondary)' }}>
                <th className="text-left px-4 py-3 font-medium">Code</th>
                <th className="text-left px-4 py-3 font-medium">Description</th>
                <th className="text-left px-4 py-3 font-medium">Unit</th>
                <th className="text-right px-4 py-3 font-medium">Rate</th>
                <th className="text-left px-4 py-3 font-medium">Region</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r) => (
                <tr key={r.id} className="border-t" style={{ borderColor: 'var(--oe-border-light)' }}>
                  <td className="px-4 py-2 font-mono text-xs" style={{ color: 'var(--oe-blue)' }}>{r.code}</td>
                  <td className="px-4 py-2">{r.description}</td>
                  <td className="px-4 py-2" style={{ color: 'var(--oe-text-secondary)' }}>{r.unit}</td>
                  <td className="px-4 py-2 text-right font-medium">{r.rate}</td>
                  <td className="px-4 py-2" style={{ color: 'var(--oe-text-secondary)' }}>{r.region || '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
