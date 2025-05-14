ALTER TABLE telemetry ADD COLUMN service TEXT;

UPDATE telemetry SET service = 'plex' WHERE service IS NULL;

ALTER TABLE telemetry ALTER COLUMN service SET NOT NULL;

CREATE INDEX idx_telemetry_service ON telemetry (service);
