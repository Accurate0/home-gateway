-- esphome plant sensors (Apollo PLT-1) report ambient light in lux and a UV
-- index alongside air temperature/humidity, so capture them on the temperature
-- sensor readings.
ALTER TABLE temperature_sensor ADD COLUMN lux DOUBLE PRECISION DEFAULT NULL;
ALTER TABLE temperature_sensor ADD COLUMN uv_index DOUBLE PRECISION DEFAULT NULL;

ALTER TABLE latest_temperature_sensor ADD COLUMN lux DOUBLE PRECISION DEFAULT NULL;
ALTER TABLE latest_temperature_sensor ADD COLUMN uv_index DOUBLE PRECISION DEFAULT NULL;
