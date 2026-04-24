'use client';

import { useState } from 'react';
import { api } from '@/lib/api';

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

export function ReportDownloadPanel({ drawingId }: Props) {
  const [open, setOpen] = useState(false);
  const [downloading, setDownloading] = useState<string | null>(null);

  const handleDownload = async (format: 'pdf' | 'csv' | 'json') => {
    if (!drawingId) return;
    setDownloading(format);
    try {
      let blob: Blob;
      let ext: string;
      if (format === 'pdf') {
        blob = await api.downloadReportPdf(drawingId);
        ext = 'pdf';
      } else if (format === 'csv') {
        blob = await api.downloadReportCsv(drawingId);
        ext = 'csv';
      } else {
        blob = await api.downloadReportJson(drawingId);
        ext = 'json';
      }
      downloadBlob(blob, `kcc-report-${drawingId}.${ext}`);
    } catch {
      // Download failed silently
    } finally {
      setDownloading(null);
    }
  };

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
          open
            ? 'bg-blue-600 text-white'
            : 'bg-gray-800 text-gray-400 hover:text-gray-100'
        }`}
        title="Download reports"
      >
        Reports
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-2 w-48 bg-gray-900 border border-gray-700 rounded-lg shadow-xl z-50 py-1">
          <div className="px-3 py-1.5 border-b border-gray-800 mb-1">
            <span className="text-xs text-gray-500 uppercase tracking-wider">Download</span>
          </div>

          <button
            onClick={() => handleDownload('pdf')}
            disabled={downloading === 'pdf'}
            className="w-full flex items-center gap-3 px-3 py-2 hover:bg-gray-800/50 text-sm text-gray-300 transition-colors disabled:opacity-50"
          >
            <svg className="w-4 h-4 text-red-400 flex-none" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m.75 12l3 3m0 0l3-3m-3 3v-6m-1.5-9H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
            </svg>
            {downloading === 'pdf' ? 'Downloading...' : 'PDF Report'}
          </button>

          <button
            onClick={() => handleDownload('csv')}
            disabled={downloading === 'csv'}
            className="w-full flex items-center gap-3 px-3 py-2 hover:bg-gray-800/50 text-sm text-gray-300 transition-colors disabled:opacity-50"
          >
            <svg className="w-4 h-4 text-green-400 flex-none" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3.375 19.5h17.25m-17.25 0a1.125 1.125 0 01-1.125-1.125M3.375 19.5h7.5c.621 0 1.125-.504 1.125-1.125m-9.75 0V5.625m0 12.75v-1.5c0-.621.504-1.125 1.125-1.125m18.375 2.625V5.625m0 12.75c0 .621-.504 1.125-1.125 1.125m1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125m0 3.75h-7.5A1.125 1.125 0 0112 18.375m9.75-12.75c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125m19.5 0v1.5c0 .621-.504 1.125-1.125 1.125M2.25 5.625v1.5c0 .621.504 1.125 1.125 1.125m0 0h17.25m-17.25 0h7.5c.621 0 1.125.504 1.125 1.125M3.375 8.25c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125m17.25-3.75h-7.5c-.621 0-1.125.504-1.125 1.125m8.625-1.125c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125m-17.25 0h7.5m-7.5 0c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125M12 10.875v-1.5m0 1.5c0 .621-.504 1.125-1.125 1.125M12 10.875c0 .621.504 1.125 1.125 1.125m-2.25 0c.621 0 1.125.504 1.125 1.125M12 12h7.5m-7.5 0c-.621 0-1.125.504-1.125 1.125M20.625 12c.621 0 1.125.504 1.125 1.125v1.5c0 .621-.504 1.125-1.125 1.125m-17.25 0h7.5M12 14.625v-1.5m0 1.5c0 .621-.504 1.125-1.125 1.125M12 14.625c0 .621.504 1.125 1.125 1.125m-2.25 0c.621 0 1.125.504 1.125 1.125m0 0v1.5c0 .621-.504 1.125-1.125 1.125M12 16.875c0-.621.504-1.125 1.125-1.125" />
            </svg>
            {downloading === 'csv' ? 'Downloading...' : 'CSV Export'}
          </button>

          <button
            onClick={() => handleDownload('json')}
            disabled={downloading === 'json'}
            className="w-full flex items-center gap-3 px-3 py-2 hover:bg-gray-800/50 text-sm text-gray-300 transition-colors disabled:opacity-50"
          >
            <svg className="w-4 h-4 text-blue-400 flex-none" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
            </svg>
            {downloading === 'json' ? 'Downloading...' : 'JSON Data'}
          </button>
        </div>
      )}
    </div>
  );
}
