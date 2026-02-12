-- Add migration script here
CREATE TABLE latest_temperature_sensor (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  entity_id TEXT UNIQUE NOT NULL,
  ieee_addr TEXT UNIQUE NOT NULL,
  temperature DOUBLE PRECISION NOT NULL,
  battery BIGINT NULL,
  humidity DOUBLE PRECISION NOT NULL,
  pressure DOUBLE PRECISION NULL,
  pm25 BIGINT NULL,
  voc_index BIGINT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
