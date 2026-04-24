import { create } from 'zustand';

type Theme = 'light' | 'dark' | 'system';

interface ThemeState {
  theme: Theme;
  resolved: 'light' | 'dark';
  init: () => void;
  setTheme: (t: Theme) => void;
  toggle: () => void;
}

function resolveTheme(t: Theme): 'light' | 'dark' {
  if (t === 'system') {
    return typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return t;
}

function applyTheme(resolved: 'light' | 'dark') {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('dark', resolved === 'dark');
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  theme: 'dark',
  resolved: 'dark',
  init: () => {
    const saved = typeof localStorage !== 'undefined' ? (localStorage.getItem('kcc_theme') as Theme | null) : null;
    const theme = saved || 'dark';
    const resolved = resolveTheme(theme);
    applyTheme(resolved);
    set({ theme, resolved });
  },
  setTheme: (theme) => {
    const resolved = resolveTheme(theme);
    applyTheme(resolved);
    if (typeof localStorage !== 'undefined') localStorage.setItem('kcc_theme', theme);
    set({ theme, resolved });
  },
  toggle: () => {
    const cur = get().resolved;
    const next = cur === 'dark' ? 'light' : 'dark';
    applyTheme(next);
    if (typeof localStorage !== 'undefined') localStorage.setItem('kcc_theme', next);
    set({ theme: next, resolved: next });
  },
}));
