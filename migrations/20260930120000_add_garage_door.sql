CREATE TYPE garage_door_state AS ENUM ('open', 'opening', 'closed', 'closing');

CREATE TABLE garage_door_events (
  event_id UUID NOT NULL,
  device_id TEXT NOT NULL,
  state garage_door_state NOT NULL,
  contact BOOLEAN,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX garage_door_events_device_id_time_idx
  ON garage_door_events (device_id, "time" DESC);

CREATE TABLE latest_garage_door_state (
  device_id TEXT PRIMARY KEY,
  state garage_door_state NOT NULL,
  contact BOOLEAN,
  changed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
