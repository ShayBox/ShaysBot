ARG RUST_IMAGE_VERSION="1.91.1-alpine3.20"
ARG ALPINE_IMAGE_VERSION="3.22.2"
ARG USER="shay"
ARG UID="1000"

####################################################################################################

FROM rust:${RUST_IMAGE_VERSION} AS build_base
WORKDIR /build

RUN apk update && apk add g++
RUN cargo +nightly install cargo-chef --locked --version 0.1.73

####################################################################################################

FROM build_base AS build_plan
COPY . .
RUN cargo +nightly chef prepare --recipe-path recipe.json

####################################################################################################

FROM build_base AS build

COPY --from=build_plan /build/recipe.json recipe.json
RUN cargo +nightly chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo +nightly build --release --locked --bin shaysbot

####################################################################################################

FROM alpine:${ALPINE_IMAGE_VERSION} AS run
ARG USER
ARG UID
ARG GID
WORKDIR /config

RUN <<EOF
    adduser \
        --disabled-password \
        --no-create-home \
        --uid "${UID}" \
        "${USER}"
    chown -R "${USER}":"${USER}" /config 
EOF
USER ${USER}

COPY --from=build /build/target/release/shaysbot /usr/local/bin/shaysbot
ENTRYPOINT [ "shaysbot" ]
