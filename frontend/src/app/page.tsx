'use client';

import { Suspense, useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useAuthStore } from '@/lib/store';
import { api } from '@/lib/api';

function AuthGate() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const redirectTo = searchParams.get('redirect') ?? '/dashboard';
  const { login, register } = useAuthStore();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isRegistering, setIsRegistering] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    if (!api.isAuthenticated) {
      setChecking(false);
      return;
    }
    // Validate token against API — clears stale tokens from DB resets
    api.listDrawings()
      .then(() => {
        router.replace(redirectTo);
      })
      .catch(() => {
        api.clearTokens();
        setChecking(false);
      });
  }, [router, redirectTo]);

  // Show nothing until we've checked auth (avoids hydration mismatch)
  if (checking) {
    return (
      <div className="min-h-screen flex flex-col">
<main className="flex-1 flex items-center justify-center">
          <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
        </main>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex flex-col">
<main className="flex-1 flex items-center justify-center px-6">
        <div className="w-full max-w-sm">
          <h1 className="text-2xl font-bold text-center mb-8">
            {isRegistering ? 'Create Account' : 'Sign In'}
          </h1>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="bg-semantic-error-bg border border-semantic-error/30 text-red-300 rounded-lg px-4 py-3 text-sm">
                {error}
              </div>
            )}

            <div>
              <label htmlFor="email" className="block text-sm font-medium text-content-secondary mb-1.5">
                Email
              </label>
              <input
                id="email"
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full bg-surface-elevated border border-border-light rounded-lg px-4 py-2.5 text-content-primary placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
                placeholder="you@company.com"
              />
            </div>

            <div>
              <label htmlFor="password" className="block text-sm font-medium text-content-secondary mb-1.5">
                Password
              </label>
              <input
                id="password"
                type="password"
                required
                minLength={8}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full bg-surface-elevated border border-border-light rounded-lg px-4 py-2.5 text-content-primary placeholder-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent"
                placeholder="Minimum 8 characters"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full oe-btn-primary disabled:bg-blue-800 disabled:text-blue-400 rounded-lg px-4 py-2.5 font-medium transition-colors"
            >
              {loading
                ? 'Please wait...'
                : isRegistering
                  ? 'Create Account'
                  : 'Sign In'}
            </button>
          </form>

          <div className="mt-6 text-center">
            <button
              onClick={() => {
                setIsRegistering(!isRegistering);
                setError(null);
              }}
              className="text-sm text-content-tertiary hover:text-content-primary transition-colors"
            >
              {isRegistering
                ? 'Already have an account? Sign in'
                : "Don't have an account? Register"}
            </button>
          </div>
        </div>
      </main>
    </div>
  );

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (isRegistering) {
        await register(email, password);
      } else {
        await login(email, password);
      }
      router.push(redirectTo);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setLoading(false);
    }
  }
}

export default function HomePage() {
  return (
    <Suspense>
      <AuthGate />
    </Suspense>
  );
}
