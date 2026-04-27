'use client';

import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { useRouter } from 'next/navigation';
import {
  ArrowUp,
  FileText,
  LayoutDashboard,
  Tag,
  FolderArchive,
  Search,
  Settings,
  Sparkles,
  Upload,
  X,
} from 'lucide-react';
import { clsx } from 'clsx';
import { api } from '@/lib/api';
import type { Drawing } from '@/types';

type Group = 'Работа' | 'Настройки';

interface Route {
  id: string;
  label: string;
  labelBg: string;
  path: string;
  group: Group;
  icon: React.ReactNode;
  keywords: string[];
}

interface ActionCmd {
  id: string;
  label: string;
  labelBg: string;
  icon: React.ReactNode;
  run: () => void;
  keywords: string[];
}

const ROUTES: Route[] = [
  { id: 'dashboard', label: 'Dashboard', labelBg: 'Табло', path: '/dashboard', group: 'Работа', icon: <LayoutDashboard size={14} />, keywords: ['начало', 'home', 'overview'] },
  { id: 'files', label: 'Files', labelBg: 'Файлове', path: '/files', group: 'Работа', icon: <FolderArchive size={14} />, keywords: ['files', 'uploads', 'drawings', 'projects', 'offers', 'documents', 'dwg', 'dxf', 'kss', 'кcс', 'отчети', 'reports'] },
  { id: 'prices', label: 'Prices', labelBg: 'Цени', path: '/prices', group: 'Работа', icon: <Tag size={14} />, keywords: ['цени', 'market', 'database', 'library', 'offer', 'xlsx', 'csv', 'норми', 'количества', 'ддс', 'vat', 'печалба', 'ставки', 'eur', '€'] },
  { id: 'settings', label: 'Settings', labelBg: 'Настройки', path: '/settings', group: 'Настройки', icon: <Settings size={14} />, keywords: ['config', 'настройки'] },
];

/** Shimmer hints surface what's reachable from the bar. */
const SHIMMER_HINTS = [
  'Табло',
  'Файлове',
  'Чертежи',
  'КСС отчети',
  'Цени и оферти',
  'Ценови настройки',
  'Количества & Норми',
  'Настройки',
];

function fuzzyScore(query: string, target: string): number {
  const q = query.toLowerCase().trim();
  const t = target.toLowerCase();
  if (!q) return 0;
  if (t.startsWith(q)) return 1;
  if (t.includes(q)) return 0.8;
  // Subsequence match
  let qi = 0;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) qi++;
  }
  return qi === q.length ? 0.5 : 0;
}

export function FloatingCommandBar() {
  const router = useRouter();

  const [query, setQuery] = useState('');
  const [focused, setFocused] = useState(false);
  const [filter, setFilter] = useState<Group | null>(null);
  const [hintIdx, setHintIdx] = useState(0);
  const [hintFading, setHintFading] = useState(false);
  const [activeIdx, setActiveIdx] = useState(0);
  const [recent, setRecent] = useState<Drawing[]>([]);

  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Shimmer rotation — slow, immersive: 500 ms fade-out, swap, 500 ms fade-in,
  // then dwell for 5 s. Total cycle ~6 s per hint.
  useEffect(() => {
    let mounted = true;
    const tick = () => {
      if (!mounted) return;
      setHintFading(true);
      setTimeout(() => {
        if (!mounted) return;
        setHintIdx((i) => (i + 1) % SHIMMER_HINTS.length);
        setHintFading(false);
      }, 500);
    };
    const id = setInterval(tick, 6000);
    return () => { mounted = false; clearInterval(id); };
  }, []);

  // Global ⌘K shortcut
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        inputRef.current?.focus();
      } else if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        inputRef.current?.blur();
        setQuery('');
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Load recent drawings once focused
  useEffect(() => {
    if (!focused || recent.length > 0) return;
    api.listDrawings().then((d) => setRecent(d.slice(0, 5))).catch(() => {});
  }, [focused, recent.length]);

  const actions: ActionCmd[] = useMemo(
    () => [
      {
        id: 'upload',
        label: 'Upload drawing',
        labelBg: 'Качи чертеж',
        icon: <Upload size={14} />,
        keywords: ['upload', 'качи', 'нов'],
        run: () => router.push('/files?upload=drawing'),
      },
      {
        id: 'ask-ai',
        label: 'Ask with AI',
        labelBg: 'Попитай AI',
        icon: <Sparkles size={14} />,
        keywords: ['ai', 'ask', 'помощ'],
        run: () => {
          // Phase 2: send query to AI. For now navigate to kss if any.
          console.info('[command] AI query:', query);
        },
      },
    ],
    [router, query],
  );

  const q = query.trim();
  const matchedRoutes = useMemo(() => {
    const pool = filter ? ROUTES.filter((r) => r.group === filter) : ROUTES;
    if (!q) return pool;
    return pool
      .map((r) => ({
        r,
        score: Math.max(
          fuzzyScore(q, r.labelBg),
          fuzzyScore(q, r.label),
          ...r.keywords.map((k) => fuzzyScore(q, k)),
        ),
      }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .map((x) => x.r);
  }, [q, filter]);

  const matchedActions = useMemo(() => {
    if (!q) return actions;
    return actions
      .map((a) => ({
        a,
        score: Math.max(
          fuzzyScore(q, a.labelBg),
          fuzzyScore(q, a.label),
          ...a.keywords.map((k) => fuzzyScore(q, k)),
        ),
      }))
      .filter((x) => x.score > 0)
      .map((x) => x.a);
  }, [q, actions]);

  const matchedDrawings = useMemo(() => {
    if (!q) return recent;
    return recent
      .map((d) => ({ d, score: fuzzyScore(q, d.filename) }))
      .filter((x) => x.score > 0)
      .map((x) => x.d);
  }, [q, recent]);

  // Flatten for keyboard nav
  const flatResults = useMemo(
    () => [
      ...matchedRoutes.map((r) => ({ kind: 'route' as const, data: r })),
      ...matchedActions.map((a) => ({ kind: 'action' as const, data: a })),
      ...matchedDrawings.map((d) => ({ kind: 'drawing' as const, data: d })),
    ],
    [matchedRoutes, matchedActions, matchedDrawings],
  );

  useEffect(() => setActiveIdx(0), [query, filter]);

  const activate = useCallback(
    (item: (typeof flatResults)[number]) => {
      if (item.kind === 'route') router.push(item.data.path);
      else if (item.kind === 'action') item.data.run();
      else if (item.kind === 'drawing') router.push(`/drawings/${item.data.id}`);
      setQuery('');
      inputRef.current?.blur();
    },
    [router],
  );

  const onInputKey = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIdx((i) => Math.min(flatResults.length - 1, i + 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIdx((i) => Math.max(0, i - 1));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = flatResults[activeIdx];
      if (item) activate(item);
    }
  };

  const showDropdown = focused;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 w-[min(640px,calc(100vw-32px))]">
      {/* Dropdown — above the bar */}
      {showDropdown && (
        <div
          ref={dropdownRef}
          className="kcc-floating-surface kcc-floating-panel mb-2 overflow-hidden oe-fade-in"
          onMouseDown={(e) => e.preventDefault()}
        >
          <div className="max-h-[360px] overflow-y-auto py-1">
            {matchedRoutes.length > 0 && (
              <GroupLabel label="Страници" />
            )}
            {matchedRoutes.map((r) => {
              const flatIdx = flatResults.findIndex((x) => x.kind === 'route' && x.data.id === r.id);
              return (
                <ResultRow
                  key={`r-${r.id}`}
                  active={activeIdx === flatIdx}
                  onClick={() => activate({ kind: 'route', data: r })}
                  icon={r.icon}
                  label={r.labelBg}
                  meta={r.path}
                />
              );
            })}

            {matchedActions.length > 0 && (
              <GroupLabel label="Действия" />
            )}
            {matchedActions.map((a) => {
              const flatIdx = flatResults.findIndex((x) => x.kind === 'action' && x.data.id === a.id);
              return (
                <ResultRow
                  key={`a-${a.id}`}
                  active={activeIdx === flatIdx}
                  onClick={() => activate({ kind: 'action', data: a })}
                  icon={a.icon}
                  label={a.labelBg}
                  meta={a.label}
                />
              );
            })}

            {matchedDrawings.length > 0 && (
              <GroupLabel label={q ? 'Съвпадащи чертежи' : 'Последни чертежи'} />
            )}
            {matchedDrawings.map((d) => {
              const flatIdx = flatResults.findIndex((x) => x.kind === 'drawing' && x.data.id === d.id);
              return (
                <ResultRow
                  key={`d-${d.id}`}
                  active={activeIdx === flatIdx}
                  onClick={() => activate({ kind: 'drawing', data: d })}
                  icon={<FileText size={14} />}
                  label={d.filename}
                  meta={new Date(d.created_at).toLocaleDateString('bg-BG')}
                />
              );
            })}

            {flatResults.length === 0 && (
              <div className="px-4 py-6 text-center text-xs text-content-tertiary">
                Няма резултати. Натисни Enter за AI заявка.
              </div>
            )}
          </div>
        </div>
      )}

      {/* Bar itself — slightly softer radius (not a hard pill) so the two-row
          layout reads as one unit without weird half-circle cuts. */}
      <div className="kcc-floating-surface kcc-floating-panel px-3 py-2.5">
        {/* Top row: group filter chips */}
        <div className="flex items-center gap-1.5 mb-2 px-1">
          {(['Работа', 'Настройки'] as Group[]).map((g) => (
            <button
              key={g}
              type="button"
              onClick={() => setFilter(filter === g ? null : g)}
              className={clsx(
                'px-2.5 py-1 rounded-full text-[11px] font-medium transition-colors border',
                filter === g
                  ? 'border-[color:var(--oe-accent)]/30 text-[color:var(--oe-accent)] bg-[color:var(--oe-accent-soft-bg)]'
                  : 'text-content-tertiary hover:text-content-secondary border-transparent',
              )}
            >
              {g}
            </button>
          ))}
          <span className="ml-auto text-[10px] text-content-tertiary pr-1">
            {ROUTES.length} места
          </span>
        </div>

        {/* Input row */}
        <div className="flex items-center gap-2 px-1">
          <Search size={16} className="flex-none text-content-tertiary" />
          <div className="flex-1 min-w-0 relative">
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onFocus={() => setFocused(true)}
              onBlur={() => setTimeout(() => setFocused(false), 120)}
              onKeyDown={onInputKey}
              className="w-full bg-transparent border-none outline-none text-[14px] text-content-primary placeholder:text-transparent"
            />
            {!query && (
              <span
                className={clsx(
                  'pointer-events-none absolute inset-0 flex items-center text-[14px] overflow-hidden transition-opacity duration-500 ease-out',
                  hintFading ? 'opacity-0 translate-y-[-2px]' : 'opacity-100 translate-y-0',
                )}
                style={{ transitionProperty: 'opacity, transform' }}
              >
                <ShimmerText text={SHIMMER_HINTS[hintIdx]} />
              </span>
            )}
          </div>

          {query && (
            <button
              type="button"
              onClick={() => { setQuery(''); inputRef.current?.focus(); }}
              className="flex-none w-6 h-6 flex items-center justify-center rounded-full hover:bg-white/5 text-content-tertiary"
              aria-label="Изчисти"
            >
              <X size={14} />
            </button>
          )}

          <kbd className="hidden sm:inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[10px] font-mono rounded-md bg-white/5 text-content-tertiary border border-white/5">
            ⌘K
          </kbd>

          <button
            type="button"
            onClick={() => {
              const item = flatResults[activeIdx];
              if (item) activate(item);
            }}
            disabled={flatResults.length === 0}
            className="flex-none w-8 h-8 flex items-center justify-center rounded-full bg-[color:var(--oe-accent)] hover:bg-[color:var(--oe-accent-hot)] text-[color:oklch(0.14_0.012_260)] disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            aria-label="Изпрати"
          >
            <ArrowUp size={14} strokeWidth={2.5} />
          </button>
        </div>
      </div>
    </div>
  );
}

function GroupLabel({ label }: { label: string }) {
  return (
    <div className="px-4 pt-3 pb-1 text-[10px] font-semibold uppercase tracking-widest text-content-tertiary">
      {label}
    </div>
  );
}

function ResultRow({
  active,
  onClick,
  icon,
  label,
  meta,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  meta?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={clsx(
        'w-full flex items-center gap-3 px-4 py-2 text-left transition-colors',
        active ? 'bg-[color:var(--oe-accent-soft-bg)]' : 'hover:bg-white/5',
      )}
    >
      <span
        className={clsx(
          'flex-none w-6 h-6 rounded-full flex items-center justify-center',
          active ? 'bg-[color:var(--oe-accent-soft-bg)] text-[color:var(--oe-accent)]' : 'bg-white/5 text-content-secondary',
        )}
      >
        {icon}
      </span>
      <span className="flex-1 min-w-0 text-sm text-content-primary truncate">{label}</span>
      {meta && (
        <span className="flex-none text-[11px] text-content-tertiary truncate max-w-[180px]">{meta}</span>
      )}
    </button>
  );
}

function ShimmerText({ text }: { text: string }) {
  return (
    <span className="kcc-shimmer truncate block w-full" key={text}>
      {text}
    </span>
  );
}
