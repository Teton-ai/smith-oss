-- To slowly move to the new API for commands, where type are enforced on API
ALTER TABLE public."command_queue"
ADD COLUMN cmd text;

