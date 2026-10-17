CREATE TABLE unraid_notification (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  subject TEXT NOT NULL,
  description TEXT NOT NULL,
  importance TEXT NOT NULL,
  link TEXT,
  forwarded BOOLEAN NOT NULL,
  received_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
