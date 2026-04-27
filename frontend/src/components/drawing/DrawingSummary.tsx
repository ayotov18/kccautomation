'use client';

/**
 * Bilingual AI drawing summary with inline editing.
 *
 * Replaces the 5-widget grid (Drawing Info / Entity Distribution /
 * Annotations / Blocks / Active Layers). One contentEditable surface,
 * EN ↔ BG segmented pill toggle, debounced auto-save. No "edit pencil"
 * — clicking the prose moves the caret in directly. No modal, no save
 * button on the editor itself; the only feedback is a tiny status line
 * underneath.
 *
 * Markdown subset rendered: paragraphs, **bold**, * bullet lists.
 * Keeps it predictable so user edits don't create exotic structures
 * the next AI regenerate has to handle.
 */

import {
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import { api } from '@/lib/api';

type Lang = 'en' | 'bg';

interface AiSummary {
  summary_en: string | null;
  summary_bg: string | null;
  generated_at: string | null;
  edited_at: string | null;
  model: string | null;
}

interface Props {
  drawingId: string;
}

const MIN_BODY = `*The AI summary has not been generated yet. Click "Generate" to produce a comprehensive bilingual summary of this drawing.*`;

export function DrawingSummary({ drawingId }: Props) {
  const [data, setData] = useState<AiSummary | null>(null);
  const [lang, setLang] = useState<Lang>('en');
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [saveState, setSaveState] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');

  const fetchSummary = useCallback(async () => {
    try {
      const res = await api.getAiSummary(drawingId);
      setData(res);
    } catch {
      setData({
        summary_en: null,
        summary_bg: null,
        generated_at: null,
        edited_at: null,
        model: null,
      });
    }
    setLoading(false);
  }, [drawingId]);

  useEffect(() => {
    fetchSummary();
  }, [fetchSummary]);

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      await api.regenerateAiSummary(drawingId);
      // Poll until both languages land.
      for (let i = 0; i < 60; i++) {
        await new Promise((r) => setTimeout(r, 2000));
        const fresh = await api.getAiSummary(drawingId);
        if (fresh.summary_en && fresh.summary_bg) {
          setData(fresh);
          break;
        }
      }
    } catch {
      /* keep generating=true so the user retries */
    }
    setGenerating(false);
  };

  // Debounced save on edit
  const saveTimer = useRef<NodeJS.Timeout | null>(null);
  const scheduleSave = useCallback(
    (next: { summary_en?: string; summary_bg?: string }) => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
      setSaveState('saving');
      saveTimer.current = setTimeout(async () => {
        try {
          await api.saveAiSummary(drawingId, next);
          setSaveState('saved');
          setTimeout(() => setSaveState('idle'), 1400);
        } catch {
          setSaveState('error');
        }
      }, 600);
    },
    [drawingId],
  );

  const handleEdit = (newText: string) => {
    if (!data) return;
    const next = { ...data };
    if (lang === 'en') next.summary_en = newText;
    else next.summary_bg = newText;
    next.edited_at = new Date().toISOString();
    setData(next);
    scheduleSave(lang === 'en' ? { summary_en: newText } : { summary_bg: newText });
  };

  const currentText = data
    ? (lang === 'en' ? data.summary_en : data.summary_bg) ?? ''
    : '';
  const hasContent = currentText.trim().length > 0;
  const isEdited = !!data?.edited_at;

  return (
    <section className="oe-card relative overflow-hidden">
      {/* Header row: title + lang toggle + actions */}
      <div className="flex items-center justify-between gap-4 px-6 pt-5 pb-3">
        <div className="flex items-center gap-3">
          <h2 className="text-[13px] font-medium text-content-tertiary tracking-[0.06em] uppercase">
            Summary
          </h2>
          {data?.model && (
            <span className="text-[11px] font-numeric text-content-tertiary">
              {data.model.split('/').pop()}
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <LangToggle lang={lang} onChange={setLang} />
          <button
            onClick={handleGenerate}
            disabled={generating}
            className="oe-btn-secondary oe-btn-sm"
            title={hasContent ? 'Regenerate summary (will overwrite)' : 'Generate summary'}
          >
            {generating ? 'Generating…' : hasContent ? 'Regenerate' : 'Generate'}
          </button>
        </div>
      </div>

      <div className="border-t border-border-light/30" />

      {/* Body */}
      <div className="px-6 py-6 min-h-[280px]">
        {loading ? (
          <div className="space-y-2 animate-pulse">
            <div className="h-3 w-3/4 bg-surface-tertiary rounded" />
            <div className="h-3 w-full bg-surface-tertiary rounded" />
            <div className="h-3 w-5/6 bg-surface-tertiary rounded" />
            <div className="h-3 w-2/3 bg-surface-tertiary rounded" />
          </div>
        ) : !hasContent ? (
          <EmptyState onGenerate={handleGenerate} generating={generating} />
        ) : (
          <EditableProse text={currentText} onChange={handleEdit} />
        )}
      </div>

      {/* Footer status — tiny */}
      {(data?.generated_at || saveState !== 'idle') && (
        <>
          <div className="border-t border-border-light/30" />
          <div className="px-6 py-2.5 flex items-center justify-between text-[11px]">
            <div className="text-content-tertiary">
              {data?.generated_at && (
                <>
                  Generated {new Date(data.generated_at).toLocaleString('en-GB', {
                    dateStyle: 'medium',
                    timeStyle: 'short',
                  })}
                  {isEdited && (
                    <>
                      {' · '}
                      <span className="text-content-secondary">edited by you</span>
                    </>
                  )}
                </>
              )}
            </div>
            <div className="font-numeric text-content-tertiary">
              {saveState === 'saving' && 'Saving…'}
              {saveState === 'saved' && 'Saved'}
              {saveState === 'error' && (
                <span className="text-semantic-error">Save failed</span>
              )}
            </div>
          </div>
        </>
      )}
    </section>
  );
}

/** Two-pill segmented control. */
function LangToggle({
  lang,
  onChange,
}: {
  lang: Lang;
  onChange: (l: Lang) => void;
}) {
  return (
    <div className="inline-flex items-center rounded-full border border-border-light/60 p-0.5 bg-surface-secondary/50">
      {(['en', 'bg'] as const).map((l) => {
        const active = l === lang;
        return (
          <button
            key={l}
            onClick={() => onChange(l)}
            className={`px-2.5 py-0.5 text-[11px] font-medium uppercase tracking-wider rounded-full transition-colors ${
              active
                ? 'bg-content-primary text-content-inverse'
                : 'text-content-tertiary hover:text-content-secondary'
            }`}
          >
            {l}
          </button>
        );
      })}
    </div>
  );
}

function EmptyState({
  onGenerate,
  generating,
}: {
  onGenerate: () => void;
  generating: boolean;
}) {
  return (
    <div className="flex flex-col items-center justify-center text-center py-10">
      <p className="font-display text-[15px] leading-[1.65] text-content-secondary max-w-md">
        {MIN_BODY.replace(/\*/g, '')}
      </p>
      <button
        onClick={onGenerate}
        disabled={generating}
        className="oe-btn-primary mt-5"
      >
        {generating ? 'Generating…' : 'Generate summary'}
      </button>
    </div>
  );
}

/**
 * ContentEditable that renders a tiny markdown subset and keeps the
 * raw markdown text in sync via the parent's `onChange`. We render once
 * from `text` then use plain DOM contentEditable; no React reconciliation
 * during typing (otherwise caret position drops on every keystroke).
 */
function EditableProse({
  text,
  onChange,
}: {
  text: string;
  onChange: (next: string) => void;
}) {
  const ref = useRef<HTMLDivElement | null>(null);

  // Re-render the prose only when external text changes (lang switch /
  // server save) — never during user typing.
  useEffect(() => {
    if (!ref.current) return;
    if (ref.current.dataset.text === text) return;
    ref.current.innerHTML = renderMarkdown(text);
    ref.current.dataset.text = text;
  }, [text]);

  const handleInput = () => {
    if (!ref.current) return;
    const md = htmlToMarkdown(ref.current);
    ref.current.dataset.text = md;
    onChange(md);
  };

  return (
    <div
      ref={ref}
      contentEditable
      suppressContentEditableWarning
      onInput={handleInput}
      spellCheck
      className="font-display text-[15px] leading-[1.7] text-content-primary outline-none focus:outline-none [&_p]:my-3 [&_strong]:font-semibold [&_strong]:text-content-primary [&_ul]:my-3 [&_ul]:pl-5 [&_li]:my-1 [&_li]:list-disc [&_li]:marker:text-content-tertiary"
    />
  );
}

/* ---------- minimal markdown subset ---------- */

function renderMarkdown(md: string): string {
  // Escape HTML
  const esc = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const lines = md.split(/\r?\n/);
  const out: string[] = [];
  let inList = false;
  let para: string[] = [];
  const flushPara = () => {
    if (para.length === 0) return;
    const joined = para.join(' ');
    out.push(`<p>${formatInline(esc(joined))}</p>`);
    para = [];
  };
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) {
      flushPara();
      if (inList) {
        out.push('</ul>');
        inList = false;
      }
      continue;
    }
    if (/^[-*]\s+/.test(line)) {
      flushPara();
      if (!inList) {
        out.push('<ul>');
        inList = true;
      }
      const item = line.replace(/^[-*]\s+/, '');
      out.push(`<li>${formatInline(esc(item))}</li>`);
      continue;
    }
    if (inList) {
      out.push('</ul>');
      inList = false;
    }
    para.push(line);
  }
  flushPara();
  if (inList) out.push('</ul>');
  return out.join('');
}

function formatInline(s: string): string {
  return s.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
}

function htmlToMarkdown(root: HTMLElement): string {
  const out: string[] = [];
  root.childNodes.forEach((node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent ?? '';
      if (text.trim()) out.push(text.trim());
      return;
    }
    if (!(node instanceof HTMLElement)) return;
    const tag = node.tagName.toLowerCase();
    if (tag === 'p' || tag === 'div') {
      out.push(serializeInline(node));
      out.push('');
    } else if (tag === 'ul') {
      node.querySelectorAll(':scope > li').forEach((li) => {
        out.push('- ' + serializeInline(li as HTMLElement));
      });
      out.push('');
    } else if (tag === 'br') {
      out.push('');
    } else {
      out.push(serializeInline(node));
    }
  });
  return out.join('\n').replace(/\n{3,}/g, '\n\n').trim();
}

function serializeInline(el: HTMLElement): string {
  let s = '';
  el.childNodes.forEach((n) => {
    if (n.nodeType === Node.TEXT_NODE) {
      s += n.textContent ?? '';
    } else if (n instanceof HTMLElement) {
      const t = n.tagName.toLowerCase();
      if (t === 'strong' || t === 'b') {
        s += `**${n.textContent ?? ''}**`;
      } else {
        s += n.textContent ?? '';
      }
    }
  });
  return s.trim();
}
