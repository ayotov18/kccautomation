'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

const navLinks = [
  { href: '/drawings', label: 'Drawings' },
  { href: '/upload', label: 'Upload' },
  { href: '/prices', label: 'Prices' },
  { href: '/settings', label: 'Settings' },
];

export function AppNav() {
  const pathname = usePathname();

  return (
    <nav className="border-b border-gray-800 bg-gray-900/50 backdrop-blur-sm">
      <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
        <Link href="/" className="flex items-center gap-3">
          <div className="w-8 h-8 rounded bg-blue-600 flex items-center justify-center font-bold text-sm">
            K
          </div>
          <span className="text-lg font-semibold tracking-tight">KCC Automation</span>
        </Link>
        <div className="flex items-center gap-6 text-sm">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={`transition-colors ${
                pathname === link.href
                  ? 'text-gray-100 font-medium'
                  : 'text-gray-400 hover:text-gray-100'
              }`}
            >
              {link.label}
            </Link>
          ))}
        </div>
      </div>
    </nav>
  );
}
