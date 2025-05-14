CREATE SCHEMA IF NOT EXISTS partman;

CREATE EXTENSION IF NOT EXISTS pg_partman WITH SCHEMA partman;

CREATE TABLE IF NOT EXISTS public.telemetry_partitioned (
  "timestamp" timestamptz NOT NULL,
  id integer GENERATED ALWAYS AS IDENTITY,
  serial_number text NOT NULL,
  data jsonb NOT NULL,
  service text NOT NULL,
  CONSTRAINT telemetry_partitioned_pkey PRIMARY KEY ("timestamp", id)
) PARTITION BY RANGE ("timestamp");

CREATE INDEX IF NOT EXISTS idx_telemetry_parent_serial_ts_svc
    ON public.telemetry_partitioned (serial_number, "timestamp" DESC, service);

SELECT partman.create_parent(
    p_parent_table := 'public.telemetry_partitioned',
    p_control := 'timestamp',
    p_interval := '1 day'
);

UPDATE partman.part_config
SET retention = '7 days'
WHERE parent_table = 'public.telemetry_partitioned';

SELECT partman.run_maintenance();
