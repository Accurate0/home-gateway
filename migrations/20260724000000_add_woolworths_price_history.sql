CREATE TABLE woolworths_price_history (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  product_id BIGINT NOT NULL,
  price DOUBLE PRECISION NOT NULL,
  display_name TEXT NOT NULL DEFAULT '',
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX idx_woolworths_price_history_product_time
  ON woolworths_price_history(product_id, time DESC);
