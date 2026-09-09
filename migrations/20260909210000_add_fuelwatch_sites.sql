CREATE TABLE fuelwatch_site (
  site_id INTEGER PRIMARY KEY,
  site_name TEXT NOT NULL,
  brand TEXT NOT NULL,
  suburb TEXT NOT NULL,
  postcode INTEGER NOT NULL,
  address TEXT NOT NULL,
  price DOUBLE PRECISION NOT NULL,
  price_tomorrow DOUBLE PRECISION,
  latitude DOUBLE PRECISION NOT NULL,
  longitude DOUBLE PRECISION NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE INDEX idx_fuelwatch_site_postcode_price
  ON fuelwatch_site(postcode, price);

CREATE TABLE fuelwatch_price_history (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  site_id INTEGER NOT NULL,
  postcode INTEGER NOT NULL,
  price DOUBLE PRECISION NOT NULL,
  price_tomorrow DOUBLE PRECISION,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX idx_fuelwatch_price_history_site_time
  ON fuelwatch_price_history(site_id, "time" DESC);
