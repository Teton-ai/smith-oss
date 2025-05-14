DO $$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE  rolname = 'fleetadmin') THEN
            CREATE ROLE fleetadmin;
        END IF;
    END
$$;
