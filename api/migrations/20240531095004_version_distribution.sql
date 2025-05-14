CREATE SEQUENCE IF NOT EXISTS distribution_release_seq;

CREATE TABLE IF NOT EXISTS release (
    id int4 NOT NULL DEFAULT nextval('distribution_release_seq'),
    distribution_id int4 NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (distribution_id) REFERENCES distribution(id)
);

ALTER SEQUENCE distribution_release_seq OWNED BY release.id;

INSERT INTO release (distribution_id, version) SELECT distribution.id, '1.0.0' FROM distribution;

ALTER TABLE device
    ADD COLUMN release_id int4,
    ADD COLUMN target_release_id int4,
    ADD CONSTRAINT fk_release_id FOREIGN KEY (release_id) REFERENCES release(id),
    ADD CONSTRAINT fk_target_release_id FOREIGN KEY (target_release_id) REFERENCES release(id);

ALTER TABLE distribution_packages RENAME TO release_packages;

ALTER TABLE release_packages
    ADD COLUMN release_id int4,
    ADD CONSTRAINT fk_release_id FOREIGN KEY (release_id) REFERENCES release(id);

UPDATE release_packages rp SET release_id = r.id FROM release r WHERE rp.distribution_id = r.distribution_id;

ALTER TABLE release_packages
    ALTER COLUMN release_id SET NOT NULL;

ALTER TABLE release_packages DROP COLUMN distribution_id;
