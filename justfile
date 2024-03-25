binary-crate  := "."
tf-dir        := "terraform"

export JUST_ROOT := justfile_directory()

nocolor       := '\033[0m'
black         := '\033[0;30m'
red           := '\033[0;31m'
green         := '\033[0;32m'
brown         := '\033[0;33m'
blue          := '\033[0;34m'
purple        := '\033[0;35m'
cyan          := '\033[0;36m'
light-gray    := '\033[0;37m'
dark-gray     := '\033[1;30m'
light-red     := '\033[1;31m'
light-green   := '\033[1;32m'
yellow        := '\033[1;33m'
light-blue    := '\033[1;34m'
light-purple  := '\033[1;35m'
light-cyan    := '\033[1;36m'
white         := '\033[1;37m'

color-cmd     := brown
color-arg     := cyan
color-val     := green
color-hint    := brown
color-desc    := blue
color-service := light-green

# Default to listing recipes
_default:
  @just --list --unsorted

alias build     := cargo-build
alias run       := cargo-run
alias test      := cargo-test
alias clean     := cargo-clean
alias check     := cargo-check
alias clippy    := cargo-clippy
alias udeps     := cargo-udeps
alias checkfmt  := cargo-checkfmt

alias tfsec     := tf-tfsec
alias tflint    := tf-tflint

################################################################################
# Meta recipes

# Format the project code
fmt target='all': (_check-string-in-set target "all,rust,tf")
  #!/bin/bash
  set -euo pipefail
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'rust'  ]] && { just cargo-fmt; }
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'tf'    ]] && { just tf-fmt;   }

# Update project documentation
docs target='all': (_check-string-in-set target "all,rust,tf")
  #!/bin/bash
  set -euo pipefail
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'rust'  ]] && { just cargo-build-docs; }
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'tf'    ]] && { just tf-docs;   }

# Run linting and tests
devloop: lint cargo-test-all

################################################################################
# Linting recipes

# Lint the project for quality issues
lint target='all': (_check-string-in-set target "all,rust,tf")
  #!/bin/bash
  set -euo pipefail
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'rust'  ]] && { just lint-rust;  }
  [[ '{{ target }}' == 'all' || '{{ target }}' == 'tf'    ]] && { just lint-tf;    }


# Lint the rust project for any quality issues
lint-rust: cargo-check cargo-clippy cargo-udeps cargo-checkfmt

# Lint the terrafrom project for any quality issues
lint-tf: tf-checkfmt tf-validate tf-tfsec tf-tflint

################################################################################
# Rust recipes

# Run a Cargo command, choose target from open-docs, build-docs, fmt, build, run, test, clean, check, clippy, udeps, checkfmt
cargo target='' sub-target='': (_check-string-in-set target "open-docs,build-docs,fmt,build,run,test,clean,check,clippy,udeps,checkfmt" "allow_empty")
  #!/bin/bash
  set -euo pipefail

  [[ '{{ target }}' == 'help'  || '{{ target }}' == 'h'  || '{{ target }}' == '' ]] && {
    printf "Available {{ color-cmd }}cargo{{ nocolor }} targets:\n"
    printf "    open-docs             {{ color-desc }}# Open rust project documentation in your local browser{{ nocolor }}\n"
    printf "    build-docs            {{ color-desc }}# Build rust project documentation{{ nocolor }}\n"
    printf "    fmt                   {{ color-desc }}# Format the application code{{ nocolor }}\n"
    printf "    build                 {{ color-desc }}# Build service for development{{ nocolor }}\n"
    printf "    run                   {{ color-desc }}# Run the service{{ nocolor }}\n"
    printf "    test target='default' {{ color-desc }}# Run project tests, choose target from default, all, doc{{ nocolor }}\n"
    printf "    clean                 {{ color-desc }}# Clean build artifacts{{ nocolor }}\n"
    printf "    check                 {{ color-desc }}# Fast check rust project for errors{{ nocolor }}\n"
    printf "    clippy                {{ color-desc }}# Check rust project with clippy{{ nocolor }}\n"
    printf "    udeps                 {{ color-desc }}# Check unused dependencies{{ nocolor }}\n"
    printf "    checkfmt              {{ color-desc }}# Check the rust code formatting{{ nocolor }}\n"

    exit 0
  }

  [[ '{{ target }}' == 'open-docs'  ]] && { just cargo-open-docs;             }
  [[ '{{ target }}' == 'build-docs' ]] && { just cargo-build-docs;            }
  [[ '{{ target }}' == 'fmt'        ]] && { just cargo-fmt;                   }
  [[ '{{ target }}' == 'build'      ]] && { just cargo-build;                 }
  [[ '{{ target }}' == 'run'        ]] && { just cargo-run;                   }
  [[ '{{ target }}' == 'test'       ]] && { just cargo-test {{ sub-target }}; }
  [[ '{{ target }}' == 'clean'      ]] && { just cargo-clean;                 }
  [[ '{{ target }}' == 'check'      ]] && { just cargo-check;                 }
  [[ '{{ target }}' == 'clippy'     ]] && { just cargo-clippy;                }
  [[ '{{ target }}' == 'udeps'      ]] && { just cargo-udeps;                 }
  [[ '{{ target }}' == 'checkfmt'   ]] && { just cargo-checkfmt;              }

# Open rust project documentation in your local browser
cargo-open-docs: (_cargo-build-docs "open" "nodeps")
  @echo '==> Opening documentation in system browser'

# Build rust project documentation
cargo-build-docs: (_cargo-build-docs "" "nodeps")

@_cargo-build-docs $open="" $nodeps="":  _check-cmd-cargo
  echo "==> Building project documentation @$JUST_ROOT/target/doc"
  cargo doc --all-features --document-private-items ${nodeps:+--no-deps} ${open:+--open}

# Format the application code
@cargo-fmt: _check-cmd-cargo-fmt
  printf '==> Running {{ color-cmd }}rustfmt{{ nocolor }}\n'
  cargo +nightly fmt

# Build service for development
cargo-build: _check-cmd-cargo
  @echo '==> Building project'
  cargo build

# Run the service
cargo-run: _check-cmd-cargo cargo-build
  @echo '==> Running project (ctrl+c to exit)'
  cargo run

# Run project tests, choose target from default, all, doc
cargo-test target='default': (_check-string-in-set target "default,all,doc")
  #!/bin/bash
  set -euo pipefail
  [[ "{{ target }}" == 'default' ]] && { just cargo-test-default; }
  [[ "{{ target }}" == 'all'     ]] && { just cargo-test-all;     }
  [[ "{{ target }}" == 'doc'     ]] && { just cargo-test-doc;     }

# Note: Currently broken as lack of test-localhost feature will run integration tests against staging and this uses
# a different env variable. This is redundant with test-all or the integration tests run in CI anyway and will be
# cleaned up later
# Run project default tests
cargo-test-default: _check-cmd-cargo
  @printf '==> Skipping default tests\n'
#  @printf '==> Testing project ({{ light-green }}default{{ nocolor }})\n'
#  cargo test

# Run project tests with all features activated
cargo-test-all: _check-cmd-cargo
  @printf '==> Testing project ({{ light-green }}all features{{ nocolor }})\n'
  cargo +nightly test --all-features

# Run tests from project documentation
cargo-test-doc: _check-cmd-cargo
  @printf '==> Testing project ({{ light-green }}docs{{ nocolor }})\n'
  cargo test --doc

# Clean build artifacts
cargo-clean: _check-cmd-cargo
  @printf '==> Cleaning project target/*\n'
  cargo clean

# Fast check rust project for errors
cargo-check: _check-cmd-cargo
  @printf '==> Checking project for compile errors\n'
  cargo check

# Check rust project with clippy
cargo-clippy: _check-cmd-cargo-clippy
  @printf '==> Running {{ color-cmd }}clippy{{ nocolor }}\n'
  cargo +nightly clippy --all-features --tests -- -D clippy::all

# Check unused dependencies
cargo-udeps: _check-cmd-cargo-udeps
  @printf '==> Running {{ color-cmd }}udeps{{ nocolor }}\n'
  cargo +nightly udeps

# Check the rust code formatting
cargo-checkfmt: _check-cmd-cargo-fmt
  @printf '==> Running {{ color-cmd }}rustfmt{{ nocolor }} --check\n'
  cargo +nightly fmt --check

################################################################################
# Terraform recipes

# Run a Terraform command, choose target from build, run, stop, clean, ps, test
tf target='': (_check-string-in-set target "fmt,checkfmt,validate,tfsec,tflint,init,plan,apply,docs,clean" "allow_empty")
  #!/bin/bash
  set -euo pipefail

  [[ '{{ target }}' == 'help'  || '{{ target }}' == 'h'  || '{{ target }}' == '' ]] && {
    printf "Available {{ color-cmd }}Terraform{{ nocolor }} targets:\n"
    printf "    fmt      {{ color-desc }}# Format the terraform code{{ nocolor }}\n"
    printf "    checkfmt {{ color-desc }}# Check Terraform formatting{{ nocolor }}\n"
    printf "    validate {{ color-desc }}# Run Terraform validation{{ nocolor }}\n"
    printf "    tfsec    {{ color-desc }}# Check Terraform configuration for potential security issues{{ nocolor }}\n"
    printf "    tflint   {{ color-desc }}# Run Terraform linter{{ nocolor }}\n"
    printf "    init     {{ color-desc }}# Init Terraform project{{ nocolor }}\n"
    printf "    plan     {{ color-desc }}# Perform a Terraform plan on the current workspace{{ nocolor }}\n"
    printf "    apply    {{ color-desc }}# Perform a Terraform apply on the current workspace{{ nocolor }}\n"
    printf "    docs     {{ color-desc }}# Update the Terraform documentation{{ nocolor }}\n"
    printf "    clean    {{ color-desc }}# Clean the Terraform environment{{ nocolor }}\n"

    exit 0
  }

  [[ '{{ target }}' == 'fmt'      ]] && { just tf-fmt;      }
  [[ '{{ target }}' == 'checkfmt' ]] && { just tf-checkfmt; }
  [[ '{{ target }}' == 'validate' ]] && { just tf-validate; }
  [[ '{{ target }}' == 'tfsec'    ]] && { just tf-tfsec;    }
  [[ '{{ target }}' == 'tflint'   ]] && { just tf-tflint;   }
  [[ '{{ target }}' == 'init'     ]] && { just tf-init;     }
  [[ '{{ target }}' == 'plan'     ]] && { just tf-plan;     }
  [[ '{{ target }}' == 'apply'    ]] && { just tf-apply;    }
  [[ '{{ target }}' == 'docs'     ]] && { just tf-docs;     }
  [[ '{{ target }}' == 'clean'    ]] && { just tf-clean;    }

# Format the terraform code
@tf-fmt: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform fmt{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform fmt -recursive

# Check Terraform formatting
@tf-checkfmt: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform fmt{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform fmt -check -recursive

# Run Terraform validation
@tf-validate: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform fmt{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform validate

# Check Terraform configuration for potential security issues
@tf-tfsec: _check-cmd-tfsec
  printf '==> Running {{ color-cmd }}tfsec{{ nocolor }}\n'
  cd {{ tf-dir }}; tfsec

# Run Terraform linter
@tf-tflint: _check-cmd-tflint
  printf '==> Running {{ color-cmd }}tflint{{ nocolor }}\n'
  cd {{ tf-dir }}; tflint --recursive

# Init Terraform project
@tf-init: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform init{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform init

# Perform a Terraform plan on the current workspace
@tf-plan: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform init{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform plan

# Perform a Terraform apply on the current workspace
@tf-apply: _check-cmd-terraform
  printf '==> Running {{ color-cmd }}terraform init{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform apply

# Update the Terraform documentation
@tf-docs: _check-cmd-tfdocs
  printf '==> Running {{ color-cmd }}terraform-docs{{ nocolor }}\n'
  cd {{ tf-dir }}; terraform-docs .

# Clean the Terraform environment
@tf-clean:
  printf '==> Clean Terraform environment\n'
  cd {{ tf-dir }}; rm -rf .terraform/ .terraform.lock.hcl

################################################################################
# Helper recipes

_check-cmd-cargo:           (_check-cmd 'cargo'           'To install see https://doc.rust-lang.org/cargo/getting-started/installation.html for details')
_check-cmd-cargo-fmt:       (_check-cmd 'cargo-fmt'       'To install run ' + color-hint + '`rustup component add rustfmt`' + nocolor + ', see https://github.com/rust-lang/rustfmt for details')
_check-cmd-cargo-clippy:    (_check-cmd 'cargo-clippy'    'To install run ' + color-hint + '`rustup component add clippy`' + nocolor + ', see https://github.com/rust-lang/rust-clippy for details')
_check-cmd-cargo-udeps:     (_check-cmd 'cargo-udeps'     'To install run ' + color-hint + '`cargo install cargo-udeps --locked`' + nocolor + ', see https://github.com/est31/cargo-udeps for details')
_check-cmd-terraform:       (_check-cmd 'terraform'       'To install see https://developer.hashicorp.com/terraform/downloads')
_check-cmd-tfsec:           (_check-cmd 'tfsec'           'To install see https://github.com/aquasecurity/tfsec#installation')
_check-cmd-tflint:          (_check-cmd 'tflint'          'To install see https://github.com/terraform-linters/tflint#installation')
_check-cmd-tfdocs:          (_check-cmd 'terraform-docs'  'To install see https://terraform-docs.io/user-guide/installation/')

[no-exit-message]
_check-cmd cmd install:
  #!/bin/bash
  set -euo pipefail

  cmd="{{ cmd }}"
  numChars=${#cmd}
  underline=$(printf '%*s' "$numChars" | tr ' ' '^')

  if ! command -v {{ cmd }} >/dev/null; then
    printf '==> {{ color-cmd }}{{ cmd }}{{ nocolor }} not found in PATH\n'
    printf '    %s {{ install }}\n' "$underline"
    exit 1
  fi

[no-exit-message]
_check-string-in-set target set options='':
  #!/bin/bash
  set -euo pipefail

  target="{{ target }}"
  set="{{ set }}"
  options="{{ options }}"

  if ! [[ -z "$target" && "$options" == "allow_empty" ]]; then
    # Convert the set into an array
    IFS=',' read -ra setArray <<< "$set"

    # Check if target is in the setArray
    found=false
    for item in "${setArray[@]}"; do
      if [[ "$item" == "$target" ]]; then
        found=true
        break
      fi
    done

    if [[ "$found" != true ]]; then
      printf "{{red }}$target{{ nocolor }} is not a valid target, accepted values are {{ brown }}[${set}]{{ nocolor }}\n"
      exit 1
    fi
  fi
