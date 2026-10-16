CREATE TABLE unraid_array_events (
  state TEXT NOT NULL,
  kilobytes_free BIGINT NOT NULL,
  kilobytes_used BIGINT NOT NULL,
  kilobytes_total BIGINT NOT NULL,
  parity_status TEXT NOT NULL,
  parity_progress BIGINT,
  parity_errors BIGINT,
  parity_running BOOLEAN NOT NULL,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE TABLE unraid_disk_events (
  disk TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT,
  temperature INTEGER,
  size_kb BIGINT,
  fs_size_kb BIGINT,
  fs_free_kb BIGINT,
  fs_used_kb BIGINT,
  errors BIGINT,
  spinning BOOLEAN,
  "time" TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
) WITH (
  tsdb.hypertable,
  tsdb.partition_column='time',
  tsdb.orderby='time DESC'
);

CREATE INDEX unraid_disk_events_disk_time_idx
  ON unraid_disk_events (disk, "time" DESC);
