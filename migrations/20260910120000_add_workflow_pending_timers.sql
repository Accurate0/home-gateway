CREATE TABLE workflow_pending_timer (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  workflow TEXT NOT NULL,
  timer_kind TEXT NOT NULL CHECK (timer_kind IN ('hold', 'delay')),
  subject_kind TEXT NOT NULL,
  subject_entity TEXT NOT NULL,
  event_id UUID NOT NULL,
  vars JSONB NOT NULL,
  armed_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
  fire_at TIMESTAMP WITH TIME ZONE NOT NULL,
  UNIQUE (workflow, timer_kind)
);

CREATE INDEX idx_workflow_pending_timer_subject
  ON workflow_pending_timer (subject_kind, subject_entity);
