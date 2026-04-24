'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { FolderOpen, Plus } from 'lucide-react';
import { api } from '@/lib/api';
import { EmptyState } from '@/components/ui/EmptyState';

export default function ProjectsPage() {
  const [projects, setProjects] = useState<Array<{ id: string; name: string; region: string; status: string; created_at: string }>>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listProjects().then(setProjects).catch(() => {}).finally(() => setLoading(false));
  }, []);

  return (
    <div className="oe-page-padding oe-fade-in">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="oe-section-title">Projects</h1>
          <p className="oe-section-subtitle">Manage construction projects</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-white" style={{ background: 'var(--oe-blue)' }}>
          <Plus className="w-4 h-4" /> New Project
        </button>
      </div>

      {loading ? (
        <div className="oe-card p-8 text-center" style={{ color: 'var(--oe-text-secondary)' }}>Loading...</div>
      ) : projects.length === 0 ? (
        <EmptyState icon={FolderOpen} title="No projects yet" description="Create your first construction project to get started." actionLabel="New Project" />
      ) : (
        <div className="grid gap-3">
          {projects.map((p) => (
            <Link key={p.id} href={`/projects/${p.id}`} className="oe-card p-4 flex items-center justify-between hover:shadow-md transition-shadow">
              <div>
                <div className="font-medium" style={{ color: 'var(--oe-text-primary)' }}>{p.name}</div>
                <div className="text-xs" style={{ color: 'var(--oe-text-secondary)' }}>{p.region} &middot; {p.status}</div>
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
