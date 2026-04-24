# syntax=docker/dockerfile:1.5

FROM rust:1 as base

LABEL maintainer="adorsys Cameroon"

ENV CARGO_TERM_COLOR=always
ENV OPENSSL_STATIC=1

WORKDIR /app

FROM base as builder

ARG TARGETARCH

# Install toolchain and dependencies for static musl builds with vendored OpenSSL
RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt-get update && \
  apt-get install -y --no-install-recommends \
    musl-tools \
    build-essential \
    pkg-config \
    perl \
  && rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

RUN \
  # Mount workspace files and only the necessary crates
  --mount=type=bind,source=./Cargo.toml,target=/app/Cargo.toml \
  --mount=type=bind,source=./Cargo.lock,target=/app/Cargo.lock \
  --mount=type=bind,source=./src,target=/app/src \
  --mount=type=cache,target=/app/target \
  --mount=type=cache,target=/usr/local/cargo/registry/cache \
  --mount=type=cache,target=/usr/local/cargo/registry/index \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  case "$TARGETARCH" in \
    "amd64") \
      export RUST_TARGET=x86_64-unknown-linux-musl; \
      export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc; \
      ;; \
    "arm64") \
      export RUST_TARGET=aarch64-unknown-linux-musl; \
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc; \
      ;; \
    *) \
      echo "Unsupported TARGETARCH: $TARGETARCH"; \
      exit 1; \
      ;; \
  esac; \
  cargo build --release --locked \
    --target "${RUST_TARGET}" \
    -p image-mcp \
  && cp ./target/"${RUST_TARGET}"/release/image-mcp image-mcp \
  && mkdir -p outputs && chmod 777 outputs

FROM gcr.io/distroless/static-debian12:nonroot as oauth2

LABEL maintainer="Stephane Segning <selastlambou@gmail.com>"
LABEL org.opencontainers.image.description="adorsys GIS Cameroon"

ENV RUST_LOG=warn
ENV PORT=8000

WORKDIR /app

COPY --from=builder /app/image-mcp /app/image-mcp
COPY --from=builder /app/outputs /app/outputs

USER nonroot:nonroot

EXPOSE $PORT

ENTRYPOINT ["/app/image-mcp", "--transport-mode", "http"]