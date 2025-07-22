CREATE TABLE unifi_clients_mapping (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  mac_address TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL
);
