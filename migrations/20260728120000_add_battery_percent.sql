ALTER TABLE device_battery ALTER COLUMN battery_voltage DROP NOT NULL;
ALTER TABLE device_battery ADD COLUMN battery_percent DOUBLE PRECISION;

CREATE TABLE device_battery_latest (
  device_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  battery_percent DOUBLE PRECISION,
  battery_voltage DOUBLE PRECISION,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
