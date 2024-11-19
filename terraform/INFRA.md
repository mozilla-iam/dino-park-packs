### Set up database after terraform apply

```
kubectl run -i -t --rm --image=postgres --command debug1 -- /bin/bash
psql -U dinopark -W oneTimePassword -h XXX.rds.amazonaws.com postgres
alter user dinopark with encrypted password 'XXX';
create database "dino-park-packs";
```

### Setting up Terraform on an ARM Mac

New developers at Mozilla are issued an ARM-based Mac, and so the installation
steps will be different.

To get started, install Terraform (using [asdf](https://asdf-vm.com/)):

```
ASDF_HASHICORP_OVERWRITE_ARCH=amd64 asdf install terraform 0.12.31
```

Next, you'll need to export your credentials:

```
$(aws configure export-credentials --format env --profile iam-admin)
```

(Consider using something like [direnv](https://direnv.net/).)

Next, initialize the working directory:

```
arch -arch x86_64 terraform init
```

And then, finally, when you're ready, generate a plan:

```
arch -arch x86_64 terraform plan -out plan
```

Once you're ready, you can ship your changes using:

```
arch -arch x86_64 terraform apply plan
```
