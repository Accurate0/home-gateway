CREATE TYPE door_state AS ENUM ('open','closed');

CREATE TABLE derived_door_events (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  id TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  state door_state NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);
