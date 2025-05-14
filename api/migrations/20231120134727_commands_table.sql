--- Maintain table that exists there already
CREATE TABLE IF NOT EXISTS public."command_queue" (
    id SERIAL PRIMARY KEY,
    bundle integer NOT NULL,
    cmd_id integer NOT NULL,
    cmd_data text NOT NULL,
    timestamp_created timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    canceled boolean DEFAULT false NOT NULL,
    device_id integer NOT NULL,
    fetched boolean DEFAULT false NOT NULL
);
