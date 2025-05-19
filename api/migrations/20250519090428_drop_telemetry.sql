DO $$
    DECLARE
        table_name text;
    BEGIN
        FOR table_name IN
            SELECT tablename
            FROM pg_tables
            WHERE tablename LIKE 'telemetry%'
              AND schemaname = current_schema()
            LOOP
                EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(table_name) || ' CASCADE';
            END LOOP;
    END $$;
