CREATE TYPE appliance_state AS ENUM ('on','off');

CREATE TABLE appliances (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  id TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  state appliance_state NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);
