'use client';

import { useEffect, useRef, useState } from 'react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { Upload } from 'lucide-react';
import { useDrawingsStore } from '@/lib/store';
import { api } from '@/lib/api';
import { PipelineProgress } from '@/components/progress/PipelineProgress';
import type { JobStatus } from '@/types';

export default function DrawingsPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { drawings, loading, error, fetchDrawings } = useDrawingsStore();
  const [deleting, setDeleting] = useState<string | null>(null);

  // Inline upload pipeline — same UX the old /upload page had, just co-located.
  const [uploadState, setUploadState] = useState<
    'idle' | 'uploading' | 'processing' | 'failed'
  >('idle');
  const [uploadFilename, setUploadFilename] = useState<string | null>(null);
  const [uploadJobStatus, setUploadJobStatus] = useState<JobStatus>('queued');
  const [uploadProgress, setUploadProgress] = useState(0);
  const [uploadError, setUploadError] = useState<string | null>(null);

  const handleUploadClick = () => fileInputRef.current?.click();

  const handleFileChosen = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const ext = file.name.split('.').pop()?.toLowerCase();
    if (ext !== 'dxf' && ext !== 'dwg' && ext !== 'pdf') {
      setUploadError('Only .dxf, .dwg, and .pdf files are supported.');
      setUploadState('failed');
      return;
    }
    setUploadError(null);
    setUploadFilename(file.name);
    setUploadState('uploading');

    try {
      const { drawing_id, job_id } = await api.uploadDrawing(file);
      setUploadState('processing');

      // Poll job status; redirect to detail page only after completion.
      const interval = setInterval(async () => {
        try {
          const job = await api.getJob(job_id);
          setUploadJobStatus(job.status);
          setUploadProgress(job.progress);
          if (job.status === 'done') {
            clearInterval(interval);
            router.push(`/drawings/${drawing_id}`);
          } else if (job.status === 'failed') {
            clearInterval(interval);
            setUploadState('failed');
            setUploadError(job.error_message || 'Processing failed');
          }
        } catch {
          clearInterval(interval);
          setUploadState('failed');
          setUploadError('Lost connection to server');
        }
      }, 1500);
    } catch (err) {
      setUploadState('failed');
      setUploadError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  const resetUpload = () => {
    setUploadState('idle');
    setUploadFilename(null);
    setUploadProgress(0);
    setUploadJobStatus('queued');
    setUploadError(null);
  };

  const uploading = uploadState === 'uploading' || uploadState === 'processing';

  const handleDelete = async (id: string, filename: string) => {
    if (!confirm(`Delete "${filename}"? This cannot be undone.`)) return;
    setDeleting(id);
    try {
      await api.deleteDrawing(id);
      fetchDrawings();
    } catch {
      // silent
    } finally {
      setDeleting(null);
    }
  };

  useEffect(() => {
    fetchDrawings();
  }, [fetchDrawings]);

  // Deep-link from legacy /upload route or the Dashboard "Upload" CTA.
  useEffect(() => {
    if (searchParams.get('upload') === '1') {
      fileInputRef.current?.click();
    }
  }, [searchParams]);

  useEffect(() => {
    document.title = 'Drawings · KCC';
  }, []);

  const formatDate = (iso: string) => {
    const d = new Date(iso);
    return d.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="min-h-screen flex flex-col">
<main className="flex-1 max-w-7xl mx-auto w-full px-6 py-8">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-2xl font-bold">Drawings</h1>
          <div className="flex items-center gap-3">
            <button
              onClick={() => fetchDrawings()}
              className="text-sm text-content-tertiary hover:text-content-primary transition-colors"
            >
              Refresh
            </button>
            <button
              onClick={handleUploadClick}
              disabled={uploading}
              className="oe-btn-primary"
            >
              <Upload size={14} strokeWidth={2} />
              {uploading ? 'Uploading…' : 'Upload drawing'}
            </button>
            <input
              ref={fileInputRef}
              type="file"
              accept=".dxf,.dwg,.pdf"
              onChange={handleFileChosen}
              className="hidden"
            />
          </div>
        </div>

        {loading && (
          <div className="flex items-center justify-center py-20">
            <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse mr-3" />
            <span className="text-content-tertiary">Loading drawings...</span>
          </div>
        )}

        {error && (
          <div className="bg-red-900/20 border border-red-800 rounded-lg px-4 py-3 text-sm text-red-400">
            {error}
          </div>
        )}

        {/* Upload pipeline overlay — visible while a file is being processed */}
        {uploadState !== 'idle' && (
          <div className="oe-card p-6 mb-6">
            <div className="flex items-start justify-between mb-4">
              <div className="min-w-0">
                <h2 className="text-sm font-semibold text-content-primary">
                  {uploadState === 'uploading'
                    ? 'Uploading drawing…'
                    : uploadState === 'processing'
                    ? 'Analyzing drawing…'
                    : 'Upload failed'}
                </h2>
                {uploadFilename && (
                  <p className="text-xs text-content-tertiary font-mono mt-0.5 truncate">
                    {uploadFilename}
                  </p>
                )}
              </div>
              {uploadState === 'failed' && (
                <button
                  onClick={resetUpload}
                  className="text-xs text-content-tertiary hover:text-content-primary"
                >
                  Dismiss
                </button>
              )}
            </div>

            {uploadState === 'uploading' && (
              <div className="flex items-center gap-3">
                <div className="w-2 h-2 rounded-full bg-sky-400 animate-pulse" />
                <span className="text-sm text-content-secondary">Sending file to server…</span>
              </div>
            )}

            {uploadState === 'processing' && (
              <PipelineProgress status={uploadJobStatus} progress={uploadProgress} />
            )}

            {uploadState === 'failed' && uploadError && (
              <p className="text-sm text-red-400">{uploadError}</p>
            )}
          </div>
        )}

        {!loading && !error && drawings.length === 0 && (
          <div className="text-center py-20">
            <p className="text-content-tertiary mb-4">No drawings uploaded yet</p>
            <button
              onClick={handleUploadClick}
              disabled={uploading}
              className="oe-btn-primary oe-btn-lg"
            >
              <Upload size={15} strokeWidth={2} />
              {uploading ? 'Uploading…' : 'Upload your first drawing'}
            </button>
          </div>
        )}

        {!loading && drawings.length > 0 && (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border-light text-left text-content-tertiary uppercase text-xs tracking-wider">
                  <th className="pb-3 pr-6 font-medium">Filename</th>
                  <th className="pb-3 pr-6 font-medium">Format</th>
                  <th className="pb-3 pr-6 font-medium">Units</th>
                  <th className="pb-3 pr-6 font-medium">Entities</th>
                  <th className="pb-3 pr-6 font-medium">Uploaded</th>
                  <th className="pb-3 font-medium w-16"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-light">
                {drawings.map((d) => (
                  <tr key={d.id} className="group hover:bg-surface-elevated/50">
                    <td className="py-4 pr-6">
                      <Link
                        href={`/drawings/${d.id}`}
                        className="text-blue-400 hover:text-blue-300 font-medium transition-colors"
                      >
                        {d.filename}
                      </Link>
                    </td>
                    <td className="py-4 pr-6">
                      <span className="px-2 py-0.5 bg-surface-tertiary rounded text-xs font-mono uppercase">
                        {d.original_format}
                      </span>
                    </td>
                    <td className="py-4 pr-6 text-content-secondary">
                      {d.units || '---'}
                    </td>
                    <td className="py-4 pr-6 text-content-secondary font-mono">
                      {d.entity_count?.toLocaleString() ?? '---'}
                    </td>
                    <td className="py-4 pr-6 text-content-tertiary">
                      {formatDate(d.created_at)}
                    </td>
                    <td className="py-4">
                      <button
                        onClick={() => handleDelete(d.id, d.filename)}
                        disabled={deleting === d.id}
                        className="text-content-tertiary hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100 disabled:opacity-50"
                        title="Delete drawing"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </main>
    </div>
  );
}
