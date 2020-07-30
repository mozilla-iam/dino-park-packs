#!/bin/bash
HOST=${HOST:-localhost:8085}
STAFF_ONLY=${2:-false}
curl -v -F "group=@/tmp/$1/g.tsv" -F "memberships=@/tmp/$1/m.tsv" -F "curators=@/tmp/$1/c.tsv" http://$HOST/import/group/full?staff_only=${STAFF_ONLY}