-- Add migration script here
CREATE TABLE public."user" (
    id serial PRIMARY KEY,
    username text NOT NULL,
    display_name text NOT NULL,
    email text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    auth_sub text
);

