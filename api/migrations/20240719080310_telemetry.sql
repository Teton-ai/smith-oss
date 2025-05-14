-- Create the telemetry table
CREATE TABLE telemetry (
    id INT GENERATED ALWAYS AS IDENTITY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    serial_number TEXT NOT NULL,
    data JSONB NOT NULL
);

-- Create indexes on the telemetry table
CREATE INDEX idx_telemetry_timestamp ON telemetry (timestamp);
CREATE INDEX idx_telemetry_serial_number ON telemetry (serial_number);
