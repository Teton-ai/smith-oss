create index if not exists telemetry_serial_number_timestamp_service_index
    on telemetry (serial_number asc, timestamp desc, service asc);
