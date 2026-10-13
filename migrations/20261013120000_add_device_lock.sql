CREATE TYPE device_lock_source AS ENUM ('physical', 'manual');

CREATE TABLE device_lock (
  device_id TEXT PRIMARY KEY,
  source device_lock_source NOT NULL,
  locked_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ
);
