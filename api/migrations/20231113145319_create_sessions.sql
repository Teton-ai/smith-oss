-- Add migration script here
CREATE TABLE public."session" (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (now() + INTERVAL '1 hour') NOT NULL,
    last_used_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id)
);

