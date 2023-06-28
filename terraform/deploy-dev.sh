#!/usr/bin/env bash
set -e

TERRAFORM_DIR="$(dirname "$0")"

accountId="$(aws sts get-caller-identity | jq -r .Account)"
region="$(cat $TERRAFORM_DIR/variables.tf | grep -A 2 region | grep default | sed -nr 's/.+default = "(.+)"/\1/p')"

imageRepo="$accountId.dkr.ecr.$region.amazonaws.com/rpc-proxy"

aws ecr get-login-password --region eu-central-1 | docker login --username AWS --password-stdin "$imageRepo"
# --platform=linux/amd64: Must target linux/amd64 as that is what ECS runs.
docker build "$TERRAFORM_DIR/.." -t "$imageRepo" --build-arg=release=true --platform=linux/amd64 $BUILD_ARGS
sha="$(docker inspect --format="{{ .Id }}" "$imageRepo" | cut -d: -f2)"
tag="$imageRepo:$sha"
docker tag "$imageRepo" "$tag"
docker push "$tag"

terraform -chdir="$TERRAFORM_DIR" workspace select dev
TF_VAR_ecr_app_version="$sha" terraform -chdir="$TERRAFORM_DIR" apply -var-file="vars/$(terraform -chdir="$TERRAFORM_DIR" workspace show).tfvars" -auto-approve
