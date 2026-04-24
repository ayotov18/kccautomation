'use client';

import { Handshake } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function TenderingPage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Tendering</h1>
      <p className="oe-section-subtitle mb-6">Bid packages, distribution, and comparison</p>
      <div className="oe-card">
        <EmptyState icon={Handshake} title="No tender packages" description="Create bid packages from BOQ sections and distribute to bidders." actionLabel="New Tender" />
      </div>
    </div>
  );
}
