CREATE TYPE role_type AS ENUM ('member', 'curator', 'admin');
CREATE TYPE capability_type AS ENUM ('gdrive', 'discourse');
CREATE TYPE permission_type AS ENUM ('invite_member', 'edit_description', 'add_curator', 'remove_curator', 'change_name', 'delete_group', 'remove_member');

CREATE TABLE groups (
    group_id SERIAL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    path VARCHAR NOT NULL,
    description TEXT NOT NULL,
    capabilities capability_type[] NOT NULL
);

CREATE TABLE roles (
    role_id SERIAL PRIMARY KEY,
    group_id SERIAL REFERENCES groups,
    typ role_type NOT NULL DEFAULT 'member',
    name VARCHAR NOT NULL,
    permissions permission_type[] NOT NULL
);

CREATE TABLE memberships (
    user_uuid UUID NOT NULL,
    group_id SERIAL REFERENCES groups,
    role_id SERIAL REFERENCES roles,
    PRIMARY KEY (user_uuid, group_id),
    expiration TIMESTAMP
);

CREATE TABLE invitations (
    invitation_id SERIAL PRIMARY KEY,
    group_id SERIAL REFERENCES groups,
    user_uuid UUID NOT NULL,
    code UUID NOT NULL,
    invitation_expiration TIMESTAMP DEFAULT (NOW() + INTERVAL '1 week'),
    group_expiration TIMESTAMP
);