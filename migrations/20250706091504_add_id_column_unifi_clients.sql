ALTER TABLE unifi_clients SET (timescaledb.compress = false);

ALTER TABLE unifi_clients
ADD COLUMN relay_id uuid NOT NULL DEFAULT gen_random_uuid();

ALTER TABLE unifi_clients SET (timescaledb.compress = true);
