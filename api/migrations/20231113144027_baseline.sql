-- we don't know how to generate root <with-no-name> (class Root) :(
create sequence if not exists commands_id_seq
    as integer;

alter sequence commands_id_seq owner to fleetadmin;

create sequence if not exists commands_queue_id_seq
    as integer;

alter sequence commands_queue_id_seq owner to fleetadmin;

create table if not exists auth
(
    id             serial
        constraint auth_pk
            primary key,
    key            bytea                                  not null,
    created_on     timestamp with time zone default now() not null,
    modified_on    timestamp with time zone default now() not null,
    read_only      boolean                  default true  not null,
    note           text,
    console_access boolean                  default false not null,
    auth_sub       text
);

alter table auth
    owner to fleetadmin;

create table if not exists command
(
    id        integer default nextval('commands_id_seq'::regclass) not null
        constraint commands_pkey
            primary key,
    operation text                                                 not null,
    data      json                                                 not null
);

alter table command
    owner to fleetadmin;

alter sequence commands_id_seq owned by command.id;

create table if not exists variable_preset
(
    id          serial
        constraint variable_preset_pk
            primary key,
    title       text                                   not null,
    description text,
    variables   jsonb                                  not null,
    created_on  timestamp with time zone default now() not null,
    modified_on timestamp with time zone default now() not null
);

alter table variable_preset
    owner to fleetadmin;

create table if not exists tag
(
    id    serial
        primary key,
    name  text not null
        constraint tag_pk
            unique,
    color text
);

alter table tag
    owner to fleetadmin;

create table if not exists config
(
    id         serial
        constraint config_pk
            primary key,
    name       text,
    created_on timestamp with time zone default now() not null,
    data       jsonb                                  not null,
    note       text
);

alter table config
    owner to fleetadmin;

DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'bugbuster') THEN
    CREATE USER bugbuster;
  END IF;
END $$;

grant usage on schema public to bugbuster;
grant connect on database postgres to bugbuster;
grant select on config to bugbuster;

create table if not exists sessions
(
    id         serial
        primary key,
    "user"     serial,
    created_on timestamp with time zone default now(),
    expires_on timestamp with time zone default (now() + '08:00:00'::interval),
    token      text                     default concat(md5((random())::text), md5((random())::text))
);

alter table sessions
    owner to fleetadmin;

create table if not exists users
(
    id        serial
        primary key,
    remote_id uuid,
    role      text default 'reader'::text,
    info      text
);

alter table users
    owner to fleetadmin;

create table if not exists applications
(
    id                   serial
        constraint applications_pk
            primary key,
    title                text                            not null,
    description          text,
    color                text    default 'gray'::text    not null,
    category             text    default 'Service'::text not null,
    default_service_name text,
    default_folder       text    default '/root/'::text  not null,
    default_schedule     boolean default false           not null
);

alter table applications
    owner to fleetadmin;

create table if not exists release_kinds
(
    id                 serial
        constraint release_kinds_pk
            primary key,
    kind               text                  not null,
    production_allowed boolean default false not null,
    color              text    default 'gray'::text
);

alter table release_kinds
    owner to fleetadmin;

create table if not exists releases
(
    id          serial
        constraint releases_pk
            primary key,
    created_on  timestamp with time zone default now() not null,
    s3          text                                   not null,
    note        text,
    version     text,
    application integer
        constraint releases_applications_null_fk
            references applications
            on update cascade on delete restrict,
    kind        integer                                not null
        constraint releases_release_kinds_null_fk
            references release_kinds
            on update cascade on delete restrict,
    constraint releases_version_kind_application_uq
        unique (version, application)
);

alter table releases
    owner to fleetadmin;

create table if not exists deployments
(
    id         serial
        primary key,
    created_at timestamp with time zone not null,
    notes      text                     not null,
    config     text,
    hash       text
);

alter table deployments
    owner to fleetadmin;

create table if not exists device
(
    id            serial
        constraint device_pk
            primary key,
    serial_number text                                   not null
        constraint serial_number_k
            unique,
    wifi_mac      text,
    created_on    timestamp with time zone default now() not null,
    modified_on   timestamp with time zone default now() not null,
    last_ping     timestamp with time zone,
    note          text,
    active        boolean                  default false not null,
    deployment    integer                  default 1
        constraint deployment_id
            references deployments
);

alter table device
    owner to fleetadmin;

grant select on device to bugbuster;

create table if not exists ping_session
(
    id       serial
        constraint ping_session_pk
            primary key,
    device   integer                  not null
        constraint ping_session_device_null_fk
            references device
            on update cascade on delete restrict,
    start_ts timestamp with time zone not null,
    end_ts   timestamp with time zone not null
);

alter table ping_session
    owner to fleetadmin;

create table if not exists ping_session_notification
(
    ping_session    integer                                not null
        constraint ping_session_notification_pk
            primary key
        constraint ping_session_notification_ping_session_null_fk
            references ping_session,
    notification_at timestamp with time zone default now() not null,
    active          boolean                  default true  not null
);

alter table ping_session_notification
    owner to fleetadmin;

create table if not exists utilization
(
    id         serial
        constraint utilization_pk
            primary key,
    device     integer                                not null
        constraint utilization_device_null_fk
            references device
            on update cascade on delete restrict,
    created_on timestamp with time zone default now() not null,
    cpu        real,
    gpu        real,
    ram        real,
    disk       real,
    pwm        real,
    temp_cpu   real,
    temp_gpu   real,
    temp_fan   real,
    temp_wifi  real
);

alter table utilization
    owner to fleetadmin;

create index if not exists index_utilization_created_on
    on utilization (created_on);

create index if not exists index_utilization_device
    on utilization (device);

create index if not exists idx_utilization_device_id_created_on_composite
    on utilization (device asc, created_on desc);

create index if not exists idx_device_created_on
    on utilization (device asc, created_on desc);

create table if not exists command_queue
(
    id                integer                  default nextval('commands_queue_id_seq'::regclass) not null
        constraint commands_queue_pkey
            primary key,
    bundle            integer                                                                     not null,
    cmd_id            integer                                                                     not null
        constraint commands_queue_cmd_id_fkey
            references command,
    cmd_data          text                                                                        not null,
    timestamp_created timestamp with time zone default CURRENT_TIMESTAMP                          not null,
    canceled          boolean                  default false                                      not null,
    device_id         integer                                                                     not null
        constraint commands_queue_device_id_fkey
            references device,
    fetched           boolean                  default false                                      not null
);

comment on table command_queue is 'old commands queue';

alter table command_queue
    owner to fleetadmin;

alter sequence commands_queue_id_seq owned by command_queue.id;

create table if not exists report
(
    id            serial
        constraint report_pk
            primary key,
    created_on    timestamp with time zone default now() not null,
    device        integer                                not null
        constraint report_device_null_fk
            references device
            on update cascade on delete restrict,
    camera_bound  boolean,
    network       text,
    boot_on       timestamp with time zone,
    disk_total    real,
    ram_total     real,
    wifi_chip     boolean,
    fan           boolean,
    network_usage jsonb                    default '[]'::jsonb
);

alter table report
    owner to fleetadmin;

create table if not exists variable
(
    name        text                                   not null,
    value       text                                   not null,
    created_on  timestamp with time zone default now() not null,
    modified_on timestamp with time zone default now() not null,
    device      integer                                not null
        constraint variable_device_null_fk
            references device
            on update cascade on delete restrict,
    id          serial
        constraint variable_pk
            primary key
);

alter table variable
    owner to fleetadmin;

create unique index if not exists uniq_device_variablename
    on variable (device, name);

create table if not exists command_queue_response
(
    id           serial
        primary key,
    cmd_queue_id integer                                            not null
        references command_queue,
    timestamp    timestamp with time zone default CURRENT_TIMESTAMP not null,
    success      boolean                                            not null,
    info         text                                               not null
);

alter table command_queue_response
    owner to fleetadmin;

create table if not exists tag_device
(
    tag_id    integer not null
        references tag,
    device_id integer not null
        references device,
    primary key (tag_id, device_id)
);

alter table tag_device
    owner to fleetadmin;

create table if not exists config_device
(
    id         serial
        constraint config_device_pk
            primary key,
    config     integer                                not null
        constraint config_device_config_null_fk
            references config
            on update cascade on delete restrict,
    device     integer                                not null
        constraint config_device_device_null_fk
            references device
            on update cascade on delete restrict,
    created_on timestamp with time zone default now() not null
);

alter table config_device
    owner to fleetadmin;

grant select on config_device to bugbuster;

create table if not exists device_applications
(
    id          serial
        constraint device_applications_pk
            primary key,
    device      integer               not null
        constraint device_applications_device_null_fk
            references device,
    application integer               not null
        constraint device_applications_applications_null_fk
            references applications,
    release     integer               not null
        constraint device_applications_releases_null_fk
            references releases,
    pinned      boolean default false not null,
    constraint device_applications_const
        unique (application, device)
);

alter table device_applications
    owner to fleetadmin;

create table if not exists ledger
(
    id        serial,
    device_id integer not null
        references device,
    timestamp timestamp with time zone default now(),
    class     text,
    text      text,
    primary key (id, device_id)
);

alter table ledger
    owner to fleetadmin;

-- create table _sqlx_migrations
-- (
--     version        bigint                                 not null
--         primary key,
--     description    text                                   not null,
--     installed_on   timestamp with time zone default now() not null,
--     success        boolean                                not null,
--     checksum       bytea                                  not null,
--     execution_time bigint                                 not null
-- );

-- alter table _sqlx_migrations
--     owner to fleetadmin;
