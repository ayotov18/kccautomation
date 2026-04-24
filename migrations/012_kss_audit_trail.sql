-- KSS Generation Audit Trail
-- Captures data from every phase of KSS generation for debugging and traceability.
-- Two view modes (DEV/USER) consume the same data with different rendering.

CREATE TABLE IF NOT EXISTS kss_audit_trails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    job_id UUID NOT NULL,
    pipeline_mode TEXT NOT NULL,              -- 'rule_based', 'ai_merged', 'ai_full'
    total_duration_ms BIGINT NOT NULL DEFAULT 0,
    total_warnings INT NOT NULL DEFAULT 0,
    total_errors INT NOT NULL DEFAULT 0,
    overall_confidence DOUBLE PRECISION,
    audit_data JSONB NOT NULL,               -- full structured audit (KssAuditTrail)
    user_summary JSONB,                      -- pre-computed user-mode summary
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_drawing ON kss_audit_trails(drawing_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON kss_audit_trails(created_at DESC);
