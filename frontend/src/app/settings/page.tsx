'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { api } from '@/lib/api';
import { IndustryPresetSelector } from '@/components/config/IndustryPresetSelector';
import { ThresholdEditor } from '@/components/config/ThresholdEditor';
import { useConfigStore } from '@/lib/configStore';
import { DEFAULT_ANALYSIS_CONFIG } from '@/types/config';

const scoringRules = [
  { name: 'Tight tolerance', points: 3, condition: 'Tolerance <= 50% of typical for feature type' },
  { name: 'Very tight tolerance', points: 5, condition: 'Tolerance <= 25% of typical' },
  { name: 'Datum reference', points: 4, condition: 'Feature is referenced by a datum (A, B, C)' },
  { name: 'GD&T controlled', points: 3, condition: 'Feature has a feature control frame' },
  { name: 'Position tolerance', points: 4, condition: 'True position GD&T applied' },
  { name: 'Pattern member', points: 2, condition: 'Part of a bolt circle or linear array' },
  { name: 'Assembly interface', points: 5, condition: 'Mounting hole, alignment pin, or mating surface' },
  { name: 'Tolerance chain critical', points: 4, condition: 'On the critical path of a tolerance chain' },
  { name: 'Thread specification', points: 2, condition: 'Feature has thread callout' },
  { name: 'Multiple GD&T controls', points: 2, condition: 'Feature has 2+ feature control frames' },
  { name: 'Surface finish specified', points: 1, condition: 'Surface finish symbol attached' },
  { name: 'Datum feature', points: 5, condition: 'Feature IS a datum (A, B, or C)' },
];

export default function SettingsPage() {
  const router = useRouter();
  const {
    analysisConfig,
    setIndustry,
    setThresholds,
    loadUserDefaults,
    saveUserDefaults,
    loading,
  } = useConfigStore();

  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (!api.isAuthenticated) {
      router.replace('/?redirect=/settings');
      return;
    }
    loadUserDefaults();
  }, [router, loadUserDefaults]);

  const handleSave = async () => {
    await saveUserDefaults();
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleReset = () => {
    setIndustry(DEFAULT_ANALYSIS_CONFIG.industry);
    setThresholds(DEFAULT_ANALYSIS_CONFIG.thresholds);
  };

  return (
    <div className="min-h-screen flex flex-col">
<main className="flex-1 max-w-3xl mx-auto w-full px-6 py-8">
        <h1 className="text-2xl font-bold mb-8">Settings</h1>

        {/* Default Analysis Configuration */}
        <section className="mb-10">
          <h2 className="text-lg font-semibold mb-1">Default Analysis Configuration</h2>
          <p className="text-sm text-content-tertiary mb-6">
            Set the default industry and thresholds for new analyses. These can be overridden per-upload.
          </p>

          <div className="space-y-6 oe-card p-6">
            <div>
              <label className="block text-xs text-content-tertiary uppercase tracking-wider mb-2">
                Industry Preset
              </label>
              <IndustryPresetSelector
                selected={analysisConfig.industry}
                onSelect={setIndustry}
              />
            </div>

            <div>
              <label className="block text-xs text-content-tertiary uppercase tracking-wider mb-3">
                Scoring Thresholds
              </label>
              <ThresholdEditor
                thresholds={analysisConfig.thresholds}
                onChange={setThresholds}
              />
            </div>

            <div className="flex items-center gap-4 pt-2 border-t border-border-light">
              <button
                onClick={handleSave}
                disabled={loading}
                className="px-5 py-2 oe-btn-primary disabled:bg-blue-800 disabled:text-blue-400 rounded-lg text-sm font-medium transition-colors"
              >
                {loading ? 'Saving...' : saved ? 'Saved' : 'Save Defaults'}
              </button>
              <button
                onClick={handleReset}
                className="text-sm text-content-tertiary hover:text-content-primary transition-colors"
              >
                Reset to Factory Defaults
              </button>
            </div>
          </div>
        </section>

        {/* Scoring Rules Reference */}
        <section>
          <h2 className="text-lg font-semibold mb-1">Scoring Rules Reference</h2>
          <p className="text-sm text-content-tertiary mb-6">
            Every KCC classification is determined by summing these rule-based scoring factors.
            Features scoring above the KCC threshold are classified as Key Characteristics.
          </p>

          <div className="oe-card overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border-light text-left text-content-tertiary uppercase text-xs tracking-wider">
                  <th className="px-4 py-3 font-medium">Factor</th>
                  <th className="px-4 py-3 font-medium w-20 text-center">Points</th>
                  <th className="px-4 py-3 font-medium">Condition</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border-light">
                {scoringRules.map((rule) => (
                  <tr key={rule.name} className="hover:bg-surface-tertiary/30">
                    <td className="px-4 py-3 font-medium text-content-primary">{rule.name}</td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`inline-flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold ${
                          rule.points >= 4
                            ? 'bg-red-900/50 text-red-400'
                            : rule.points >= 2
                              ? 'bg-yellow-900/50 text-yellow-400'
                              : 'bg-surface-tertiary text-content-secondary'
                        }`}
                      >
                        {rule.points}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-content-tertiary">{rule.condition}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="mt-4 flex gap-6 text-xs text-content-tertiary">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-green-700/60" />
              Standard: 0 &ndash; {analysisConfig.thresholds.important_threshold - 1} pts
            </div>
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-yellow-600/60" />
              Important: {analysisConfig.thresholds.important_threshold} &ndash;{' '}
              {analysisConfig.thresholds.kcc_threshold - 1} pts
            </div>
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-red-600/60" />
              KCC: {analysisConfig.thresholds.kcc_threshold}+ pts
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
