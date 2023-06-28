# Terraform Infrastructure

Get yourself some AWS creds and then init your workspace:

`terraform -chdir=terraform init`

Use the dev workspace:

`terraform -chdir=terraform workspace select dev`

Now you can apply the changes:

`terraform -chdir=terraform apply -var-file="vars/$(terraform -chdir=terraform workspace show).tfvars"`

## Deploying local code changes

```bash
nano .env # set AWS access keys and GRAFANA_AUTH
source .env
./terraform/deploy-dev.sh
```

### macOS considerations

If you get this error:

```
assertion failed [find_leftmost_allocation_holding_lock(interval) == nullptr]: interval being added overlaps existing allocation
(VMAllocationTracker.cpp:322 add)
```

Try disabling "Use Rosetta for x86/amd64 emulation on Apple Silicon" in Docker Desktop settings.

#### Remote building

If amd64 builds are too slow on your Mac (likely), consider using a remote builder on a linux/amd64 host:

```bash
docker buildx create --name=remote-amd64 --driver=docker-container ssh://<my-amd64-host>
BUILD_ARGS="--builder=remote-amd64 --load" ./terraform/deploy-dev.sh
```
