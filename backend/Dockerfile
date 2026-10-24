# Repository-root context is required for the embedded API specification.
# Immutable Rust 1.98.0 toolchain, compatible with the existing lockfile.
FROM rust:1-bookworm@sha256:82150a52ec202c1b14d7817e14516c392bb7f5cfebd88f1ed531cb37ebd39922 AS builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/src ./backend/src
COPY backend/migrations ./backend/migrations
COPY docs/openapi.json ./docs/openapi.json
# Limit peak build memory on free builders and retain dependencies across builds.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/backend/target \
    cargo build --release --locked --jobs 2 --manifest-path backend/Cargo.toml \
    && strip backend/target/release/backend \
    && cp backend/target/release/backend /app/smartride

FROM debian:bookworm-slim AS runtime
# Reuse trusted CA certificates; the runtime needs no package installation.
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /app/smartride /usr/local/bin/smartride
USER 10001:10001
# Hosts may override PORT. Demo mode must always be explicitly enabled.
ENV PORT=8080 APP_MODE=production POSTCODE_PROVIDER=mock RUST_LOG=info
EXPOSE 8080
HEALTHCHECK --start-period=90s --interval=30s --timeout=10s --retries=3 \
    CMD ["smartride", "--healthcheck"]
CMD ["smartride"]
