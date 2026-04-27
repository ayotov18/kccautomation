-- ═══════════════════════════════════════════════════════════
-- Multi-module drawing support
--
-- Many wood-cabin / modular sheets pack two or more independent floor plans
-- into one DWG laid out side-by-side. Treating that as one structure
-- produces a single KSS sized for one cabin, when the project actually
-- needs three. This migration adds:
--
--   1. drawing_structures — one row per detected spatial module, with
--      bbox + a human-friendly label extracted from drawing annotations.
--   2. structure_id columns on the existing per-drawing extracted-data
--      tables so layer/dim/annotation rows can be filtered to a module.
--   3. structure_id on kss_line_items so a single KSS report can carry
--      per-module line items, and the recap rolls up by structure.
--
-- All structure_id columns are nullable: legacy drawings without detected
-- structures (single-module) keep working unchanged. New uploads tag every
-- row with a real structure_id.
-- ═══════════════════════════════════════════════════════════

CREATE TABLE drawing_structures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    drawing_id UUID NOT NULL REFERENCES drawings(id) ON DELETE CASCADE,
    structure_index INT NOT NULL,
    label TEXT NOT NULL,
    bbox_min_x DOUBLE PRECISION NOT NULL,
    bbox_min_y DOUBLE PRECISION NOT NULL,
    bbox_max_x DOUBLE PRECISION NOT NULL,
    bbox_max_y DOUBLE PRECISION NOT NULL,
    entity_count INT NOT NULL DEFAULT 0,
    dimension_count INT NOT NULL DEFAULT 0,
    annotation_count INT NOT NULL DEFAULT 0,
    UNIQUE (drawing_id, structure_index)
);
CREATE INDEX idx_drawing_structures_drawing ON drawing_structures(drawing_id);

-- Tag extracted-data rows with their owning structure (nullable for legacy
-- drawings that pre-date structure detection).
ALTER TABLE drawing_layers
    ADD COLUMN structure_id UUID REFERENCES drawing_structures(id) ON DELETE SET NULL;
ALTER TABLE drawing_blocks
    ADD COLUMN structure_id UUID REFERENCES drawing_structures(id) ON DELETE SET NULL;
ALTER TABLE drawing_dimensions
    ADD COLUMN structure_id UUID REFERENCES drawing_structures(id) ON DELETE SET NULL;
ALTER TABLE drawing_annotations
    ADD COLUMN structure_id UUID REFERENCES drawing_structures(id) ON DELETE SET NULL;

CREATE INDEX idx_drawing_layers_structure ON drawing_layers(structure_id);
CREATE INDEX idx_drawing_blocks_structure ON drawing_blocks(structure_id);
CREATE INDEX idx_drawing_dims_structure ON drawing_dimensions(structure_id);
CREATE INDEX idx_drawing_anns_structure ON drawing_annotations(structure_id);

-- Per-line-item structure tagging. The KSS report is unified (one report
-- per drawing, same as today); each line item carries the structure_id it
-- belongs to. Frontend groups by structure_id to render per-module tabs;
-- a recap rolls subtotals up across structures.
ALTER TABLE kss_line_items
    ADD COLUMN structure_id UUID REFERENCES drawing_structures(id) ON DELETE SET NULL;
ALTER TABLE kss_line_items
    ADD COLUMN structure_label TEXT;
CREATE INDEX idx_kss_items_structure ON kss_line_items(report_id, structure_id);
