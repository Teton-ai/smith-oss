CREATE TYPE network_type AS ENUM ('wifi', 'ethernet', 'dongle');

CREATE TABLE network (
    id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    network_type network_type NOT NULL,
    is_network_hidden BOOLEAN NOT NULL,
    ssid TEXT,
    name TEXT NOT NULL,
    description TEXT,

    -- Password is optional.
    password TEXT,

    -- Ensure the ssid field is set in case the network type is wifi.
    CHECK (network_type != 'wifi' OR ssid IS NOT NULL)
);
