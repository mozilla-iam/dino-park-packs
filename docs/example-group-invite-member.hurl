# Copy your cookie out of your browser's requests.
#
# For more details, see the OpenAPI Spec:
# * Finding members not in a group: https://github.com/mozilla-iam/dino-park-packs/blob/fc8c06a978ae4d44b571fab766eac3d048672192/docs/openapi.yml#L943-L992
# * Invite a member: https://github.com/mozilla-iam/dino-park-packs/blob/fc8c06a978ae4d44b571fab766eac3d048672192/docs/openapi.yml#L490-L517
# * The `Invitation` struct: https://github.com/mozilla-iam/dino-park-packs/blob/fc8c06a978ae4d44b571fab766eac3d048672192/src/api/invitations.rs#L15-L26
#
# Run with:
# export PEOPLEMO_COOKIE=...
# hurl --secret cookie="$PEOPLEMO_COOKIE" --variable group="dinopark-dev" --variable username="example" dev/example-group-invite-member.hurl

# Get their uuid
GET https://dev.people.allizom.org/groups/api/v1/users?q={{ username }}&g={{ group }}&c=false&a=true
Cookie: {{ cookie }}
HTTP 200
[Captures]
user_uuid: jsonpath "$[0]['user_uuid']"

# Invite them to the group.
POST https://dev.people.allizom.org/groups/api/v1/invitations/dinopark-dev
Content-Type: application/json
Cookie: {{ cookie }}

{
    "user_uuid": "{{ user_uuid }}",
    "invitations_expiration": 14,
    "group_expiration": 180
}

HTTP 200
