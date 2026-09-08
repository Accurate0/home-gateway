ALTER TABLE device_intent
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'light';

ALTER TABLE device_intent
    ALTER COLUMN kind DROP DEFAULT;

DROP INDEX device_intent_address_pending_idx;

CREATE INDEX device_intent_target_pending_idx
    ON device_intent (kind, address, requested_at DESC)
    WHERE status = 'pending';
