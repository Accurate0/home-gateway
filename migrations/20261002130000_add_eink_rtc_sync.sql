ALTER TABLE eink_display
    ADD COLUMN rtc_synced_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN rtc_drift_ms BIGINT;
