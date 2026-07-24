CREATE UNIQUE INDEX idx_api_keys_name ON api_keys (name) WHERE revoked_at IS NULL;
