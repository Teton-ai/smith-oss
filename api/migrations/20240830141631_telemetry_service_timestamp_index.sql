-- Add migration script here
create index if not exists telemetry_service_timestamp_index
    on telemetry (service asc, timestamp desc);
