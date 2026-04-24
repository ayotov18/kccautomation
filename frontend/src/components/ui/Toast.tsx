'use client';

import { useToastStore } from '@/lib/toastStore';
import { CheckCircle2, AlertCircle, AlertTriangle, Info, X } from 'lucide-react';

const icons = { success: CheckCircle2, error: AlertCircle, warning: AlertTriangle, info: Info };
const colors = {
  success: 'border-l-green-500 bg-green-950/50',
  error: 'border-l-red-500 bg-red-950/50',
  warning: 'border-l-yellow-500 bg-yellow-950/50',
  info: 'border-l-blue-500 bg-blue-950/50',
};

export function ToastContainer() {
  const { toasts, removeToast } = useToastStore();
  if (!toasts.length) return null;
  return (
    <div className="fixed bottom-4 right-4 z-[9999] flex flex-col gap-2 max-w-sm">
      {toasts.map((t) => {
        const Icon = icons[t.type];
        return (
          <div key={t.id} className={`flex items-start gap-3 px-4 py-3 rounded-lg border-l-4 backdrop-blur-sm ${colors[t.type]} oe-slide-in-right`}>
            <Icon className="w-5 h-5 shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium">{t.title}</p>
              {t.message && <p className="text-xs text-gray-400 mt-0.5">{t.message}</p>}
            </div>
            <button onClick={() => removeToast(t.id)} className="text-gray-500 hover:text-gray-300"><X className="w-4 h-4" /></button>
          </div>
        );
      })}
    </div>
  );
}
