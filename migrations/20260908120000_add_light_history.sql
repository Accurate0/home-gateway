CREATE TABLE light_history (
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  address TEXT NOT NULL,
  device_id TEXT,
  on_state BOOLEAN NOT NULL,
  brightness INTEGER,
  colour_temp INTEGER,
  colour TEXT,
  source TEXT NOT NULL,
  event_id UUID,
  CONSTRAINT light_history_source CHECK (source IN ('edge', 'sample'))
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX light_history_address_time_idx ON light_history (address, "time" DESC);

CREATE MATERIALIZED VIEW light_activity_30m
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('30 minutes', "time") AS bucket,
  address,
  avg(on_state::int) AS on_fraction,
  count(*) AS observations,
  count(*) FILTER (WHERE source = 'edge' AND on_state) AS turned_on,
  count(*) FILTER (WHERE source = 'edge' AND NOT on_state) AS turned_off
FROM light_history
GROUP BY bucket, address
WITH NO DATA;

SELECT add_continuous_aggregate_policy('light_activity_30m',
  start_offset => INTERVAL '7 days',
  end_offset => INTERVAL '30 minutes',
  schedule_interval => INTERVAL '30 minutes');
