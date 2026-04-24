'use client';

import { TrendingUp } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function CostModelPage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">5D Cost Model</h1>
      <p className="oe-section-subtitle mb-6">Earned Value Management — budget vs actual tracking</p>
      <div className="oe-card">
        <EmptyState icon={TrendingUp} title="No cost model" description="Create EVM snapshots to track project cost performance." actionLabel="Create Snapshot" />
      </div>
    </div>
  );
}
