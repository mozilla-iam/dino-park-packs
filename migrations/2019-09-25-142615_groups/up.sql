CREATE TYPE group_type AS ENUM ('closed', 'reviewd', 'open');
CREATE TYPE role_type AS ENUM ('member', 'curator', 'admin');
CREATE TYPE capability_type AS ENUM ('gdrive', 'discourse');
CREATE TYPE permission_type AS ENUM ('invite_member', 'edit_description', 'add_curator', 'remove_curator', 'change_name', 'delete_group', 'remove_member', 'edit_terms');
CREATE TYPE trust_type AS ENUM ('public', 'authenticated', 'vouched', 'ndaed', 'staff');

CREATE TABLE groups (
    group_id SERIAL PRIMARY KEY,
    name VARCHAR UNIQUE NOT NULL,
    path VARCHAR NOT NULL,
    description TEXT NOT NULL,
    capabilities capability_type[] NOT NULL,
    typ group_type NOT NULL DEFAULT 'closed'
);

CREATE TABLE terms (
    group_id SERIAL PRIMARY KEY REFERENCES groups,
    text TEXT
);

CREATE TABLE roles (
    role_id SERIAL PRIMARY KEY,
    group_id SERIAL REFERENCES groups,
    typ role_type NOT NULL DEFAULT 'member',
    name VARCHAR NOT NULL,
    permissions permission_type[] NOT NULL,
    UNIQUE (group_id, typ)
);

CREATE TABLE memberships (
    user_uuid UUID NOT NULL,
    group_id SERIAL REFERENCES groups,
    role_id SERIAL REFERENCES roles,
    expiration TIMESTAMP,
    added_by UUID,
    added_ts TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_uuid, group_id)
);

CREATE TABLE invitations (
    group_id SERIAL REFERENCES groups,
    user_uuid UUID NOT NULL,
    code UUID NOT NULL,
    invitation_expiration TIMESTAMP DEFAULT (NOW() + INTERVAL '1 week'),
    group_expiration TIMESTAMP,
    added_by UUID,
    PRIMARY KEY (group_id, user_uuid)
);

CREATE TABLE user_ids (
    user_id VARCHAR PRIMARY KEY,
    user_uuid UUID UNIQUE NOT NULL
);

CREATE TABLE users_staff (
    user_uuid UUID PRIMARY KEY,
    picture VARCHAR,
    first_name VARCHAR,
    last_name VARCHAR,
    username VARCHAR,
    email VARCHAR,
    trust trust_type NOT NULL
);

CREATE TABLE users_ndaed (
    user_uuid UUID PRIMARY KEY,
    picture VARCHAR,
    first_name VARCHAR,
    last_name VARCHAR,
    username VARCHAR,
    email VARCHAR,
    trust trust_type NOT NULL
);

CREATE TABLE users_vouched (
    user_uuid UUID PRIMARY KEY,
    picture VARCHAR,
    first_name VARCHAR,
    last_name VARCHAR,
    username VARCHAR,
    email VARCHAR,
    trust trust_type NOT NULL
);

CREATE TABLE users_authenticated (
    user_uuid UUID PRIMARY KEY,
    picture VARCHAR,
    first_name VARCHAR,
    last_name VARCHAR,
    username VARCHAR,
    email VARCHAR,
    trust trust_type NOT NULL
);

CREATE TABLE users_public (
    user_uuid UUID PRIMARY KEY,
    picture VARCHAR,
    first_name VARCHAR,
    last_name VARCHAR,
    username VARCHAR,
    email VARCHAR,
    trust trust_type NOT NULL
);
