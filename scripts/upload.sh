#!/bin/bash
HOST=${HOST:-localhost:8085}
TRUST=${2:-Ndaed}
curl -v -F "group=@/tmp/$1/g.tsv" -F "memberships=@/tmp/$1/m.tsv" -F "curators=@/tmp/$1/c.tsv" http://$HOST/import/group/full?trust=${TRUST}