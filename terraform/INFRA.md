### Set up database after terraform apply

```
kubectl run -i -t --rm --image=postgres --command debug1 -- /bin/bash
psql -U dinopark -W oneTimePassword -h XXX.rds.amazonaws.com postgres
alter user dinopark with encrypted password 'XXX';
create database "dino-park-packs";
```
