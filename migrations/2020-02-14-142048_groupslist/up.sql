CREATE VIEW groups_list AS
SELECT
    groups.name,
    groups.typ,
    groups.trust,
    count(memberships.user_uuid) AS members_count
FROM
    GROUPS
    JOIN memberships ON groups.group_id = memberships.group_id
GROUP BY
    groups.group_id;

