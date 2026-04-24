'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { LayoutDashboard, FileText, Layers, Upload, AlertTriangle } from 'lucide-react';
import { api } from '@/lib/api';

export default function DashboardPage() {
  const [stats, setStats] = useState({ projects: 0, drawings: 0, features: 0 });

  useEffect(() => {
    api.listDrawings().then((d) => setStats((s) => ({ ...s, drawings: d.length }))).catch(() => {});
  }, []);

  const cards = [
    { label: 'Active Projects', value: stats.projects, icon: LayoutDashboard, color: 'text-blue-400', href: '/projects' },
    { label: 'Drawings', value: stats.drawings, icon: FileText, color: 'text-emerald-400', href: '/drawings' },
    { label: 'Features Detected', value: stats.features, icon: Layers, color: 'text-sky-300', href: '/drawings' },
    { label: 'Open Issues', value: 0, icon: AlertTriangle, color: 'text-sky-300', href: '/validation' },
  ];

  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Dashboard</h1>
      <p className="oe-section-subtitle mb-6">Overview of your construction projects</p>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        {cards.map((c) => (
          <Link key={c.label} href={c.href} className="oe-card p-5 hover:shadow-md transition-shadow group">
            <div className="flex items-center justify-between mb-3">
              <c.icon className={`w-5 h-5 ${c.color}`} />
            </div>
            <div className="text-2xl font-bold" style={{ color: 'var(--oe-text-primary)' }}>{c.value}</div>
            <div className="text-xs mt-1" style={{ color: 'var(--oe-text-secondary)' }}>{c.label}</div>
          </Link>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="oe-card p-5">
          <h3 className="font-semibold mb-3" style={{ color: 'var(--oe-text-primary)' }}>Quick Actions</h3>
          <div className="space-y-2">
            <Link href="/drawings/upload" className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition">
              <Upload className="w-4 h-4 text-blue-400" /> <span className="text-sm">Upload Drawing</span>
            </Link>
            <Link href="/projects" className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition">
              <LayoutDashboard className="w-4 h-4 text-emerald-400" /> <span className="text-sm">New Project</span>
            </Link>
            <Link href="/costs" className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-surface-secondary/50 transition">
              <Layers className="w-4 h-4 text-sky-300" /> <span className="text-sm">Browse Cost Database</span>
            </Link>
          </div>
        </div>
        <div className="oe-card p-5">
          <h3 className="font-semibold mb-3" style={{ color: 'var(--oe-text-primary)' }}>Recent Activity</h3>
          <p className="text-sm" style={{ color: 'var(--oe-text-secondary)' }}>No recent activity yet.</p>
        </div>
      </div>
    </div>
  );
}
