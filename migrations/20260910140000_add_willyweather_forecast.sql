CREATE TABLE willyweather_forecast_day (
  location TEXT NOT NULL,
  date DATE NOT NULL,
  precis_code TEXT NOT NULL,
  precis TEXT NOT NULL,
  min_temp INTEGER NOT NULL,
  max_temp INTEGER NOT NULL,
  uv_max DOUBLE PRECISION,
  rain_probability INTEGER,
  rain_start_range INTEGER,
  rain_end_range INTEGER,
  rain_range_code TEXT,
  wind_max_speed DOUBLE PRECISION,
  first_light TIMESTAMP WITH TIME ZONE,
  sunrise TIMESTAMP WITH TIME ZONE,
  sunset TIMESTAMP WITH TIME ZONE,
  last_light TIMESTAMP WITH TIME ZONE,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  PRIMARY KEY (location, date)
);

CREATE TABLE willyweather_forecast_hour (
  location TEXT NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE NOT NULL,
  temperature DOUBLE PRECISION,
  wind_speed DOUBLE PRECISION,
  wind_direction DOUBLE PRECISION,
  wind_direction_text TEXT,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  PRIMARY KEY (location, "time")
);

CREATE TABLE willyweather_forecast_day_history (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  location TEXT NOT NULL,
  date DATE NOT NULL,
  precis_code TEXT NOT NULL,
  precis TEXT NOT NULL,
  min_temp INTEGER NOT NULL,
  max_temp INTEGER NOT NULL,
  uv_max DOUBLE PRECISION,
  rain_probability INTEGER,
  rain_start_range INTEGER,
  rain_end_range INTEGER,
  rain_range_code TEXT,
  wind_max_speed DOUBLE PRECISION,
  first_light TIMESTAMP WITH TIME ZONE,
  sunrise TIMESTAMP WITH TIME ZONE,
  sunset TIMESTAMP WITH TIME ZONE,
  last_light TIMESTAMP WITH TIME ZONE,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX idx_willyweather_forecast_day_history_location_time
  ON willyweather_forecast_day_history(location, "time" DESC);

CREATE TABLE willyweather_forecast_hour_history (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  location TEXT NOT NULL,
  forecast_time TIMESTAMP WITH TIME ZONE NOT NULL,
  temperature DOUBLE PRECISION,
  wind_speed DOUBLE PRECISION,
  wind_direction DOUBLE PRECISION,
  wind_direction_text TEXT,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX idx_willyweather_forecast_hour_history_location_time
  ON willyweather_forecast_hour_history(location, "time" DESC);
