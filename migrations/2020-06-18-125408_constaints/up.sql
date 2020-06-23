ALTER TABLE groups
    ADD CONSTRAINT check_name_length CHECK (length(name) >= 3 and length(name) <= 64);