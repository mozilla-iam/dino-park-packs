# Copy your cookie out of your browser's requests.
#
# For more details, see the OpenAPI Spec:
# * https://github.com/mozilla-iam/dino-park-packs/blob/fc8c06a978ae4d44b571fab766eac3d048672192/docs/openapi.yml#L778-L857
#
# Run with:
# export PEOPLEMO_COOKIE=...
# hurl --secret cookie="$PEOPLEMO_COOKIE" dev/example-group-list.hurl

GET https://dev.people.allizom.org/groups/api/v1/members/dinopark-dev?s=20&by=RoleAsc&r=Any&p=false
Cookie: {{ cookie }}

HTTP 200
[Captures]
next: jsonpath "$['next']"

#GET https://dev.people.allizom.org/groups/api/v1/members/dinopark-dev?n={{ next }}s=20&by=RoleAsc&r=Any&p=false
# Cookie: {{ cookie }}
#
# HTTP 200
