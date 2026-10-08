CREATE TYPE air_purifier_mode AS ENUM ('manual', 'sleep', 'auto');

CREATE TABLE air_purifier_events (
  event_id UUID NOT NULL,
  device_id TEXT NOT NULL,
  is_on BOOLEAN NOT NULL,
  mode air_purifier_mode,
  speed INTEGER,
  pm25 DOUBLE PRECISION,
  filter_life DOUBLE PRECISION,
  display BOOLEAN,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX air_purifier_events_device_id_time_idx
  ON air_purifier_events (device_id, "time" DESC);

CREATE TABLE latest_air_purifier_state (
  device_id TEXT PRIMARY KEY,
  is_on BOOLEAN NOT NULL,
  mode air_purifier_mode,
  speed INTEGER,
  pm25 DOUBLE PRECISION,
  filter_life DOUBLE PRECISION,
  display BOOLEAN,
  changed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
