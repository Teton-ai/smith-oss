CREATE SCHEMA IF NOT EXISTS auth;

CREATE TABLE auth.users (
    id SERIAL PRIMARY KEY,
    auth0_user_id VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT now (),
    updated_at TIMESTAMPTZ DEFAULT now ()
);

CREATE TABLE auth.users_roles (
    user_id INT NOT NULL REFERENCES auth.users (id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'default',
    PRIMARY KEY (user_id, role)
);

CREATE OR REPLACE FUNCTION auth.assign_default_role()
RETURNS TRIGGER AS $$
BEGIN
    -- Insert the default role for the new user
    INSERT INTO auth.users_roles (user_id, role)
    VALUES (NEW.id, 'default')
    ON CONFLICT (user_id, role) DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_user_default_role
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE FUNCTION auth.assign_default_role();
