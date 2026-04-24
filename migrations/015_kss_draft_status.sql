-- Add status column to kss_reports for draft/final workflow
ALTER TABLE kss_reports ADD COLUMN IF NOT EXISTS status TEXT DEFAULT 'final';
CREATE INDEX IF NOT EXISTS idx_kss_reports_status ON kss_reports(drawing_id, status);
