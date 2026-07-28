CREATE INDEX door_sensor_ieee_addr_time_idx
  ON door_sensor (ieee_addr, "time" DESC);

CREATE INDEX light_ieee_addr_time_idx
  ON light (ieee_addr, "time" DESC);

CREATE INDEX smart_switch_ieee_addr_time_idx
  ON smart_switch (ieee_addr, "time" DESC);

CREATE INDEX temperature_sensor_ieee_addr_time_idx
  ON temperature_sensor (ieee_addr, "time" DESC);

CREATE INDEX derived_door_events_id_time_idx
  ON derived_door_events (id, "time" DESC);
