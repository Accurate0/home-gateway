-- Device config ids moved from SCREAMING_SNAKE_CASE to hyphenated slugs.
-- latest_temperature_sensor upserts on entity_id while ieee_addr is UNIQUE, so
-- without renaming the stored ids the actor inserts a fresh entity_id against an
-- existing ieee_addr and trips the unique constraint. Rename the historical and
-- latest rows so ids stay continuous across the change.

UPDATE latest_temperature_sensor SET entity_id = CASE entity_id
  WHEN 'OUTDOOR' THEN 'outdoor'
  WHEN 'LIVING_ROOM' THEN 'living-room'
  WHEN 'BATHROOM' THEN 'bathroom'
  WHEN 'BEDROOM' THEN 'bedroom'
  WHEN 'HALLWAY_PLANT' THEN 'hallway-plant'
  ELSE entity_id
END;

UPDATE temperature_sensor SET id = CASE id
  WHEN 'OUTDOOR' THEN 'outdoor'
  WHEN 'LIVING_ROOM' THEN 'living-room'
  WHEN 'BATHROOM' THEN 'bathroom'
  WHEN 'BEDROOM' THEN 'bedroom'
  WHEN 'HALLWAY_PLANT' THEN 'hallway-plant'
  ELSE id
END;

UPDATE appliances SET id = CASE id
  WHEN 'WASHING_MACHINE' THEN 'washing-machine'
  ELSE id
END;

UPDATE derived_door_events SET id = CASE id
  WHEN 'FRONT_DOOR' THEN 'front-door'
  WHEN 'GARAGE_DOOR' THEN 'garage-door'
  ELSE id
END;
