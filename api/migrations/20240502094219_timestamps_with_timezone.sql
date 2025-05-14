-- Add migration script here
alter table command2_response
    alter column created_at type timestamp with time zone using created_at::timestamp with time zone;

alter table command2_queue
    alter column created_at type timestamp with time zone using created_at::timestamp with time zone;

alter table command2_queue
    alter column fetched_at type timestamp with time zone using fetched_at::timestamp with time zone;
