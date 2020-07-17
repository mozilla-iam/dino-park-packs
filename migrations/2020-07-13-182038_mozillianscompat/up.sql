CREATE TABLE invitationtexts (
    group_id SERIAL PRIMARY KEY REFERENCES groups,
    body TEXT NOT NULL
);
