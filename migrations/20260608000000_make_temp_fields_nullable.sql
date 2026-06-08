-- esphome sensors report temperature/pressure on separate topics and may never report
-- humidity or battery, so these can no longer be required.
ALTER TABLE temperature_sensor ALTER COLUMN battery DROP NOT NULL;
ALTER TABLE temperature_sensor ALTER COLUMN humidity DROP NOT NULL;
ALTER TABLE temperature_sensor ALTER COLUMN pressure DROP NOT NULL;

ALTER TABLE latest_temperature_sensor ALTER COLUMN humidity DROP NOT NULL;
