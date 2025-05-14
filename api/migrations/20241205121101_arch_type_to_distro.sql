ALTER TABLE distribution
    ADD COLUMN architecture text NOT NULL DEFAULT 'arm64';
