'use client';

import { useParams } from 'next/navigation';

export default function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Project Detail</h1>
      <p className="oe-section-subtitle mb-6">Project ID: {id}</p>
      <div className="oe-card p-6">
        <p style={{ color: 'var(--oe-text-secondary)' }}>Project overview, BOQs, drawings, and schedule will appear here.</p>
      </div>
    </div>
  );
}
