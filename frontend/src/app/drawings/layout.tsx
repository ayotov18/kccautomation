import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Drawings',
  description: 'Upload DWG/DXF drawings, generate KSS reports, and manage modules.',
};

export default function DrawingsLayout({ children }: { children: React.ReactNode }) {
  return children;
}
