DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'deployment_status') THEN
            CREATE TYPE deployment_status AS ENUM ('in_progress', 'failed', 'canceled', 'done');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS deployment (
    id integer generated always as identity,
    release_id int4 NOT NULL,
    status deployment_status NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (release_id),
    FOREIGN KEY (release_id) REFERENCES release(id)
);

CREATE TABLE IF NOT EXISTS deployment_devices
(
    id integer generated always as identity,
    deployment_id int4 NOT NULL,
    device_id int4 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (deployment_id) REFERENCES deployment(id),
    FOREIGN KEY (device_id) REFERENCES device(id)
)
