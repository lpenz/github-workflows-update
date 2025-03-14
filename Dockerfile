# Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
# This file is subject to the terms and conditions defined in
# file 'LICENSE', which is part of this source code package.

FROM docker.io/rust:1.85-slim-bookworm AS build
ENV DEBIAN_FRONTEND=noninteractive
WORKDIR /src
COPY Cargo.* ./
COPY src ./src
COPY build.rs ./
RUN set -e -x; \
    apt-get update; \
    apt-get install -y --no-install-recommends libssl-dev pkg-config; \
    cargo build --release

FROM docker.io/debian:bookworm-slim
ENV DEBIAN_FRONTEND=noninteractive
RUN set -e -x; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates
COPY --from=build /src/target/release/github-workflows-update /usr/local/bin/
CMD ["/usr/local/bin/github-workflows-update", "-n", "--output-format", "github-warning", "--error-on-outdated"]
