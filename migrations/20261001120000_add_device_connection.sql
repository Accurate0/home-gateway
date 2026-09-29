CREATE TABLE device_connection (
    device_id TEXT PRIMARY KEY,
    connected BOOLEAN NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL
);
