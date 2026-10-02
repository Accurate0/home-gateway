ALTER TABLE eink_display
    ADD COLUMN rtc_reported_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN rtc_reported_offset_ms BIGINT;
