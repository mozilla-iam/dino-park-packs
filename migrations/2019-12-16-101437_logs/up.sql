CREATE TYPE log_target_type AS ENUM (
    'group',
    'terms',
    'membership',
    'role',
    'invitation'
);

CREATE TYPE log_operation_type AS ENUM (
    'created',
    'deleted',
    'updated'
);

CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    ts TIMESTAMP NOT NULL DEFAULT now(),
    target log_target_type NOT NULL,
    operation log_operation_type NOT NULL,
    group_id SERIAL REFERENCES groups,
    host_uuid UUID NOT NULL,
    user_uuid UUID,
    ok BOOLEAN NOT NULL DEFAULT false, 
    body JSONB
);