binary-crate            := "."

export JUST_ROOT        := justfile_directory()

# Default to listing recipes
_default:
  @just --list --list-prefix '  > '

# Fast check project for errors
check:
  @echo '==> Checking project for compile errors'
  cargo check

# Build service for development
build:
  @echo '==> Building project'
  cargo build

# Run the service
run: build
  @echo '==> Running project (ctrl+c to exit)'
  cargo run

# Run project test suite, skipping storage tests
test:
  @echo '==> Testing project (default)'
  cargo test

# Run project test suite, including storage tests (requires storage docker services to be running)
test-all:
  @echo '==> Testing project (all features)'
  cargo test --all-features

# Clean build artifacts
clean:
  @echo '==> Cleaning project target/*'
  cargo clean

# Lint the project for any quality issues
lint: check fmt clippy commit-check

# Run project linter
clippy:
  #!/bin/bash
  set -euo pipefail

  if command -v cargo-clippy >/dev/null; then
    echo '==> Running clippy'
    cargo clippy --all-features --tests -- -D clippy::all -W clippy::style
  else
    echo '==> clippy not found in PATH, skipping'
  fi

# Run code formatting check
fmt:
  #!/bin/bash
  set -euo pipefail

  if command -v cargo-fmt >/dev/null; then
    echo '==> Running rustfmt'
    cargo +nightly fmt
  else
    echo '==> rustfmt not found in PATH, skipping'
  fi

  if command -v terraform -version >/dev/null; then
    echo '==> Running terraform fmt'
    terraform -chdir=terraform fmt -recursive
  else
    echo '==> terraform not found in PATH, skipping'
  fi

# Run commit checker
commit-check:
  #!/bin/bash
  set -euo pipefail

  if command -v cog >/dev/null; then
    echo '==> Running cog check'
    cog check --from-latest-tag
  else
    echo '==> cog not found in PATH, skipping'
  fi

lint-tf: tf-validate tf-fmt tfsec tflint

# Check Terraform formating
tf-fmt:
  #!/bin/bash
  set -euo pipefail

  if command -v terraform >/dev/null; then
    echo '==> Running terraform fmt'
    terraform -chdir=terraform fmt -check -recursive
  else
    echo '==> Terraform not found in PATH, skipping'
  fi

tf-validate:
  #!/bin/bash
  set -euo pipefail

  if command -v terraform >/dev/null; then
    echo '==> Running terraform fmt'
    terraform -chdir=terraform validate
  else
    echo '==> Terraform not found in PATH, skipping'
  fi

# Check Terraform for potential security issues
tfsec:
  #!/bin/bash
  set -euo pipefail

  if command -v tfsec >/dev/null; then
    echo '==> Running tfsec'
    cd terraform
    tfsec
  else
    echo '==> tfsec not found in PATH, skipping'
  fi

# Run Terraform linter
tflint:
  #!/bin/bash
  set -euo pipefail

  if command -v tflint >/dev/null; then
    echo '==> Running tflint'
    cd terraform; tflint
    cd ecs; tflint
    cd ../monitoring; tflint
    cd ../private_zone; tflint
    cd ../redis; tflint

  else
    echo '==> tflint not found in PATH, skipping'
  fi
