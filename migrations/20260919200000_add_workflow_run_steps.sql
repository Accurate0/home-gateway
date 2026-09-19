ALTER TABLE workflow_runs ADD COLUMN trigger JSONB;

CREATE TABLE workflow_run_steps (
    run_id BIGINT NOT NULL REFERENCES workflow_runs (id) ON DELETE CASCADE,
    seq INT NOT NULL,
    depth SMALLINT NOT NULL,
    kind TEXT NOT NULL,
    outcome TEXT NOT NULL,
    guard TEXT,
    detail TEXT,
    error TEXT,
    duration_ms BIGINT NOT NULL,
    at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (run_id, seq)
);
