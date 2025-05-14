-- Add migration script here

-- Conditional creation of the pgcrypto extension
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_extension
        WHERE extname = 'pgcrypto'
    ) THEN
        CREATE EXTENSION pgcrypto;
    END IF;
END
$$;

DO $$
BEGIN
    -- Check if the column 'token' does not exist in the 'device' table
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name='device' AND column_name='token'
    ) THEN
        -- Add the column if it does not exist
        ALTER TABLE device ADD COLUMN token TEXT;
    END IF;
END
$$;
