import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Prices & Sources',
  description:
    'Manage your XLSX offer library, CSV price lists, and live scrape sources. Pin offers to drawings for 1:1 RAG generation.',
};

export default function PricesLayout({ children }: { children: React.ReactNode }) {
  return children;
}
