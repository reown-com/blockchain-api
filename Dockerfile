################################################################################
#
# Build args
#
################################################################################
ARG                 base="rust:buster"
ARG                 runtime="debian:buster-slim"
ARG                 bin="rpc-proxy"
ARG                 version="unknown"
ARG                 sha="unknown"
ARG                 maintainer="WalletConnect"
ARG                 release=""
ARG                 WORK_DIR="/app"

################################################################################
#
# Install cargo-chef
#
################################################################################
FROM                ${base} AS chef

WORKDIR             ${WORK_DIR} 
RUN                 cargo install cargo-chef --locked

################################################################################
#
# Generate recipe file
#
################################################################################
FROM                chef AS plan

WORKDIR             /app
COPY                Cargo.lock Cargo.toml ./
COPY                src ./src
RUN                 cargo chef prepare --recipe-path recipe.json

################################################################################
#
# Build the binary
#
################################################################################
FROM                chef AS build

ARG                 release
ARG                 PROFILE="release-debug"
ARG                 BUILD_PROFILE="--profile ${PROFILE}"

WORKDIR             /app
# Cache dependancies
COPY --from=plan    /app/recipe.json recipe.json
RUN                 cargo chef cook ${BUILD_PROFILE} --recipe-path recipe.json 
# Build the local binary
COPY                . .
RUN                 cargo build --bin rpc-proxy ${RELEASE}

################################################################################
#
# Runtime image
#
################################################################################
FROM                ${runtime} AS runtime

ARG                 bin
ARG                 version
ARG                 sha
ARG                 maintainer
ARG                 release
ARG                 binpath=${release:+release}
ARG                 WORK_DIR 

LABEL               version=${version}
LABEL               sha=${sha}
LABEL               maintainer=${maintainer}

ENV                 RPC_PROXY_HOST=0.0.0.0

WORKDIR             /app
COPY --from=build   /app/target/${binpath:-debug}/rpc-proxy /usr/local/bin/rpc-proxy
RUN                 apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl-dev curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Enable writing files to the work dir.
RUN                 chown -R 1001:1001 ${WORK_DIR}
RUN                 chmod 755 ${WORK_DIR}

USER                1001:1001
EXPOSE              3000/tcp
ENTRYPOINT          ["/usr/local/bin/rpc-proxy"]
