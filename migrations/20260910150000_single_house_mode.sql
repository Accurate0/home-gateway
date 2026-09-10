DELETE FROM state WHERE key LIKE 'mode:%';

INSERT INTO state (key, value) VALUES ('mode', 'home')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
