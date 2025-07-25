CREATE TABLE woolworths_product_tracking (
  id UUID DEFAULT gen_random_uuid() NOT NULL,
  product_id BIGINT UNIQUE NOT NULL,
  notify_channel BIGINT NOT NULL,
  mentions BIGINT[] NOT NULL
);

INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (327603, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (679135, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (911076, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (909279, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (911019, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (911092, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (315125, 906076901406306355, '{}');
INSERT INTO woolworths_product_tracking (product_id, notify_channel, mentions) VALUES (324461, 906076901406306355, '{}');
