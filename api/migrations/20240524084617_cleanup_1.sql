DROP TABLE device_applications;
DROP TABLE releases;
DROP TABLE applications;
DROP TABLE auth;
DROP TABLE command_queue_response;
DROP TABLE command_queue;
DROP TABLE command;

ALTER TABLE device DROP COLUMN deployment;
DROP TABLE deployments;

DROP TABLE ping_session_notification;
DROP TABLE ping_session;
DROP TABLE release_devices2;
DROP TABLE release_kinds;
DROP TABLE report;
DROP TABLE session;
DROP TABLE sessions;
DROP TABLE user_roles;
DROP TABLE users;
DROP TABLE "user";
DROP TABLE role;
DROP TABLE utilization;
