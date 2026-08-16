CREATE TABLE adhoc_cron_run (
    name TEXT PRIMARY KEY,
    last_run_at TIMESTAMPTZ NOT NULL,
    duration_ms BIGINT NOT NULL,
    rows_affected BIGINT NOT NULL,
    outcome TEXT NOT NULL
);
