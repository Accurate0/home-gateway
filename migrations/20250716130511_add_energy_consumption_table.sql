CREATE TABLE energy_consumption (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  energy_used DOUBLE PRECISION NOT NULL,
  solar_exported DOUBLE PRECISION NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE UNIQUE INDEX idx_energy_time
  ON energy_consumption(energy_used, solar_exported, time);
