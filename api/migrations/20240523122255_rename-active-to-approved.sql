-- Add migration script here
-- Rename the column
ALTER TABLE device RENAME COLUMN active TO approved;

-- Set the default value to false
ALTER TABLE device ALTER COLUMN approved SET DEFAULT false;

ALTER TABLE device ALTER COLUMN approved SET NOT NULL;