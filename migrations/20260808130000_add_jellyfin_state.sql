DROP TABLE latest_jellyfin_session;

CREATE TABLE jellyfin_state (
  user_name TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  device_name TEXT NOT NULL,
  client TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_name TEXT NOT NULL,
  item_type TEXT NOT NULL,
  series_name TEXT,
  season INTEGER,
  episode INTEGER,
  position_seconds DOUBLE PRECISION,
  runtime_seconds DOUBLE PRECISION,
  play_method TEXT,
  paused BOOLEAN NOT NULL,
  event_id UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
