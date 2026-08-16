CREATE TABLE adhoc_task_run (
    ordinal BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    checksum TEXT NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    duration_ms BIGINT NOT NULL
);
