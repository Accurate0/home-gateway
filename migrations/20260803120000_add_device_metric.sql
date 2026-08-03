CREATE TABLE device_metric (
  event_id UUID NOT NULL,
  address TEXT NOT NULL,
  device_id TEXT,
  metric TEXT NOT NULL,
  value DOUBLE PRECISION,
  text_value TEXT,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  CONSTRAINT device_metric_one_value CHECK (num_nonnulls(value, text_value) = 1)
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX device_metric_address_metric_time_idx
  ON device_metric (address, metric, "time" DESC);

CREATE TABLE latest_device_metric (
  address TEXT NOT NULL,
  metric TEXT NOT NULL,
  device_id TEXT,
  value DOUBLE PRECISION,
  text_value TEXT,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  PRIMARY KEY (address, metric),
  CONSTRAINT latest_device_metric_one_value CHECK (num_nonnulls(value, text_value) = 1)
);
