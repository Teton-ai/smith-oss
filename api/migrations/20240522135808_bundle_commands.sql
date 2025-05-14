create table command_bundles
(
    uuid        uuid                     default gen_random_uuid() not null
        constraint command_bundles_pk
            primary key,
    created_on  timestamp with time zone default now()             not null
);

alter table command2_queue
    add bundle uuid not null;

alter table command2_queue
    add constraint command2_queue_command_bundles_uuid_fk
        foreign key (bundle) references command_bundles;
