'use client';

import { useState, useEffect, useRef } from 'react';
import { api } from '@/lib/api';
import { KssEditor } from './KssEditor';
import { Select } from '@/components/ui/Select';

interface PriceListInfo {
  id: string;
  name: string;
  item_count: number;
  created_at: string;
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

interface Props {
  drawingId: string | null;
}

export function KssPanel({ drawingId }: Props) {
  const [open, setOpen] = useState(false);
  const [priceLists, setPriceLists] = useState<PriceListInfo[]>([]);
  const [selectedPl, setSelectedPl] = useState<string>('');
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showEditor, setShowEditor] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      api.listPriceLists().then(setPriceLists).catch(() => {});
    }
  }, [open]);

  const handleUploadPl = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const result = await api.uploadPriceList(file);
      setPriceLists((prev) => [{ ...result, created_at: new Date().toISOString() }, ...prev]);
      setSelectedPl(result.id);
    } catch {
      setError('Failed to upload price list');
    }
  };

  const handleGenerate = async () => {
    if (!drawingId) return;
    setGenerating(true);
    setDone(false);
    setError(null);
    setProgress(0);

    try {
      const { job_id } = await api.generateKss(drawingId, selectedPl || undefined);

      // Poll job status
      const poll = setInterval(async () => {
        try {
          const job = await api.getJob(job_id);
          setProgress(job.progress);
          if (job.status === 'done') {
            clearInterval(poll);
            setGenerating(false);
            setDone(true);
          } else if (job.status === 'failed') {
            clearInterval(poll);
            setGenerating(false);
            setError(job.error_message || 'KSS generation failed');
          }
        } catch {
          clearInterval(poll);
          setGenerating(false);
          setError('Lost connection');
        }
      }, 1500);
    } catch {
      setGenerating(false);
      setError('Failed to start KSS generation');
    }
  };

  const handleDownloadExcel = async () => {
    if (!drawingId) return;
    try {
      const blob = await api.downloadKssExcel(drawingId);
      downloadBlob(blob, `kss-report-${drawingId}.xlsx`);
    } catch {
      setError('Excel download failed');
    }
  };

  const handleDownloadPdf = async () => {
    if (!drawingId) return;
    try {
      const blob = await api.downloadKssPdf(drawingId);
      downloadBlob(blob, `kss-report-${drawingId}.pdf`);
    } catch {
      setError('PDF download failed');
    }
  };

  return (
    <div className="relative z-40">
      <button
        onClick={() => setOpen(!open)}
        className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
          open
            ? 'bg-sky-500 text-gray-900'
            : 'bg-sky-500/90 text-gray-900 hover:bg-sky-400'
        }`}
        title="Generate KSS (Bill of Quantities)"
      >
        KSS
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-2 w-72 bg-gray-900 border border-gray-700 rounded-lg shadow-xl z-50 p-4 space-y-3">
          <div className="text-xs text-gray-500 uppercase tracking-wider">
            Bill of Quantities
          </div>

          {/* Price list selector */}
          <div>
            <label className="block text-xs text-gray-400 mb-1">Price List</label>
            <div className="flex gap-2">
              <div className="flex-1">
                <Select
                  ariaLabel="Price list"
                  value={selectedPl}
                  onChange={setSelectedPl}
                  options={[
                    { value: '', label: 'Use scraped prices (auto)' },
                    ...priceLists.map((pl) => ({
                      value: pl.id,
                      label: pl.name,
                      hint: `${pl.item_count}`,
                    })),
                  ]}
                />
              </div>
              <button
                onClick={() => fileInputRef.current?.click()}
                className="px-2 py-1.5 bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded text-xs text-gray-400 transition-colors"
                title="Upload price list CSV"
              >
                +
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept=".csv"
                onChange={handleUploadPl}
                className="hidden"
              />
            </div>
          </div>

          {/* Generate button */}
          {!generating && !done && (
            <button
              onClick={handleGenerate}
              disabled={!drawingId}
              className="w-full px-3 py-2 bg-sky-500 hover:bg-sky-400 disabled:bg-gray-700 disabled:text-gray-500 rounded-lg text-sm font-medium transition-colors"
            >
              Generate KSS
            </button>
          )}

          {/* Progress */}
          {generating && (
            <div>
              <div className="flex items-center gap-2 mb-1">
                <div className="w-2 h-2 rounded-full bg-sky-400 animate-pulse" />
                <span className="text-xs text-gray-400">Generating...</span>
              </div>
              <div className="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
                <div
                  className="h-full bg-sky-500 rounded-full transition-all duration-500"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          )}

          {/* Done — download buttons + editor toggle */}
          {done && (
            <div className="space-y-2">
              <div className="text-xs text-sky-300 flex items-center gap-1">
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                KSS generated
                <span className="ml-1 px-1.5 py-0.5 bg-sky-900/40 text-sky-300 rounded text-[10px] font-medium">
                  AI-enhanced
                </span>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleDownloadExcel}
                  className="flex-1 px-3 py-1.5 bg-sky-600 hover:bg-sky-500 rounded text-sm text-white transition-colors"
                >
                  Excel (.xlsx)
                </button>
                <button
                  onClick={handleDownloadPdf}
                  className="flex-1 px-3 py-1.5 bg-gray-700 hover:bg-gray-600 rounded text-sm text-gray-200 transition-colors"
                >
                  PDF
                </button>
              </div>
              <button
                onClick={() => setShowEditor(!showEditor)}
                className="text-xs text-sky-300 hover:text-sky-200 transition-colors"
              >
                {showEditor ? 'Hide editor' : 'Edit & correct KSS items'}
              </button>
              <button
                onClick={() => { setDone(false); setProgress(0); setShowEditor(false); }}
                className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
              >
                Generate again
              </button>
            </div>
          )}

          {/* Error */}
          {error && (
            <div className="text-xs text-red-400">{error}</div>
          )}
        </div>
      )}

      {/* KSS Editor — full-width panel below the viewer */}
      {showEditor && drawingId && (
        <div className="fixed inset-x-0 bottom-0 z-50 bg-gray-900 border-t border-gray-700 shadow-2xl max-h-[50vh] overflow-y-auto">
          <div className="max-w-7xl mx-auto p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <span className="text-xs text-sky-300 font-medium uppercase tracking-wider">KSS Editor</span>
                <span className="text-xs text-gray-600">Corrections train the DRM learning system</span>
              </div>
              <button
                onClick={() => setShowEditor(false)}
                className="text-gray-500 hover:text-gray-300 text-sm"
              >
                Close
              </button>
            </div>
            <KssEditor drawingId={drawingId} />
          </div>
        </div>
      )}
    </div>
  );
}
