'use client';

import type { Feature, KccResult, KccClassification } from '@/types';

interface FeatureInspectorProps {
  feature: Feature | null;
  kccResult: KccResult | null;
}

const CLASSIFICATION_STYLES: Record<KccClassification, { bg: string; text: string; label: string }> = {
  kcc: { bg: 'bg-red-900/30 border-red-800', text: 'text-red-400', label: 'KCC' },
  important: { bg: 'bg-yellow-900/30 border-yellow-800', text: 'text-yellow-400', label: 'Important' },
  standard: { bg: 'bg-green-900/30 border-green-800', text: 'text-green-400', label: 'Standard' },
};

export function FeatureInspector({ feature, kccResult }: FeatureInspectorProps) {
  if (!feature) {
    return (
      <div className="h-full flex items-center justify-center p-6">
        <div className="text-center">
          <svg
            className="w-10 h-10 mx-auto text-gray-700 mb-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122"
            />
          </svg>
          <p className="text-sm text-gray-600">
            Click a feature marker on the drawing to inspect its details
          </p>
        </div>
      </div>
    );
  }

  const classification = kccResult?.classification ?? 'standard';
  const style = CLASSIFICATION_STYLES[classification];

  return (
    <div className="p-4 space-y-4">
      {/* Header */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs text-gray-500 font-mono">{feature.id}</span>
          <span className={`px-2 py-0.5 rounded text-xs font-bold border ${style.bg} ${style.text}`}>
            {style.label}
          </span>
        </div>
        <h2 className="text-lg font-semibold">{feature.description}</h2>
        <div className="flex items-center gap-2 mt-1">
          <span className="px-2 py-0.5 bg-gray-800 rounded text-xs text-gray-400 capitalize">
            {feature.feature_type}
          </span>
          {kccResult && (
            <span className={`text-sm font-bold ${style.text}`}>
              Score: {kccResult.score}
            </span>
          )}
        </div>
      </div>

      {/* Location */}
      <div className="bg-gray-900/50 rounded-lg p-3">
        <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">Location</h3>
        <div className="grid grid-cols-2 gap-2 text-sm">
          <div>
            <span className="text-gray-500">X: </span>
            <span className="font-mono">{feature.centroid_x.toFixed(3)}</span>
          </div>
          <div>
            <span className="text-gray-500">Y: </span>
            <span className="font-mono">{feature.centroid_y.toFixed(3)}</span>
          </div>
        </div>
      </div>

      {/* Properties */}
      {Object.keys(feature.properties).length > 0 && (
        <div className="bg-gray-900/50 rounded-lg p-3">
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">Properties</h3>
          <dl className="space-y-1">
            {Object.entries(feature.properties).map(([key, value]) => (
              <div key={key} className="flex items-center justify-between text-sm">
                <dt className="text-gray-500 capitalize">{key.replace(/_/g, ' ')}</dt>
                <dd className="font-mono text-gray-300">{String(value)}</dd>
              </div>
            ))}
          </dl>
        </div>
      )}

      {/* Scoring factors */}
      {kccResult && kccResult.factors.length > 0 && (
        <div>
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">
            Scoring Factors
          </h3>
          <div className="space-y-1.5">
            {kccResult.factors
              .sort((a, b) => b.points - a.points)
              .map((factor, i) => (
                <div
                  key={i}
                  className="bg-gray-900/50 rounded-lg px-3 py-2"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-gray-300 capitalize">
                      {factor.name.replace(/_/g, ' ')}
                    </span>
                    <span className="text-sm font-bold text-blue-400">
                      +{factor.points}
                    </span>
                  </div>
                  <p className="text-xs text-gray-500 mt-0.5">{factor.reason}</p>
                </div>
              ))}
          </div>
        </div>
      )}

      {/* Dimensions */}
      {feature.dimensions && feature.dimensions.length > 0 && (
        <div>
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">
            Dimensions
          </h3>
          <div className="space-y-1.5">
            {feature.dimensions.map((dim, i) => (
              <div key={i} className="bg-gray-900/50 rounded-lg px-3 py-2 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-gray-400 capitalize">{dim.dim_type}</span>
                  <span className="font-mono text-gray-200">
                    {dim.nominal_value.toFixed(3)}
                  </span>
                </div>
                {(dim.tolerance_upper !== null || dim.tolerance_lower !== null) && (
                  <div className="text-xs text-gray-500 mt-1 font-mono">
                    {dim.tolerance_upper !== null && `+${dim.tolerance_upper.toFixed(3)}`}
                    {dim.tolerance_lower !== null && ` / ${dim.tolerance_lower.toFixed(3)}`}
                  </div>
                )}
                {dim.raw_text && (
                  <div className="text-xs text-gray-600 mt-0.5">{dim.raw_text}</div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* GD&T */}
      {feature.gdt && feature.gdt.length > 0 && (
        <div>
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">
            GD&amp;T Controls
          </h3>
          <div className="space-y-1.5">
            {feature.gdt.map((gdt, i) => (
              <div key={i} className="bg-gray-900/50 rounded-lg px-3 py-2 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-gray-300 capitalize">
                    {gdt.symbol.replace(/_/g, ' ')}
                  </span>
                  <span className="font-mono text-gray-200">
                    {gdt.tolerance_value.toFixed(3)}
                  </span>
                </div>
                <div className="flex items-center gap-2 mt-1">
                  {gdt.material_condition && (
                    <span className="text-xs px-1.5 py-0.5 bg-gray-800 rounded text-gray-400 uppercase">
                      {gdt.material_condition}
                    </span>
                  )}
                  {gdt.datum_refs.length > 0 && (
                    <span className="text-xs text-gray-500">
                      Datums: {gdt.datum_refs.join(', ')}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Datum references */}
      {feature.datum_refs && feature.datum_refs.length > 0 && (
        <div className="bg-gray-900/50 rounded-lg p-3">
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">
            Datum References
          </h3>
          <div className="flex items-center gap-2">
            {feature.datum_refs.map((ref) => (
              <span
                key={ref}
                className="w-7 h-7 flex items-center justify-center rounded bg-blue-900/30 border border-blue-800 text-blue-400 text-sm font-bold"
              >
                {ref}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Tolerance chain */}
      {kccResult?.tolerance_chain && (
        <div className="bg-gray-900/50 rounded-lg p-3">
          <h3 className="text-xs text-gray-500 uppercase tracking-wider mb-2">
            Tolerance Chain
          </h3>
          <dl className="space-y-1 text-sm">
            <div className="flex justify-between">
              <dt className="text-gray-500">Chain length</dt>
              <dd className="font-mono">{kccResult.tolerance_chain.chain_length}</dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">Worst-case</dt>
              <dd className="font-mono text-yellow-400">
                {kccResult.tolerance_chain.accumulated_tolerance_wc.toFixed(4)}
              </dd>
            </div>
            <div className="flex justify-between">
              <dt className="text-gray-500">RSS</dt>
              <dd className="font-mono text-green-400">
                {kccResult.tolerance_chain.accumulated_tolerance_rss.toFixed(4)}
              </dd>
            </div>
          </dl>
          {kccResult.tolerance_chain.critical_path.length > 0 && (
            <div className="mt-2 pt-2 border-t border-gray-800">
              <span className="text-xs text-gray-500">Critical path: </span>
              <span className="text-xs font-mono text-gray-400">
                {kccResult.tolerance_chain.critical_path.join(' -> ')}
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
