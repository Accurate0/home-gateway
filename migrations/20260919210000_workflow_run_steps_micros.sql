ALTER TABLE workflow_run_steps RENAME COLUMN duration_ms TO duration_us;

UPDATE workflow_run_steps SET duration_us = duration_us * 1000;
