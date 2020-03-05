CREATE TABLE requests (
    group_id SERIAL REFERENCES groups,
    user_uuid UUID NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT NOW(),
    request_expiration TIMESTAMP DEFAULT (NOW() + INTERVAL '1 week'),
    PRIMARY KEY (group_id, user_uuid)
);

ALTER TYPE log_target_type RENAME TO log_target_type__;
CREATE TYPE log_target_type AS ENUM (
    'group',
    'terms',
    'membership',
    'role',
    'invitation',
    'request'
);
ALTER TABLE logs
    ALTER COLUMN target type log_target_type using target::text::log_target_type;
DROP TYPE log_target_type__;