ALTER TABLE release2 RENAME TO distribution;
ALTER TABLE distribution RENAME COLUMN codename to name;

ALTER TABLE release_packages2 RENAME TO distribution_packages;
ALTER TABLE distribution_packages RENAME COLUMN release_id to distribution_id;
