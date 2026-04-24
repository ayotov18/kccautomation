'use client';

import { Layers } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function AssembliesPage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">Assemblies</h1>
      <p className="oe-section-subtitle mb-6">Composite rate recipes for construction items</p>
      <div className="oe-card">
        <EmptyState icon={Layers} title="No assemblies" description="Create reusable assembly recipes to speed up estimation." actionLabel="New Assembly" />
      </div>
    </div>
  );
}
