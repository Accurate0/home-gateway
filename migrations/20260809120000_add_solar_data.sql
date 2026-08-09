CREATE TABLE solar_data_tsdb (
  current_kwh DOUBLE PRECISION NOT NULL,
  raw_data JSONB NOT NULL,
  uv_level DOUBLE PRECISION,
  temperature DOUBLE PRECISION,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE solar_cached_token (
  id SERIAL PRIMARY KEY,
  login_data JSONB NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
