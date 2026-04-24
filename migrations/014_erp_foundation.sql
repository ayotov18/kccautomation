-- ═══════════════════════════════════════════════════════════
-- ERP Foundation — Projects, Hierarchical BOQ, Costs, Assemblies,
-- Schedule, Cost Model, Validation, Tendering, CDE
-- ═══════════════════════════════════════════════════════════

-- Projects
CREATE TABLE IF NOT EXISTS oe_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    region TEXT NOT NULL DEFAULT 'BG',
    classification_standard TEXT DEFAULT 'sek',
    currency TEXT DEFAULT 'BGN',
    locale TEXT DEFAULT 'bg',
    status TEXT DEFAULT 'active',
    owner_id UUID REFERENCES users(id),
    project_code TEXT,
    phase TEXT,
    budget_estimate DOUBLE PRECISION,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_projects_owner ON oe_projects(owner_id);

-- Hierarchical BOQ
CREATE TABLE IF NOT EXISTS oe_boq (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES oe_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'draft',
    is_locked BOOLEAN DEFAULT false,
    estimate_type TEXT DEFAULT 'detailed',
    base_date TEXT,
    created_by UUID REFERENCES users(id),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_boq_project ON oe_boq(project_id);

-- BOQ Positions (hierarchical, self-referencing)
CREATE TABLE IF NOT EXISTS oe_boq_position (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boq_id UUID NOT NULL REFERENCES oe_boq(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES oe_boq_position(id) ON DELETE CASCADE,
    ordinal TEXT NOT NULL,
    description TEXT NOT NULL,
    unit TEXT,
    quantity TEXT DEFAULT '0',
    unit_rate TEXT DEFAULT '0',
    total TEXT DEFAULT '0',
    classification JSONB DEFAULT '{}',
    source TEXT DEFAULT 'manual',
    confidence DOUBLE PRECISION,
    cad_element_ids JSONB DEFAULT '[]',
    validation_status TEXT DEFAULT 'pending',
    sort_order INT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_boq_pos_boq ON oe_boq_position(boq_id);
CREATE INDEX IF NOT EXISTS idx_boq_pos_parent ON oe_boq_position(parent_id);
CREATE INDEX IF NOT EXISTS idx_boq_pos_sort ON oe_boq_position(boq_id, sort_order);

-- BOQ Markups (ordered overhead chain)
CREATE TABLE IF NOT EXISTS oe_boq_markup (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boq_id UUID NOT NULL REFERENCES oe_boq(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    markup_type TEXT DEFAULT 'percentage',
    category TEXT DEFAULT 'overhead',
    percentage TEXT DEFAULT '0',
    fixed_amount TEXT DEFAULT '0',
    apply_to TEXT DEFAULT 'direct_cost',
    sort_order INT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_boq_markup_boq ON oe_boq_markup(boq_id);

-- BOQ Snapshots (versioning)
CREATE TABLE IF NOT EXISTS oe_boq_snapshot (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    boq_id UUID NOT NULL REFERENCES oe_boq(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_boq_snap_boq ON oe_boq_snapshot(boq_id);

-- BOQ Activity Log (audit trail)
CREATE TABLE IF NOT EXISTS oe_boq_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID,
    boq_id UUID,
    user_id UUID REFERENCES users(id),
    action TEXT NOT NULL,
    target_type TEXT,
    target_id UUID,
    description TEXT,
    changes JSONB,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_boq_log_boq ON oe_boq_activity_log(boq_id, created_at DESC);

-- Cost Database (CWICR equivalent)
CREATE TABLE IF NOT EXISTS oe_costs_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL,
    description TEXT NOT NULL,
    unit TEXT,
    rate TEXT DEFAULT '0',
    currency TEXT DEFAULT 'BGN',
    source TEXT DEFAULT 'custom',
    region TEXT,
    classification JSONB DEFAULT '{}',
    components JSONB DEFAULT '[]',
    tags TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_costs_code_region ON oe_costs_item(code, region) WHERE region IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_costs_search ON oe_costs_item USING gin(description gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_costs_tags ON oe_costs_item USING gin(tags);

-- Assemblies (composite rate recipes)
CREATE TABLE IF NOT EXISTS oe_assemblies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT,
    name TEXT NOT NULL,
    description TEXT,
    unit TEXT,
    category TEXT,
    classification JSONB DEFAULT '{}',
    total_rate TEXT DEFAULT '0',
    currency TEXT DEFAULT 'BGN',
    regional_factors JSONB DEFAULT '{}',
    is_template BOOLEAN DEFAULT true,
    project_id UUID REFERENCES oe_projects(id),
    owner_id UUID REFERENCES users(id),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Assembly Components
CREATE TABLE IF NOT EXISTS oe_assembly_component (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assembly_id UUID NOT NULL REFERENCES oe_assemblies(id) ON DELETE CASCADE,
    cost_item_id UUID REFERENCES oe_costs_item(id),
    description TEXT,
    factor DOUBLE PRECISION DEFAULT 1.0,
    quantity DOUBLE PRECISION DEFAULT 1.0,
    unit TEXT,
    unit_cost DOUBLE PRECISION DEFAULT 0,
    total DOUBLE PRECISION DEFAULT 0,
    sort_order INT DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_asm_comp_asm ON oe_assembly_component(assembly_id);

-- Schedule (4D)
CREATE TABLE IF NOT EXISTS oe_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    schedule_type TEXT DEFAULT 'master',
    start_date TEXT,
    end_date TEXT,
    status TEXT DEFAULT 'draft',
    data_date TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Schedule Activities (CPM tasks)
CREATE TABLE IF NOT EXISTS oe_schedule_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES oe_schedule(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES oe_schedule_activity(id),
    name TEXT NOT NULL,
    wbs_code TEXT,
    start_date TEXT,
    end_date TEXT,
    duration_days INT DEFAULT 0,
    progress_pct TEXT DEFAULT '0',
    status TEXT DEFAULT 'not_started',
    activity_type TEXT DEFAULT 'task',
    dependencies JSONB DEFAULT '[]',
    resources JSONB DEFAULT '[]',
    boq_position_ids JSONB DEFAULT '[]',
    constraint_type TEXT DEFAULT 'as_soon_as_possible',
    constraint_date TEXT,
    early_start TEXT,
    early_finish TEXT,
    late_start TEXT,
    late_finish TEXT,
    total_float INT,
    free_float INT,
    is_critical BOOLEAN DEFAULT false,
    sort_order INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_sched_act_sched ON oe_schedule_activity(schedule_id);

-- Schedule Relationships
CREATE TABLE IF NOT EXISTS oe_schedule_relationship (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID NOT NULL REFERENCES oe_schedule(id) ON DELETE CASCADE,
    predecessor_id UUID NOT NULL REFERENCES oe_schedule_activity(id) ON DELETE CASCADE,
    successor_id UUID NOT NULL REFERENCES oe_schedule_activity(id) ON DELETE CASCADE,
    relationship_type TEXT DEFAULT 'FS',
    lag_days INT DEFAULT 0,
    UNIQUE(predecessor_id, successor_id)
);

-- Cost Model (5D EVM)
CREATE TABLE IF NOT EXISTS oe_costmodel_snapshot (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    period TEXT NOT NULL,
    planned_cost DOUBLE PRECISION DEFAULT 0,
    earned_value DOUBLE PRECISION DEFAULT 0,
    actual_cost DOUBLE PRECISION DEFAULT 0,
    forecast_eac DOUBLE PRECISION,
    spi DOUBLE PRECISION,
    cpi DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_cm_snap_proj ON oe_costmodel_snapshot(project_id, period);

-- Budget Lines
CREATE TABLE IF NOT EXISTS oe_costmodel_budget_line (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    boq_position_id UUID,
    activity_id UUID,
    category TEXT DEFAULT 'material',
    description TEXT,
    planned_amount DOUBLE PRECISION DEFAULT 0,
    committed_amount DOUBLE PRECISION DEFAULT 0,
    actual_amount DOUBLE PRECISION DEFAULT 0,
    forecast_amount DOUBLE PRECISION DEFAULT 0,
    period_start TEXT,
    period_end TEXT,
    currency TEXT DEFAULT 'BGN',
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Cash Flow
CREATE TABLE IF NOT EXISTS oe_costmodel_cash_flow (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    period TEXT NOT NULL,
    category TEXT DEFAULT 'total',
    planned_inflow DOUBLE PRECISION DEFAULT 0,
    planned_outflow DOUBLE PRECISION DEFAULT 0,
    actual_inflow DOUBLE PRECISION DEFAULT 0,
    actual_outflow DOUBLE PRECISION DEFAULT 0,
    cumulative_planned DOUBLE PRECISION DEFAULT 0,
    cumulative_actual DOUBLE PRECISION DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Validation Reports
CREATE TABLE IF NOT EXISTS oe_validation_report (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id),
    target_type TEXT NOT NULL,
    target_id UUID NOT NULL,
    rule_set TEXT NOT NULL,
    status TEXT NOT NULL,
    score DOUBLE PRECISION,
    total_rules INT DEFAULT 0,
    passed INT DEFAULT 0,
    warnings INT DEFAULT 0,
    errors INT DEFAULT 0,
    results JSONB NOT NULL DEFAULT '[]',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_val_target ON oe_validation_report(target_type, target_id);

-- Tendering
CREATE TABLE IF NOT EXISTS oe_tendering_package (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    boq_id UUID REFERENCES oe_boq(id),
    name TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'draft',
    due_date TEXT,
    sections JSONB DEFAULT '[]',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS oe_tendering_bid (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES oe_tendering_package(id) ON DELETE CASCADE,
    bidder_name TEXT NOT NULL,
    bidder_email TEXT,
    total_amount DOUBLE PRECISION,
    status TEXT DEFAULT 'submitted',
    bid_data JSONB DEFAULT '{}',
    notes TEXT,
    submitted_at TIMESTAMPTZ DEFAULT now()
);

-- CDE (Common Data Environment — ISO 19650)
CREATE TABLE IF NOT EXISTS oe_cde_document (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    document_type TEXT DEFAULT 'general',
    status TEXT DEFAULT 'wip',
    version INT DEFAULT 1,
    s3_key TEXT,
    file_size BIGINT,
    mime_type TEXT,
    tags TEXT[] DEFAULT '{}',
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

-- Takeoff (PDF measurement annotations)
CREATE TABLE IF NOT EXISTS oe_takeoff_document (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES oe_projects(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    s3_key TEXT,
    page_count INT,
    extracted_text TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE IF NOT EXISTS oe_takeoff_measurement (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES oe_takeoff_document(id) ON DELETE CASCADE,
    page_number INT NOT NULL,
    measurement_type TEXT NOT NULL,
    points JSONB NOT NULL DEFAULT '[]',
    measurement_value DOUBLE PRECISION,
    measurement_unit TEXT,
    scale_pixels_per_unit DOUBLE PRECISION,
    linked_boq_position_id UUID,
    label TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Link existing drawings to projects
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'drawings' AND column_name = 'project_id') THEN
        ALTER TABLE drawings ADD COLUMN project_id UUID REFERENCES oe_projects(id);
    END IF;
END$$;
