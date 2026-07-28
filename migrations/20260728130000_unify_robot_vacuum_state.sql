CREATE TABLE robot_vacuum_events (
  event_id UUID NOT NULL,
  device_id TEXT NOT NULL,
  source TEXT NOT NULL,
  state TEXT,
  battery_level INT,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX robot_vacuum_events_device_id_time_idx
  ON robot_vacuum_events (device_id, "time" DESC);

CREATE TABLE latest_robot_vacuum_state (
  device_id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  state TEXT,
  battery_level INT,
  fan_speed TEXT,
  current_clean_area DOUBLE PRECISION,
  clean_count INT,
  room TEXT,
  attributes JSONB,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO robot_vacuum_events (event_id, device_id, source, state, battery_level, "time")
SELECT event_id, identifier, 'valetudo', state, battery_level, "time"
FROM valetudo_events;

INSERT INTO latest_robot_vacuum_state
  (device_id, source, state, battery_level, fan_speed, current_clean_area, clean_count, attributes, updated_at)
SELECT identifier, 'valetudo', state, battery_level, fan_speed, current_clean_area, clean_count, attributes, updated_at
FROM latest_valetudo_state;

DROP TABLE valetudo_events;
DROP TABLE latest_valetudo_state;
