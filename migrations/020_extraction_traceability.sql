-- Every KSS line item now carries its geometric provenance + confidence.
-- Research reference: the "traceability" and "1-10-100 rule" sections of the
-- backend QTO research brief. Downstream effect: the AI prompt knows what the
-- geometry actually measured vs what the extractor guessed, and the frontend
-- can highlight the source entity when a row is challenged.

ALTER TABLE kss_line_items
    ADD COLUMN IF NOT EXISTS source_entity_id   TEXT,
    ADD COLUMN IF NOT EXISTS source_layer       TEXT,
    ADD COLUMN IF NOT EXISTS centroid_x         DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS centroid_y         DOUBLE PRECISION,
    -- polyline_shoelace | linear_polyline | block_instance_count | wall_area_from_centerline
    -- | wall_volume_from_centerline | assumed_default | ai_inferred
    ADD COLUMN IF NOT EXISTS extraction_method  TEXT,
    ADD COLUMN IF NOT EXISTS geometry_confidence DOUBLE PRECISION DEFAULT 0.5,
    ADD COLUMN IF NOT EXISTS needs_review       BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_kss_line_items_needs_review
    ON kss_line_items(report_id, needs_review)
    WHERE needs_review = true;

-- Note: ai_kss_research_items is price-research-only (not geometry-backed), so
-- it does NOT need the traceability columns. Only kss_line_items — the final
-- report rows — carry source-entity provenance.
