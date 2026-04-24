'use client';

import { ShieldCheck } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function ValidationPage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Validation</h1>
      <p className="oe-section-subtitle mb-6">BOQ quality checks and compliance validation</p>
      <div className="oe-card">
        <EmptyState icon={ShieldCheck} title="No validation reports" description="Select a BOQ and run validation to check for quality issues." actionLabel="Run Validation" />
      </div>
    </div>
  );
}
