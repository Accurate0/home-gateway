CREATE TABLE public_holiday (
  uid TEXT PRIMARY KEY,
  date DATE NOT NULL,
  name TEXT NOT NULL,
  public BOOLEAN NOT NULL,
  regions TEXT[] NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE INDEX idx_public_holiday_date ON public_holiday(date);
