-- Add migration script here
CREATE TABLE command2_queue (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    cmd JSON NOT NULL,
    continue_on_error BOOLEAN NOT NULL,
    canceled BOOLEAN NOT NULL DEFAULT FALSE,
    fetched  BOOLEAN NOT NULL DEFAULT FALSE,
    fetched_at TIMESTAMP NULL,
    FOREIGN KEY (device_id) REFERENCES device(id)
);

CREATE INDEX idx_command2_queue_device_id ON command2_queue(device_id);

CREATE TABLE command2_response (
    id SERIAL PRIMARY KEY,
    device_id INTEGER NOT NULL,
    command_id INTEGER,
    response JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (device_id) REFERENCES device(id),
    FOREIGN KEY (command_id) REFERENCES command2_queue(id)
);

CREATE INDEX idx_command2_response_device_id ON command2_response(device_id);
CREATE INDEX idx_command2_response_command_id ON command2_response(command_id);
