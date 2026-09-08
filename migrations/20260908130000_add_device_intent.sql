CREATE TABLE device_intent (
    id BIGSERIAL PRIMARY KEY,
    address TEXT NOT NULL,
    attributes JSONB NOT NULL,
    relative BOOLEAN NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'confirmed', 'failed', 'superseded')),
    attempts INT NOT NULL DEFAULT 0,
    event_id UUID NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_attempt_at TIMESTAMPTZ,
    settled_at TIMESTAMPTZ
);

CREATE INDEX device_intent_claimable_idx
    ON device_intent (last_attempt_at NULLS FIRST)
    WHERE status = 'pending';

CREATE INDEX device_intent_address_pending_idx
    ON device_intent (address, requested_at DESC)
    WHERE status = 'pending';

CREATE INDEX device_intent_settled_at_idx ON device_intent (settled_at);
