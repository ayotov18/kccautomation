'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronRight } from 'lucide-react';
import { api } from '@/lib/api';

/**
 * Floating breadcrumb pill — top-left. Matches FloatingCommandBar style.
 * Long labels (drawing filenames, KSS titles) fade to transparent on the
 * right half via mask-image, hover shows the full text in a tooltip.
 */

const ROUTE_LABELS: Record<string, string> = {
  dashboard: 'Табло',
  projects: 'Проекти',
  drawings: 'Чертежи',
  upload: 'Качване',
  viewer: 'Преглед',
  view: 'Преглед',
  kss: 'КСС',
  prepare: 'AI подготовка',
  costs: 'Ценова база',
  assemblies: 'Сглобки',
  validation: 'Валидация',
  schedule: 'График',
  costmodel: '5D модел',
  tendering: 'Тръжни документи',
  cde: 'Документи',
  prices: 'Цени',
  settings: 'Настройки',
  boq: 'BoQ',
  'drm-stats': 'DRM',
  reports: 'Отчети',
};

function isUuid(s: string) {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s);
}

interface Crumb {
  label: string;
  href?: string;
  fade?: boolean;
}

export function FloatingBreadcrumb() {
  const pathname = usePathname();
  const [drawingNames, setDrawingNames] = useState<Record<string, string>>({});

  // Opportunistically resolve UUID segments to filenames
  useEffect(() => {
    const segs = pathname.split('/').filter(Boolean);
    const uuids = segs.filter(isUuid).filter((u) => !(u in drawingNames));
    if (uuids.length === 0) return;
    Promise.all(
      uuids.map((id) =>
        api
          .getDrawing(id)
          .then((d) => [id, d.filename] as const)
          .catch(() => [id, id.slice(0, 8)] as const),
      ),
    ).then((pairs) => {
      setDrawingNames((prev) => ({
        ...prev,
        ...Object.fromEntries(pairs),
      }));
    });
  }, [pathname, drawingNames]);

  const crumbs: Crumb[] = (() => {
    const segs = pathname.split('/').filter(Boolean);
    if (segs.length === 0) return [{ label: 'Табло' }];
    const out: Crumb[] = [];
    let path = '';
    for (let i = 0; i < segs.length; i++) {
      const s = segs[i];
      path += `/${s}`;
      const isLast = i === segs.length - 1;
      let label: string;
      let fade = false;
      if (isUuid(s)) {
        label = drawingNames[s] ?? '…';
        fade = true; // filenames tend to be long
      } else {
        label = ROUTE_LABELS[s] ?? s;
      }
      out.push({ label, href: isLast ? undefined : path, fade });
    }
    return out;
  })();

  return (
    <div className="fixed top-4 left-1/2 -translate-x-1/2 z-40">
      <nav
        aria-label="Breadcrumb"
        className="kcc-floating-surface flex items-center gap-1.5 px-4 py-2 max-w-[min(560px,70vw)]"
      >
        {crumbs.map((c, i) => {
          const last = i === crumbs.length - 1;
          const content = (
            <span
              className={
                last
                  ? 'text-content-primary font-medium'
                  : 'text-content-secondary hover:text-content-primary transition-colors'
              }
              title={c.label}
            >
              {c.label}
            </span>
          );
          return (
            <span key={i} className="flex items-center gap-1.5 min-w-0">
              {i > 0 && (
                <ChevronRight
                  size={12}
                  className="flex-none text-content-tertiary"
                  strokeWidth={2.25}
                />
              )}
              <span
                className={
                  'block min-w-0 text-[13px] ' +
                  (c.fade ? 'kcc-fade-end max-w-[240px] truncate' : 'max-w-[220px] truncate')
                }
              >
                {c.href && !last ? (
                  <Link href={c.href} className="no-underline">
                    {content}
                  </Link>
                ) : (
                  content
                )}
              </span>
            </span>
          );
        })}
      </nav>
    </div>
  );
}
