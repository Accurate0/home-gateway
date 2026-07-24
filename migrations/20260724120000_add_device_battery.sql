CREATE TABLE device_battery (
  event_id UUID NOT NULL,
  device_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  battery_voltage DOUBLE PRECISION NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX device_battery_device_id_time_idx
  ON device_battery (device_id, "time" DESC);

CREATE TABLE eink_display (
  device_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  battery_voltage DOUBLE PRECISION,
  image_key TEXT,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
