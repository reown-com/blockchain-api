#!/usr/bin/env bash
set -e

TERRAFORM_DIR=$(dirname $0)

accountId="$(aws sts get-caller-identity | jq -r .Account)"
region="$(cat $TERRAFORM_DIR/variables.tf | grep -A 2 region | grep default | sed -nr 's/.+default = "(.+)"/\1/p')"
appVersion="dev"
tag="$accountId.dkr.ecr.$region.amazonaws.com/rpc-proxy:$appVersion"

aws ecr get-login-password --region eu-central-1 | docker login --username AWS --password-stdin "$tag"
# --platform=linux/amd64: Must target linux/amd64 as that is what ECS runs.
docker build $TERRAFORM_DIR/.. -t "$tag" --build-arg=release=true --platform=linux/amd64 $BUILD_ARGS
docker push "$tag"

terraform -chdir=$TERRAFORM_DIR workspace select dev
TF_VAR_ecr_app_version="$appVersion" terraform -chdir=$TERRAFORM_DIR apply -var-file="vars/$(terraform -chdir=$TERRAFORM_DIR workspace show).tfvars"
