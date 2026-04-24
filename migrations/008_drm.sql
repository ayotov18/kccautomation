-- Deterministic Retrieval Memory (DRM)
-- Enable trigram fuzzy matching
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Core learning table: one row per unique mapping decision
CREATE TABLE drm_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,

    -- Input signature
    artifact_type TEXT NOT NULL,
    input_key TEXT NOT NULL,
    input_key_normalized TEXT NOT NULL,

    -- Output mapping
    sek_code TEXT,
    sek_group TEXT,
    description_bg TEXT,
    unit TEXT,
    quantity_formula TEXT,

    -- Provenance
    source TEXT NOT NULL DEFAULT 'auto',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 0.5,
    times_confirmed INT NOT NULL DEFAULT 1,
    times_overridden INT NOT NULL DEFAULT 0,

    -- Metadata
    properties JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_drm_trgm ON drm_artifacts USING gin (input_key_normalized gin_trgm_ops);
CREATE INDEX idx_drm_type ON drm_artifacts(artifact_type);
CREATE INDEX idx_drm_user ON drm_artifacts(user_id);
CREATE INDEX idx_drm_sek ON drm_artifacts(sek_code);
CREATE INDEX idx_drm_confidence ON drm_artifacts(confidence DESC);
CREATE INDEX idx_drm_user_type_key ON drm_artifacts(user_id, artifact_type, input_key_normalized);

-- User corrections to generated KSS line items
CREATE TABLE kss_corrections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    job_id UUID REFERENCES jobs(id) ON DELETE SET NULL,

    original_sek_code TEXT,
    original_description TEXT,
    original_quantity DOUBLE PRECISION,
    original_unit TEXT,

    corrected_sek_code TEXT,
    corrected_description TEXT,
    corrected_quantity DOUBLE PRECISION,
    corrected_unit TEXT,

    correction_type TEXT NOT NULL,
    source_layer TEXT,
    source_block TEXT,
    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_corrections_user ON kss_corrections(user_id);
CREATE INDEX idx_corrections_drawing ON kss_corrections(drawing_id);

-- Audit log for every DRM retrieval action
CREATE TABLE drm_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    job_id UUID REFERENCES jobs(id) ON DELETE SET NULL,

    action TEXT NOT NULL,
    artifact_id UUID REFERENCES drm_artifacts(id) ON DELETE SET NULL,
    input_key TEXT NOT NULL,
    matched_sek_code TEXT,
    similarity_score DOUBLE PRECISION,
    times_confirmed INT,
    previous_confidence DOUBLE PRECISION,
    new_confidence DOUBLE PRECISION,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_drm_audit_drawing ON drm_audit_log(drawing_id);
CREATE INDEX idx_drm_audit_action ON drm_audit_log(action);
