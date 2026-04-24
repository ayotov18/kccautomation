'use client';

import { FolderArchive } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function CdePage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Common Data Environment</h1>
      <p className="oe-section-subtitle mb-6">ISO 19650 document management</p>
      <div className="oe-card">
        <EmptyState icon={FolderArchive} title="No documents" description="Upload and manage construction documents with version control." actionLabel="Upload Document" />
      </div>
    </div>
  );
}
