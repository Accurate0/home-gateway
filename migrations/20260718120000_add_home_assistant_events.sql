CREATE TABLE home_assistant_events (
  event_id UUID NOT NULL,
  entity_id TEXT NOT NULL,
  state TEXT NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX home_assistant_events_entity_time_idx
  ON home_assistant_events (entity_id, "time" DESC);

CREATE TABLE latest_home_assistant_state (
  entity_id TEXT PRIMARY KEY,
  state TEXT NOT NULL,
  event_id UUID NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
