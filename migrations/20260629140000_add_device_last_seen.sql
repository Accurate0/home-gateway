CREATE TABLE device_last_seen (
    device_key TEXT PRIMARY KEY,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    alerted_at TIMESTAMPTZ NULL
);
