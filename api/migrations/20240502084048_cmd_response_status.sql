-- Add migration script here
alter table command2_response
    add status integer default 0 not null;
