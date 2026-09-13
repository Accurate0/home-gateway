DROP TABLE unifi_clients;
DROP TABLE unifi_clients_mapping;

CREATE TABLE unifi_client_state (
  mac_address TEXT PRIMARY KEY,
  name TEXT,
  hostname TEXT,
  state unifi_state NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
