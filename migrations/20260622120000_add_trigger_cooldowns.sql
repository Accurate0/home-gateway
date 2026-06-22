CREATE TABLE trigger_cooldowns (
    name TEXT PRIMARY KEY,
    last_fired TIMESTAMPTZ NOT NULL
);
