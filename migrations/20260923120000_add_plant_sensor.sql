CREATE TABLE plant_sensor (
  event_id UUID NOT NULL,
  id TEXT NULL,
  name TEXT NOT NULL,
  address TEXT NOT NULL,
  soil_moisture DOUBLE PRECISION NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE latest_plant_sensor (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  entity_id TEXT UNIQUE NOT NULL,
  address TEXT UNIQUE NOT NULL,
  soil_moisture DOUBLE PRECISION NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE INDEX plant_sensor_address_time_idx
  ON plant_sensor (address, "time" DESC);
