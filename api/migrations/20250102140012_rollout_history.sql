CREATE TABLE IF NOT EXISTS device_release_upgrades (
    id integer generated always as identity,
    device_id int4 NOT NULL,
    previous_release_id int4 NOT NULL,
    upgraded_release_id int4 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (device_id) REFERENCES device(id),
    FOREIGN KEY (previous_release_id) REFERENCES release(id),
    FOREIGN KEY (upgraded_release_id) REFERENCES release(id)
);
