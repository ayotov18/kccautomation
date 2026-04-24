'use client';

import { useParams } from 'next/navigation';

export default function BOQEditorPage() {
  const { id } = useParams<{ id: string }>();
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">BOQ Editor</h1>
      <p className="oe-section-subtitle mb-6">Bill of Quantities — {id}</p>
      <div className="oe-card p-6">
        <p style={{ color: 'var(--oe-text-secondary)' }}>AG Grid BOQ editor with hierarchical positions, markups, and versioning will appear here.</p>
      </div>
    </div>
  );
}
