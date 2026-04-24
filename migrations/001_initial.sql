-- Users and auth
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    name TEXT NOT NULL,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Drawings
CREATE TABLE drawings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    original_format TEXT NOT NULL,
    s3_key_original TEXT NOT NULL,
    s3_key_dxf TEXT,
    units TEXT,
    entity_count INT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Analysis jobs
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'queued',
    progress INT NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_drawing ON jobs(drawing_id);

-- Extracted features
CREATE TABLE features (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    feature_type TEXT NOT NULL,
    description TEXT NOT NULL,
    centroid_x DOUBLE PRECISION NOT NULL,
    centroid_y DOUBLE PRECISION NOT NULL,
    geometry_refs JSONB NOT NULL,
    properties JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_features_drawing ON features(drawing_id);
CREATE INDEX idx_features_type ON features(feature_type);

-- Dimensions linked to features
CREATE TABLE feature_dimensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_id UUID NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    dim_type TEXT NOT NULL,
    nominal_value DOUBLE PRECISION NOT NULL,
    tolerance_upper DOUBLE PRECISION,
    tolerance_lower DOUBLE PRECISION,
    raw_text TEXT,
    entity_id BIGINT NOT NULL
);

CREATE INDEX idx_feature_dims_feature ON feature_dimensions(feature_id);

-- GD&T frames linked to features
CREATE TABLE feature_gdt (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_id UUID NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    symbol TEXT NOT NULL,
    tolerance_value DOUBLE PRECISION NOT NULL,
    material_condition TEXT,
    datum_refs JSONB,
    entity_id BIGINT NOT NULL
);

CREATE INDEX idx_feature_gdt_feature ON feature_gdt(feature_id);

-- Datums
CREATE TABLE datums (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    label CHAR(1) NOT NULL,
    attached_feature_id UUID REFERENCES features(id),
    position_x DOUBLE PRECISION NOT NULL,
    position_y DOUBLE PRECISION NOT NULL,
    UNIQUE(drawing_id, label)
);

-- KCC classifications
CREATE TABLE kcc_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_id UUID NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    classification TEXT NOT NULL,
    score INT NOT NULL,
    factors JSONB NOT NULL,
    tolerance_chain JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_kcc_drawing ON kcc_results(drawing_id);
CREATE INDEX idx_kcc_classification ON kcc_results(classification);

-- Reports
CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    format TEXT NOT NULL,
    s3_key TEXT NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- KCC threshold configuration
CREATE TABLE kcc_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    thresholds JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
