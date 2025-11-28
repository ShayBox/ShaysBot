ARG RUST_IMAGE_VERSION="1.91.1-alpine3.20"
ARG ALPINE_IMAGE_VERSION="3.22.2"
ARG TARGET_PLATFORM="unknowntarget"
ARG USER="shay"
ARG UID="1000"
ARG GID="1000"

# Because we can't use variable expansion in `RUN --mount=from=`, we use this to alias the image
FROM rust:${RUST_IMAGE_VERSION} AS build_base
RUN apk update && apk add g++

####################################################################################################
FROM build_base AS build
ARG TARGET_PLATFORM
WORKDIR /app

COPY . .

# Mounts are complicated. See https://github.com/rust-lang/cargo/issues/2644#issuecomment-570749508
# We set `id` to separate caches by target platform so we can build for multiple architectures
RUN \
    --mount=type=cache,target=/usr/local/cargo,from=build_base,source=/usr/local/cargo,id=${TARGET_PLATFORM}_cargohome \
    --mount=type=cache,target=/app/target,id=${TARGET_PLATFORM}_target <<EOF
    /usr/local/cargo/bin/cargo build --release --locked
    cp ./target/release/shaysbot /app/shaysbot
EOF

####################################################################################################
FROM alpine:${ALPINE_IMAGE_VERSION} AS run
ARG USER
ARG UID
ARG GID
WORKDIR /app

RUN <<EOF
    addgroup \
        --gid "${GID}" \
        "${USER}"
    adduser \
        --disabled-password \
        --ingroup shay \
        --no-create-home \
        --uid "${UID}" \
        "${USER}"
    chown -R "${USER}":"${USER}" /app 
EOF
USER ${USER}

# Create default config files, so a bind mount to their locations will work out-of-the-box.
COPY ./example-config/* .

COPY \
    --from=build \
    --chown=${UID}:${GID} \
    /app/shaysbot .
CMD ["./shaysbot"]
