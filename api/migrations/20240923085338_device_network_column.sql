-- Add the new network_id column to the device table
ALTER TABLE device
ADD COLUMN network_id INTEGER;

-- Add the foreign key constraint to the network_id column
ALTER TABLE device
ADD CONSTRAINT fk_network_id
FOREIGN KEY (network_id)
REFERENCES network(id);
