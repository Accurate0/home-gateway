CREATE TABLE jellyfin_playback_events (
  event_id UUID NOT NULL,
  state TEXT NOT NULL,
  session_id TEXT NOT NULL,
  user_name TEXT NOT NULL,
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
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX jellyfin_playback_events_user_time_idx
  ON jellyfin_playback_events (user_name, "time" DESC);

CREATE INDEX jellyfin_playback_events_item_time_idx
  ON jellyfin_playback_events (item_id, "time" DESC);

CREATE TABLE latest_jellyfin_session (
  session_id TEXT PRIMARY KEY,
  user_name TEXT NOT NULL,
  device_name TEXT NOT NULL,
  client TEXT NOT NULL,
  item_id TEXT,
  item_name TEXT,
  item_type TEXT,
  series_name TEXT,
  season INTEGER,
  episode INTEGER,
  position_seconds DOUBLE PRECISION,
  runtime_seconds DOUBLE PRECISION,
  play_method TEXT,
  paused BOOLEAN NOT NULL DEFAULT false,
  event_id UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
