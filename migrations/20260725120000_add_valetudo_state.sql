CREATE TABLE valetudo_events (
  event_id UUID NOT NULL,
  identifier TEXT NOT NULL,
  state TEXT,
  battery_level INT,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX valetudo_events_identifier_time_idx
  ON valetudo_events (identifier, "time" DESC);

CREATE TABLE latest_valetudo_state (
  identifier TEXT PRIMARY KEY,
  state TEXT,
  battery_level INT,
  fan_speed TEXT,
  current_clean_area DOUBLE PRECISION,
  clean_count INT,
  attributes JSONB,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
