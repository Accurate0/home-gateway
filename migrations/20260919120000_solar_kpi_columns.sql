ALTER TABLE solar_data_tsdb
  ADD COLUMN today_kwh DOUBLE PRECISION,
  ADD COLUMN month_kwh DOUBLE PRECISION,
  ADD COLUMN total_kwh DOUBLE PRECISION;

UPDATE solar_data_tsdb SET
  today_kwh = (raw_data -> 'data' -> 'kpi' ->> 'power')::double precision,
  month_kwh = (raw_data -> 'data' -> 'kpi' ->> 'month_generation')::double precision,
  total_kwh = (raw_data -> 'data' -> 'kpi' ->> 'total_power')::double precision;
