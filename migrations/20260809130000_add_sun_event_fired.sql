CREATE TABLE sun_event_fired (
    transition TEXT NOT NULL,
    offset_seconds BIGINT NOT NULL,
    fired_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (transition, offset_seconds)
);
