ALTER TABLE groups
    ADD CONSTRAINT mintrust CHECK (trust > 'public');