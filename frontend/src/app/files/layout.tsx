import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Files',
  description: 'All your uploaded drawings and price-library offers in one place.',
};

export default function FilesLayout({ children }: { children: React.ReactNode }) {
  return children;
}
