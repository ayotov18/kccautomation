'use client';

import { Calendar } from 'lucide-react';
import { EmptyState } from '@/components/ui/EmptyState';

export default function SchedulePage() {
  return (
    <div className="oe-page-padding oe-fade-in">
      <h1 className="oe-section-title">4D Schedule</h1>
      <p className="oe-section-subtitle mb-6">Project scheduling with CPM and Gantt visualization</p>
      <div className="oe-card">
        <EmptyState icon={Calendar} title="No schedule" description="Create a schedule linked to your BOQ for 4D planning." actionLabel="New Schedule" />
      </div>
    </div>
  );
}
