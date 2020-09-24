ALTER TABLE GROUPS
    ADD CONSTRAINT mintrust CHECK (trust > 'public');

ALTER TABLE profiles
    ADD COLUMN trust trust_type NOT NULL DEFAULT 'public';

UPDATE
    profiles p
SET
    trust = u.trust
FROM
    users_staff u
WHERE
    p.user_uuid = u.user_uuid;
