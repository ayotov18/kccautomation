'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { FileText, FolderArchive, Tag, Upload } from 'lucide-react';
import { api } from '@/lib/api';

export default function DashboardPage() {
  const [drawingCount, setDrawingCount] = useState(0);
  const [offerCount, setOfferCount] = useState(0);
  const [corpusRows, setCorpusRows] = useState(0);

  useEffect(() => {
    api.listDrawings().then((d) => setDrawingCount(d.length)).catch(() => {});
    api
      .listCorpusImports()
      .then((d) => {
        setOfferCount(d.imports.length);
        setCorpusRows(d.total_corpus_rows);
      })
      .catch(() => {});
  }, []);

  const cards = [
    {
      label: 'Files',
      value: drawingCount + offerCount,
      hint: `${drawingCount} drawings · ${offerCount} offers`,
      icon: FolderArchive,
      href: '/files',
    },
    {
      label: 'Reports',
      value: drawingCount,
      hint: 'KSS exports per drawing',
      icon: FileText,
      href: '/reports/kss',
    },
    {
      label: 'Prices in library',
      value: corpusRows,
      hint: `${offerCount} uploaded offers`,
      icon: Tag,
      href: '/prices',
    },
  ];

  return (
    <div className="oe-fade-in">
      <div className="max-w-5xl mx-auto px-6 py-8 space-y-6">
        <div>
          <h1 className="text-[26px] font-semibold tracking-tight text-content-primary">
            Dashboard
          </h1>
          <p className="mt-1 text-[12.5px] text-content-tertiary">
            Three places matter: <Link href="/files" className="underline hover:text-sky-300">Files</Link>,{' '}
            <Link href="/reports/kss" className="underline hover:text-sky-300">Reports</Link>, and{' '}
            <Link href="/prices" className="underline hover:text-sky-300">Prices &amp; Data</Link>. Everything else is reachable from one of those.
          </p>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {cards.map((c) => (
            <Link
              key={c.label}
              href={c.href}
              className="oe-card p-5 hover:bg-surface-secondary/30 transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <c.icon className="w-5 h-5 text-content-tertiary" />
              </div>
              <div className="text-3xl font-semibold font-numeric text-content-primary">
                {c.value.toLocaleString('en-GB')}
              </div>
              <div className="mt-1 text-[12.5px] text-content-tertiary">{c.label}</div>
              <div className="mt-1 text-[11px] text-content-tertiary/70 font-numeric">
                {c.hint}
              </div>
            </Link>
          ))}
        </div>

        <div className="oe-card p-5">
          <h3 className="text-sm font-medium text-content-primary mb-3">Quick actions</h3>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-2">
            <Link
              href="/drawings/upload"
              className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition-colors text-sm"
            >
              <Upload className="w-4 h-4 text-content-tertiary" />
              <span className="text-content-secondary">Upload drawing</span>
            </Link>
            <Link
              href="/prices"
              className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition-colors text-sm"
            >
              <Tag className="w-4 h-4 text-content-tertiary" />
              <span className="text-content-secondary">Upload offer (XLSX)</span>
            </Link>
            <Link
              href="/files"
              className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition-colors text-sm"
            >
              <FolderArchive className="w-4 h-4 text-content-tertiary" />
              <span className="text-content-secondary">Browse all files</span>
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
