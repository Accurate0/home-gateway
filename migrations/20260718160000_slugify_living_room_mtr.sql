-- The slugify migration (20260630110000) missed LIVING_ROOM_MTR, so the stale
-- latest_temperature_sensor row keeps the old entity_id while ieee_addr stays
-- apollo-mtr-1-livingroom. New reports insert entity_id living-room-mtr-1 against
-- the existing ieee_addr and trip the ieee_addr unique constraint. Rename to keep
-- ids continuous.

UPDATE latest_temperature_sensor SET entity_id = 'living-room-mtr-1' WHERE entity_id = 'LIVING_ROOM_MTR';

UPDATE temperature_sensor SET id = 'living-room-mtr-1' WHERE id = 'LIVING_ROOM_MTR';
