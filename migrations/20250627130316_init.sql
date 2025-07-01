CREATE EXTENSION timescaledb;
CREATE EXTENSION "uuid-ossp";

CREATE TYPE event_type AS ENUM ('mqtt','unifi');
CREATE TYPE unifi_state AS ENUM ('connected', 'disconnected');

CREATE TABLE events (
    id UUID DEFAULT uuid_generate_v4() NOT NULL,
    raw_data JSONB NOT NULL,
    event_type event_type NOT NULL,
    "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE door_sensor (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  contact BOOLEAN NOT NULL,
  battery BIGINT NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE light (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  state TEXT NOT NULL,
  brightness BIGINT NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE smart_switch (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  voltage BIGINT NOT NULL,
  power BIGINT NOT NULL,
  current BIGINT NOT NULL,
  energy DOUBLE PRECISION NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE temperature_sensor (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  ieee_addr TEXT NOT NULL,
  temperature DOUBLE PRECISION NOT NULL,
  battery BIGINT NOT NULL,
  humidity DOUBLE PRECISION NOT NULL,
  pressure DOUBLE PRECISION NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE unifi_clients (
  event_id UUID NOT NULL,
  name TEXT NOT NULL,
  id TEXT NOT NULL,
  state unifi_state NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE known_devices (
  ieee_addr TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

