CREATE TABLE workflow_runs (
    id BIGSERIAL PRIMARY KEY,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    event_id UUID NOT NULL,
    outcome TEXT NOT NULL,
    dry_run BOOLEAN NOT NULL,
    duration_ms BIGINT NOT NULL,
    error TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX workflow_runs_slug_started_at_idx ON workflow_runs (slug, started_at DESC);
CREATE INDEX workflow_runs_started_at_idx ON workflow_runs (started_at DESC);
