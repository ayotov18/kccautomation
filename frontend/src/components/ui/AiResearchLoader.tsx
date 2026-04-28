'use client';

/**
 * AI research loader. Centered hero composition: animated amber halo with
 * three orbiting dots, an italic-serif tagline, the dynamic Bulgarian
 * phase label, and a thin progress hairline. Used on the AI Price Research
 * page while Perplexity researches and Opus generates.
 */

import { Sparkles } from 'lucide-react';

interface AiResearchLoaderProps {
  title: string;
  /** Bulgarian/English phase line under the hero — what the system is doing. */
  subtitle?: string;
  /** 0–100 progress; pass undefined to render an indeterminate hairline. */
  progress?: number;
}

export function AiResearchLoader({ title, subtitle, progress }: AiResearchLoaderProps) {
  return (
    <div className="oe-fade-in flex flex-col items-center justify-center min-h-[60vh] px-6 py-12 text-center">
      {/* Halo + orbiting dots */}
      <div className="relative w-32 h-32 mb-8">
        {/* Soft amber glow behind the halo */}
        <div
          className="absolute inset-0 rounded-full blur-2xl opacity-50"
          style={{ background: 'radial-gradient(circle, var(--oe-accent) 0%, transparent 70%)' }}
        />
        {/* Outer slow-rotating hairline ring */}
        <div className="absolute inset-2 rounded-full border border-[color:var(--oe-accent)]/30 animate-[spin_8s_linear_infinite]" />
        {/* Inner faster gradient ring */}
        <div
          className="absolute inset-5 rounded-full border-2 border-transparent animate-[spin_2.5s_linear_infinite]"
          style={{
            borderImage:
              'conic-gradient(from 0deg, transparent 0%, var(--oe-accent) 35%, transparent 60%) 1',
            // Fallback for browsers without border-image support
            borderTopColor: 'var(--oe-accent)',
            borderRightColor: 'var(--oe-accent-hot)',
          }}
        />
        {/* Sparkle in the center */}
        <div className="absolute inset-0 flex items-center justify-center">
          <span
            className="kcc-pulse-glow flex items-center justify-center w-10 h-10 rounded-full"
            style={{
              background: 'var(--oe-accent-soft-bg)',
              color: 'var(--oe-accent)',
              boxShadow: '0 0 24px var(--oe-accent-soft-bg)',
            }}
          >
            <Sparkles size={18} strokeWidth={1.75} />
          </span>
        </div>
        {/* Three orbiting dots — each on a distinct rotating layer. */}
        <Orbit delay="0s" duration="4s" />
        <Orbit delay="-1.3s" duration="4s" />
        <Orbit delay="-2.6s" duration="4s" />
      </div>

      <h2 className="text-[28px] leading-[1.1] font-semibold tracking-[-0.025em] text-content-primary max-w-md">
        {title.split(' ').slice(0, -1).join(' ')}{' '}
        <span className="oe-display text-content-secondary">
          {title.split(' ').slice(-1)[0]}
        </span>
      </h2>

      {subtitle && (
        <p className="mt-3 text-[13px] text-content-tertiary max-w-md">{subtitle}</p>
      )}

      {/* Progress hairline */}
      <div className="mt-8 w-full max-w-sm">
        {progress === undefined ? (
          <div className="relative h-[2px] bg-[color:var(--oe-border-light)] rounded-full overflow-hidden">
            <div
              className="absolute inset-y-0 left-0 w-1/3 rounded-full"
              style={{
                background:
                  'linear-gradient(90deg, transparent 0%, var(--oe-accent) 50%, transparent 100%)',
                animation: 'kccIndeterminate 1.6s cubic-bezier(0.4, 0, 0.2, 1) infinite',
              }}
            />
          </div>
        ) : (
          <>
            <div className="relative h-[2px] bg-[color:var(--oe-border-light)] rounded-full overflow-hidden">
              <div
                className="absolute inset-y-0 left-0 rounded-full transition-all duration-500"
                style={{
                  width: `${progress}%`,
                  background:
                    'linear-gradient(90deg, var(--oe-accent) 0%, var(--oe-accent-hot) 100%)',
                }}
              />
            </div>
            <div className="mt-2 font-numeric text-[11px] text-content-quaternary tracking-wide">
              {Math.round(progress)}%
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function Orbit({ delay, duration }: { delay: string; duration: string }) {
  return (
    <div
      className="absolute inset-0"
      style={{
        animation: `spin ${duration} linear infinite`,
        animationDelay: delay,
      }}
    >
      <span
        className="absolute top-0 left-1/2 -translate-x-1/2 w-1.5 h-1.5 rounded-full"
        style={{
          background: 'var(--oe-accent)',
          boxShadow: '0 0 8px var(--oe-accent)',
        }}
      />
    </div>
  );
}
