CREATE TABLE woolworths_product_price (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  product_id BIGINT NOT NULL,
  price DOUBLE PRECISION NOT NULL
);

CREATE UNIQUE INDEX idx_product_id
  ON woolworths_product_price(product_id);
