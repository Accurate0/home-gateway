CREATE TABLE push_notification (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  tag TEXT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  category TEXT NOT NULL,
  actions JSONB NOT NULL,
  remind_after_secs BIGINT NOT NULL,
  max_reminders INT NOT NULL,
  send_count INT NOT NULL DEFAULT 1,
  next_reminder_at TIMESTAMP WITH TIME ZONE,
  acknowledged_at TIMESTAMP WITH TIME ZONE,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE INDEX idx_push_notification_pending_reminder
  ON push_notification (next_reminder_at)
  WHERE acknowledged_at IS NULL AND next_reminder_at IS NOT NULL;

CREATE INDEX idx_push_notification_created_at
  ON push_notification (created_at DESC);

CREATE TABLE push_notification_interaction (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  notification_id UUID NOT NULL REFERENCES push_notification (id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('acknowledged', 'opened', 'dismissed', 'snoozed', 'swiped')),
  at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE INDEX idx_push_notification_interaction_notification
  ON push_notification_interaction (notification_id, at);
