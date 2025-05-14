-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Sequence and defined type
CREATE SEQUENCE IF NOT EXISTS release_new_id_seq;

-- Table Definition
CREATE TABLE "public"."release2" (
    "id" int4 NOT NULL DEFAULT nextval('release_new_id_seq'::regclass),
    "codename" text NOT NULL,
    "description" text,
    PRIMARY KEY ("id")
);

-- Sequence and defined type
CREATE SEQUENCE IF NOT EXISTS package2_id_seq;

-- Table Definition
CREATE TABLE "public"."package2" (
    "id" int4 NOT NULL DEFAULT nextval('package2_id_seq'::regclass),
    "name" text NOT NULL,
    "version" text NOT NULL,
    "file" text NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY ("id")
);

-- Table Definition
CREATE TABLE "public"."release_packages2" (
    "release_id" int4 NOT NULL,
    "package_id" int4 NOT NULL,
    CONSTRAINT "release_packages2_release_id_fkey" FOREIGN KEY ("release_id") REFERENCES "public"."release2"("id"),
    CONSTRAINT "release_packages2_package_id_fkey" FOREIGN KEY ("package_id") REFERENCES "public"."package2"("id"),
    PRIMARY KEY ("release_id","package_id")
);

-- Table Definition
CREATE TABLE "public"."release_devices2" (
    "release_id" int4 NOT NULL,
    "device_id" int4 NOT NULL,
    CONSTRAINT "release_devices2_device_id_fkey" FOREIGN KEY ("device_id") REFERENCES "public"."device"("id"),
    CONSTRAINT "release_devices2_release_id_fkey" FOREIGN KEY ("release_id") REFERENCES "public"."release2"("id"),
    PRIMARY KEY ("release_id","device_id")
);
