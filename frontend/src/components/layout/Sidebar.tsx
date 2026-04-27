'use client';

import { useState, useCallback } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { clsx } from 'clsx';
import {
  LayoutDashboard,
  FolderOpen,
  Layers,
  ShieldCheck,
  FileText,
  FileSpreadsheet,
  FolderArchive,
  Tag,
  Settings,
  ChevronRight,
  PanelLeftClose,
  PanelLeft,
  BarChart3,
} from 'lucide-react';

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
  mobileOpen: boolean;
  onMobileClose: () => void;
}

/* ---- Navigation structure ---- */

interface NavItem {
  label: string;
  path: string;
  icon: React.ReactNode;
}

interface NavGroup {
  title: string;
  items: NavItem[];
}

/*
 * Information architecture: 3 workflow-oriented groups.
 *
 *   Work     → what the user does daily on a drawing.
 *   Reports  → the deliverables produced from analysis.
 *   Data     → reference data, settings, audit/learning.
 *
 * "Upload Drawing" is no longer a peer entry — it becomes an action inside
 * the Drawings list page.
 */
const navGroups: NavGroup[] = [
  {
    title: 'Work',
    items: [
      { label: 'Dashboard', path: '/dashboard', icon: <LayoutDashboard size={16} strokeWidth={1.75} /> },
      { label: 'Drawings', path: '/drawings', icon: <FileText size={16} strokeWidth={1.75} /> },
      { label: 'Projects', path: '/projects', icon: <FolderOpen size={16} strokeWidth={1.75} /> },
      { label: 'Files', path: '/files', icon: <FolderArchive size={16} strokeWidth={1.75} /> },
    ],
  },
  {
    title: 'Reports',
    items: [
      { label: 'КСС (Bills of Quantities)', path: '/reports/kss', icon: <FileSpreadsheet size={16} strokeWidth={1.75} /> },
      { label: 'Validation', path: '/validation', icon: <ShieldCheck size={16} strokeWidth={1.75} /> },
    ],
  },
  {
    title: 'Data',
    items: [
      { label: 'Prices & Sources', path: '/prices', icon: <Tag size={16} strokeWidth={1.75} /> },
      { label: 'Assemblies', path: '/assemblies', icon: <Layers size={16} strokeWidth={1.75} /> },
      { label: 'Documents (CDE)', path: '/cde', icon: <FolderArchive size={16} strokeWidth={1.75} /> },
      { label: 'DRM Learning', path: '/drm-stats', icon: <BarChart3 size={16} strokeWidth={1.75} /> },
    ],
  },
];

/* ---- Collapsible group ---- */

/** Remember which groups the user collapsed across sessions. */
const GROUP_STORAGE_KEY = 'kcc_sidebar_collapsed_groups';

function readCollapsedGroups(): Set<string> {
  if (typeof window === 'undefined') return new Set();
  try {
    const raw = window.localStorage.getItem(GROUP_STORAGE_KEY);
    return raw ? new Set(JSON.parse(raw) as string[]) : new Set();
  } catch {
    return new Set();
  }
}

function writeCollapsedGroups(groups: Set<string>) {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(GROUP_STORAGE_KEY, JSON.stringify([...groups]));
}

function SidebarGroup({
  group,
  collapsed,
  defaultExpanded,
}: {
  group: NavGroup;
  collapsed: boolean;
  defaultExpanded: boolean;
}) {
  const [expanded, setExpanded] = useState(() => {
    if (typeof window === 'undefined') return defaultExpanded;
    return !readCollapsedGroups().has(group.title);
  });
  const pathname = usePathname();

  const isGroupActive = group.items.some(
    (item) =>
      pathname === item.path ||
      (item.path !== '/dashboard' && pathname.startsWith(item.path)),
  );

  const handleToggle = useCallback(() => {
    if (collapsed) return;
    setExpanded((prev) => {
      const next = !prev;
      const stored = readCollapsedGroups();
      if (next) stored.delete(group.title);
      else stored.add(group.title);
      writeCollapsedGroups(stored);
      return next;
    });
  }, [collapsed, group.title]);

  if (collapsed) {
    return (
      <div className="mb-1">
        {group.items.map((item) => (
          <SidebarLink key={item.path} item={item} collapsed={collapsed} />
        ))}
      </div>
    );
  }

  return (
    <div className="mb-1">
      <button
        onClick={handleToggle}
        className={clsx(
          'flex items-center w-full px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wider',
          'text-[var(--oe-text-tertiary)] hover:text-[var(--oe-text-secondary)]',
          'transition-colors duration-150',
          isGroupActive && 'text-[var(--oe-text-secondary)]',
        )}
      >
        <ChevronRight
          size={12}
          className={clsx(
            'mr-1 transition-transform duration-150',
            expanded && 'rotate-90',
          )}
        />
        {group.title}
      </button>
      {expanded && (
        <div className="mt-0.5">
          {group.items.map((item) => (
            <SidebarLink key={item.path} item={item} collapsed={collapsed} />
          ))}
        </div>
      )}
    </div>
  );
}

/* ---- Single nav link ---- */

function SidebarLink({ item, collapsed }: { item: NavItem; collapsed: boolean }) {
  const pathname = usePathname();
  // Drawings route also owns the KSS report subtree (per new IA).
  const isActive = (() => {
    if (item.path === '/dashboard') {
      return pathname === '/dashboard' || pathname === '/';
    }
    if (item.path === '/reports/kss') {
      return (
        pathname === '/reports/kss' ||
        pathname.startsWith('/reports/kss/') ||
        // legacy route still redirects here; treat it as active too
        /^\/drawings\/[^/]+\/kss(\/|$)/.test(pathname)
      );
    }
    if (item.path === '/drawings') {
      return (
        pathname === '/drawings' ||
        (pathname.startsWith('/drawings/') && !pathname.includes('/kss'))
      );
    }
    return pathname === item.path || pathname.startsWith(item.path + '/');
  })();

  return (
    <Link
      href={item.path}
      className={clsx(
        'flex items-center gap-2.5 rounded-md text-[13px] transition-all duration-150',
        collapsed ? 'justify-center mx-1 px-2 py-1.5' : 'mx-2 px-2.5 py-[5px]',
        isActive
          ? 'bg-oe-blue-subtle text-oe-blue font-medium'
          : 'text-content-secondary hover:bg-[var(--oe-sidebar-hover)] hover:text-content-primary',
      )}
      title={collapsed ? item.label : undefined}
    >
      <span className="flex-shrink-0">{item.icon}</span>
      {!collapsed && (
        <span className="text-sm truncate">{item.label}</span>
      )}
    </Link>
  );
}

/* ---- Main Sidebar ---- */

export function Sidebar({ collapsed, onToggle, mobileOpen, onMobileClose }: SidebarProps) {
  return (
    <>
      {/* Mobile overlay */}
      {mobileOpen && (
        <div
          className="fixed inset-0 z-40 bg-black/30 backdrop-blur-sm lg:hidden"
          onClick={onMobileClose}
        />
      )}

      <aside
        className={clsx(
          'fixed top-0 left-0 z-50 h-full flex flex-col',
          'bg-[var(--oe-sidebar-bg)]',
          'shadow-[1px_0_0_0_var(--oe-border-light)]',
          'transition-all duration-200 ease-out',
          // Desktop
          'lg:relative lg:translate-x-0',
          collapsed ? 'lg:w-[68px]' : 'lg:w-[var(--oe-sidebar-width)]',
          // Mobile
          mobileOpen ? 'translate-x-0 w-[var(--oe-sidebar-width)]' : '-translate-x-full lg:translate-x-0',
        )}
      >
        {/* Logo / Brand */}
        <div
          className={clsx(
            'flex items-center h-[var(--oe-header-height)] border-b border-border-light',
            collapsed ? 'justify-center px-2' : 'px-4',
          )}
        >
          {collapsed ? (
            <span className="text-lg font-bold text-[var(--oe-blue)]">K</span>
          ) : (
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold text-[var(--oe-blue)]">KCC</span>
              <span className="text-sm font-medium text-[var(--oe-text-secondary)]">Automation</span>
            </div>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 overflow-y-auto py-3">
          {navGroups.map((group) => (
            <SidebarGroup
              key={group.title}
              group={group}
              collapsed={collapsed}
              defaultExpanded={true}
            />
          ))}
        </nav>

        {/* Bottom actions */}
        <div className="border-t border-border-light py-2">
          <SidebarLink
            item={{ label: 'Settings', path: '/settings', icon: <Settings size={16} strokeWidth={1.75} /> }}
            collapsed={collapsed}
          />
          <button
            onClick={onToggle}
            className={clsx(
              'hidden lg:flex items-center gap-2.5 w-full rounded-lg transition-colors duration-150',
              'text-[var(--oe-text-tertiary)] hover:bg-[var(--oe-sidebar-hover)] hover:text-[var(--oe-text-secondary)]',
              collapsed ? 'justify-center mx-1 px-2 py-2' : 'mx-2 px-3 py-2',
            )}
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? <PanelLeft size={16} strokeWidth={1.75} /> : <PanelLeftClose size={16} strokeWidth={1.75} />}
            {!collapsed && <span className="text-sm">Collapse</span>}
          </button>
        </div>
      </aside>
    </>
  );
}
